//! Read-only, Bevy-style inspector for the decoded world in a Halo Infinite film.
//!
//! Run with the retained fixture:
//! `cargo run --example film_world_inspector`
//!
//! Or inspect chunks saved by `download_film`:
//! `cargo run --example film_world_inspector -- /path/to/chunk-directory`
//!
//! The retained `experiments/films` corpus predates the downloaded-zlib input
//! contract, so only its `film.json`-listed clear chunks are wrapped in zlib.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use eframe::egui;
use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::film::{EntityId, KeyframeId, ReplicationField, ReplicationValue};
use halo_api::theater::runtime::EntityState;
use halo_api::theater::{Film, TheaterRuntime};
use serde::Deserialize;

type Error = Box<dyn std::error::Error>;

#[derive(Deserialize)]
struct FixtureManifest {
    chunks: Vec<FixtureChunk>,
}

#[derive(Deserialize)]
struct FixtureChunk {
    file: String,
    index: i32,
    chunk_type: i32,
    start_time_offset_ms: i64,
}

fn zlib_wrap(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

fn load_chunks(directory: &Path) -> Result<Vec<RawFilmChunk>, Error> {
    let manifest_path = directory.join("film.json");
    if manifest_path.exists() {
        // These historical corpus files contain clear bytes. This conversion
        // is strictly an input adapter for the example, not parser behavior.
        let manifest: FixtureManifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
        return manifest
            .chunks
            .into_iter()
            .map(|chunk| {
                let path = directory.join(chunk.file);
                Ok(RawFilmChunk {
                    kind: RawChunkKind::from(chunk.chunk_type),
                    index: chunk.index,
                    start_ms: chunk.start_time_offset_ms,
                    data: zlib_wrap(&fs::read(path)?)?,
                })
            })
            .collect();
    }

    // `download_film` writes the original zlib streams with this file name.
    let mut chunks = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(stem) = name
            .strip_prefix("chunk-")
            .and_then(|s| s.strip_suffix(".bin"))
        else {
            continue;
        };
        let Some((index, kind)) = stem.split_once("-type-") else {
            continue;
        };
        chunks.push(RawFilmChunk {
            kind: RawChunkKind::from(kind.parse::<i32>()?),
            index: index.parse::<i32>()?,
            // A download directory has filenames but no saved manifest timing.
            // Playback uses timestamps decoded from the replication stream.
            start_ms: 0,
            data: fs::read(path)?,
        });
    }
    if chunks.is_empty() {
        return Err(format!("no film chunks found in {}", directory.display()).into());
    }
    Ok(chunks)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Live(EntityId),
    Keyframe(KeyframeId),
}

struct Inspector {
    runtime: TheaterRuntime,
    times: Vec<u64>,
    time_us: u64,
    search: String,
    selection: Option<Selection>,
    show_keyframes: bool,
    source: String,
}

impl Inspector {
    fn new(film: Film, source: String) -> Self {
        let mut runtime = TheaterRuntime::load(film);
        let mut times = runtime
            .observations()
            .iter()
            .map(|observation| observation.timestamp_us)
            .collect::<Vec<_>>();
        times.sort_unstable();
        times.dedup();
        runtime.advance_to(0);
        Self {
            runtime,
            times,
            time_us: 0,
            search: String::new(),
            selection: None,
            show_keyframes: false,
            source,
        }
    }

    fn step(&mut self, forward: bool) {
        let next = self.times.partition_point(|&time| time <= self.time_us);
        if forward {
            if let Some(&time) = self.times.get(next) {
                self.time_us = time;
            }
        } else {
            let previous = self.times.partition_point(|&time| time < self.time_us);
            self.time_us = previous
                .checked_sub(1)
                .and_then(|index| self.times.get(index).copied())
                .unwrap_or(0);
        }
    }
}

impl eframe::App for Inspector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let end_us = self.times.last().copied().unwrap_or(0);
        egui::TopBottomPanel::top("timeline").show(ctx, |ui| {
            ui.heading("Halo Infinite film world inspector");
            ui.label(&self.source);
            ui.horizontal(|ui| {
                if ui.button("⏮ Start").clicked() {
                    self.time_us = 0;
                }
                if ui.button("◀ Previous observation").clicked() {
                    self.step(false);
                }
                if ui.button("Next observation ▶").clicked() {
                    self.step(true);
                }
                if ui.button("End ⏭").clicked() {
                    self.time_us = end_us;
                }
            });
            ui.add(egui::Slider::new(&mut self.time_us, 0..=end_us).text("Film time (µs)"));
            ui.label(format!(
                "{:.3} s / {:.3} s",
                self.time_us as f64 / 1e6,
                end_us as f64 / 1e6
            ));
        });

        if self.runtime.timestamp_us() != Some(self.time_us) {
            self.runtime.advance_to(self.time_us);
        }
        let world = self.runtime.current();

        egui::SidePanel::left("entities")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.heading("Entities");
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search ID, archetype, component, or field"),
                );
                ui.checkbox(&mut self.show_keyframes, "Show keyframe baselines");
                let search = self.search.trim().to_lowercase();
                ui.label(format!(
                    "{} live · {} keyframes",
                    world.entities.len(),
                    world.keyframes.len()
                ));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (&id, state) in &world.entities {
                        if !matches_search(id.raw(), state, &search) {
                            continue;
                        }
                        let label = format!(
                            "{} · archetype {} · {} components",
                            id.raw(),
                            state.archetype.raw(),
                            state.components.len()
                        );
                        if ui
                            .selectable_label(self.selection == Some(Selection::Live(id)), label)
                            .clicked()
                        {
                            self.selection = Some(Selection::Live(id));
                        }
                    }
                    if self.show_keyframes {
                        ui.separator();
                        ui.label("Keyframe baselines (separate ID domain)");
                        for (&id, state) in &world.keyframes {
                            if !matches_search(id.raw(), state, &search) {
                                continue;
                            }
                            let label = format!(
                                "{} · archetype {} · {} components",
                                id.raw(),
                                state.archetype.raw(),
                                state.components.len()
                            );
                            if ui
                                .selectable_label(
                                    self.selection == Some(Selection::Keyframe(id)),
                                    label,
                                )
                                .clicked()
                            {
                                self.selection = Some(Selection::Keyframe(id));
                            }
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let found = match self.selection {
                Some(Selection::Live(id)) => world.entities.get(&id).map(|state| {
                    show_entity(
                        ui,
                        &format!(
                            "Entity {} (generation {}, slot {})",
                            id.raw(),
                            id.generation(),
                            id.slot()
                        ),
                        state,
                    );
                }),
                Some(Selection::Keyframe(id)) => world.keyframes.get(&id).map(|state| {
                    show_entity(ui, &format!("Keyframe baseline {}", id.raw()), state);
                }),
                None => None,
            };
            if found.is_none() {
                ui.heading("Select a live entity");
                ui.label("Move the timeline or choose an entity on the left. An entity disappears from the live list after its recorded deletion.");
            }
        });

        egui::TopBottomPanel::bottom("coverage").show(ctx, |ui| {
            ui.small("Read-only decoded film state. Incomplete observations are omitted. Keyframe IDs are not assumed to identify live entities; no parent/child hierarchy is inferred.");
        });
    }
}

fn show_entity<Id>(ui: &mut egui::Ui, title: &str, state: &EntityState<Id>) {
    ui.heading(title);
    let created = state
        .created_at_us
        .map_or("keyframe baseline".to_owned(), |time| {
            format!("created at {:.3} s", time as f64 / 1e6)
        });
    ui.label(format!(
        "Archetype {} · {created} · last update {:.3} s",
        state.archetype.raw(),
        state.updated_at_us as f64 / 1e6
    ));
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        if !state.default_fields.is_empty() {
            egui::CollapsingHeader::new(format!(
                "Creation defaults ({})",
                state.default_fields.len()
            ))
            .show(ui, |ui| show_fields(ui, &state.default_fields));
        }
        if !state.fields.is_empty() {
            egui::CollapsingHeader::new(format!("Latest entity fields ({})", state.fields.len()))
                .show(ui, |ui| show_fields(ui, &state.fields));
        }
        ui.heading("Components");
        for (slot, component) in &state.components {
            let label = format!("{} · slot {}", component.name, slot.index());
            ui.push_id(slot.index(), |ui| {
                egui::CollapsingHeader::new(label).show(ui, |ui| {
                    if let Some(weapon) = component.weapon_id {
                        let name = weapon.name().unwrap_or("Unknown weapon");
                        ui.label(format!("Weapon: {name} ({})", weapon.id()));
                    }
                    show_fields(ui, &component.fields);
                });
            });
        }
        if state.components.is_empty() {
            ui.label("No decoded components at this time.");
        }
    });
}

fn matches_search<Id>(id: u32, state: &EntityState<Id>, search: &str) -> bool {
    search.is_empty()
        || id.to_string().contains(search)
        || format!("archetype {}", state.archetype.raw()).contains(search)
        || state
            .default_fields
            .iter()
            .chain(&state.fields)
            .any(|field| field.name.to_lowercase().contains(search))
        || state.components.values().any(|component| {
            component.name.to_lowercase().contains(search)
                || component
                    .fields
                    .iter()
                    .any(|field| field.name.to_lowercase().contains(search))
                || component
                    .weapon_id
                    .and_then(|weapon| weapon.name())
                    .is_some_and(|name| name.to_lowercase().contains(search))
        })
}

fn show_fields(ui: &mut egui::Ui, fields: &[ReplicationField]) {
    if fields.is_empty() {
        ui.label("No decoded fields.");
    }
    for field in fields {
        ui.horizontal_wrapped(|ui| {
            ui.strong(&field.name);
            match &field.value {
                ReplicationValue::Empty => {
                    ui.monospace("empty");
                }
                ReplicationValue::Scalar { value, bit_len } => {
                    ui.monospace(format!("{value} (0x{value:x}, {bit_len} bits)"));
                }
                ReplicationValue::Tag(tag) => {
                    ui.monospace(format!("tag 0x{:08x}", tag.raw()));
                }
                ReplicationValue::Wide { bit_len, bytes } => {
                    ui.monospace(format!("{bit_len} bits: {}", hex_bytes(bytes)));
                }
            }
        });
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}

fn default_film_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films/aim/02-yaw-right")
}

fn main() -> Result<(), Error> {
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_film_directory);
    let chunks = load_chunks(&directory)?;
    let film = Film::parse(chunks)?;
    let source = directory.display().to_string();
    eframe::run_native(
        "Halo Infinite film world inspector",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1250.0, 800.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(Inspector::new(film, source)))),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_film_supports_inspection() {
        let chunks = load_chunks(&default_film_directory()).unwrap();
        let film = Film::parse(chunks).unwrap();
        let mut inspector = Inspector::new(film, "bundled film".to_owned());
        assert!(!inspector.times.is_empty());
        let end_us = *inspector.times.last().unwrap();
        inspector.runtime.advance_to(end_us);
        assert!(!inspector.runtime.current().entities.is_empty());
    }
}
