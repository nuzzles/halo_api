//! Read-only, Bevy-style inspector for the decoded world in a Halo Infinite film.
//!
//! Run with the locally retained Ranked Arena Oddball match, when available:
//! `cargo run --example film_world_inspector`
//!
//! Or inspect chunks saved by `download_film`:
//! `cargo run --example film_world_inspector -- /path/to/chunk-directory`
//!
//! The retained `.local/theater/films` corpus predates the downloaded-zlib input
//! contract, so only its `film.json`-listed clear chunks are wrapped in zlib.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use eframe::egui;
use halo_api::clients::hi::models::{RawChunkKind, RawFilmChunk};
use halo_api::theater::film::{
    DamageReference, EntityId, EntitySchemaId, EntitySchemaKind, KeyframeId, ProjectileReference,
    ReplicationField, ReplicationValue, TagKind, TagLink, TagReference,
};
use halo_api::theater::runtime::EntityState;
use halo_api::theater::{Film, FilmRuntime};
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

fn bundled_fixtures() -> Vec<(String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local/theater/films");
    let mut fixtures = Vec::new();
    if let Ok(categories) = fs::read_dir(root) {
        for category in categories.flatten() {
            let Ok(entries) = fs::read_dir(category.path()) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("film.json").is_file() {
                    let label = format!(
                        "{} / {}",
                        category.file_name().to_string_lossy(),
                        entry.file_name().to_string_lossy()
                    );
                    fixtures.push((label, path));
                }
            }
        }
    }
    fixtures.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    fixtures
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Live(EntityId),
    Keyframe(KeyframeId),
}

struct Inspector {
    runtime: FilmRuntime,
    times: Vec<u64>,
    start_us: u64,
    time_us: u64,
    search: String,
    selection: Option<Selection>,
    show_keyframes: bool,
    source: String,
    fixtures: Vec<(String, PathBuf)>,
    directory_input: String,
    loading: Option<Receiver<Result<(Film, String), String>>>,
    load_error: Option<String>,
}

impl Inspector {
    fn new(film: Film, source: String) -> Self {
        let mut runtime = FilmRuntime::load(film);
        let mut times = runtime
            .film()
            .replication_stream
            .observations
            .iter()
            .map(|observation| observation.timestamp_us)
            .collect::<Vec<_>>();
        times.sort_unstable();
        times.dedup();
        let start_us = times.first().copied().unwrap_or(0);
        runtime.advance_to(start_us);
        Self {
            runtime,
            times,
            start_us,
            time_us: start_us,
            search: String::new(),
            selection: None,
            show_keyframes: false,
            directory_input: source.clone(),
            source,
            fixtures: bundled_fixtures(),
            loading: None,
            load_error: None,
        }
    }

    fn open_fixture(&mut self, directory: PathBuf) {
        let (sender, receiver) = mpsc::channel();
        self.loading = Some(receiver);
        self.load_error = None;
        std::thread::spawn(move || {
            let result = load_chunks(&directory)
                .and_then(|chunks| Film::parse(chunks).map_err(Into::into))
                .map(|film| (film, directory.display().to_string()))
                .map_err(|error: Error| error.to_string());
            let _ = sender.send(result);
        });
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
                .unwrap_or(self.start_us);
        }
    }
}

impl eframe::App for Inspector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.loading {
            match receiver.try_recv() {
                Ok(Ok((film, source))) => *self = Inspector::new(film, source),
                Ok(Err(error)) => {
                    self.load_error = Some(error);
                    self.loading = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.load_error = Some("Film loader stopped unexpectedly".to_owned());
                    self.loading = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(100))
                }
            }
        }

        let mut selected_fixture = None;
        egui::TopBottomPanel::top("file_menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.add_enabled_ui(self.loading.is_none(), |ui| {
                        if !self.fixtures.is_empty() {
                            ui.menu_button("Open fixture", |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(400.0)
                                    .show(ui, |ui| {
                                        for (label, path) in &self.fixtures {
                                            if ui.button(label).clicked() {
                                                selected_fixture = Some(path.clone());
                                                ui.close();
                                            }
                                        }
                                    });
                            });
                        }
                        ui.menu_button("Open directory", |ui| {
                            ui.text_edit_singleline(&mut self.directory_input);
                            if ui.button("Open").clicked() {
                                selected_fixture = Some(PathBuf::from(&self.directory_input));
                                ui.close();
                            }
                        });
                    });
                });
                if self.loading.is_some() {
                    ui.spinner();
                    ui.label("Loading fixture…");
                } else {
                    ui.label(&self.source);
                }
            });
            if let Some(error) = &self.load_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        });
        if let Some(path) = selected_fixture {
            self.open_fixture(path);
        }

        let end_us = self.times.last().copied().unwrap_or(0);
        egui::TopBottomPanel::bottom("timeline").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏮").on_hover_text("Start").clicked() {
                    self.time_us = self.start_us;
                }
                if ui
                    .button("◀")
                    .on_hover_text("Previous observation")
                    .clicked()
                {
                    self.step(false);
                }
                if ui.button("▶").on_hover_text("Next observation").clicked() {
                    self.step(true);
                }
                if ui.button("⏭").on_hover_text("End").clicked() {
                    self.time_us = end_us;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{} / {}",
                        format_elapsed(self.time_us.saturating_sub(self.start_us)),
                        format_elapsed(end_us.saturating_sub(self.start_us))
                    ));
                    ui.spacing_mut().slider_width =
                        (ui.available_width() - ui.spacing().item_spacing.x).max(0.0);
                    ui.add(
                        egui::Slider::new(&mut self.time_us, self.start_us..=end_us)
                            .show_value(false),
                    );
                });
            });
        });
        if self.runtime.timestamp_us() != Some(self.time_us) {
            self.runtime.advance_to(self.time_us);
        }
        let world = self.runtime.current();

        egui::SidePanel::left("entities")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.heading("World");
                ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text("Search ID, game name, schema, component, or field"),
                );
                let search = self.search.trim().to_lowercase();
                egui::CollapsingHeader::new(format!("Entities ({})", world.entities.len()))
                    .id_salt("entities_list")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.show_keyframes, "Show keyframe baselines");
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (&id, state) in &world.entities {
                                    if !matches_search(&id.to_string(), id.raw(), state, &search) {
                                        continue;
                                    }
                                    let label = format!(
                                        "Entity {} ({}) +{}",
                                        id,
                                        schema_name_label(state.schema_kind, state.schema),
                                        state.components.len()
                                    );
                                    if ui
                                        .add(
                                            egui::Button::selectable(
                                                self.selection == Some(Selection::Live(id)),
                                                label,
                                            )
                                            .min_size(egui::vec2(ui.available_width(), 0.0)),
                                        )
                                        .clicked()
                                    {
                                        self.selection = Some(Selection::Live(id));
                                    }
                                }
                                if self.show_keyframes {
                                    ui.separator();
                                    ui.label("Keyframe baselines (separate ID domain)");
                                    for (&id, state) in &world.keyframes {
                                        if !matches_search(
                                            &id.raw().to_string(),
                                            id.raw(),
                                            state,
                                            &search,
                                        ) {
                                            continue;
                                        }
                                        let label = format!(
                                            "Keyframe {} ({}) +{}",
                                            id.raw(),
                                            schema_name_label(state.schema_kind, state.schema),
                                            state.components.len()
                                        );
                                        if ui
                                            .add(
                                                egui::Button::selectable(
                                                    self.selection == Some(Selection::Keyframe(id)),
                                                    label,
                                                )
                                                .min_size(egui::vec2(ui.available_width(), 0.0)),
                                            )
                                            .clicked()
                                        {
                                            self.selection = Some(Selection::Keyframe(id));
                                        }
                                    }
                                }
                            });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let found = match self.selection {
                Some(Selection::Live(id)) => world.entities.get(&id).map(|state| {
                    show_entity(
                        ui,
                        &format!(
                            "Entity {id} ({}) +{}",
                            schema_name_label(state.schema_kind, state.schema),
                            state.components.len()
                        ),
                        state,
                    );
                }),
                Some(Selection::Keyframe(id)) => world.keyframes.get(&id).map(|state| {
                    show_entity(
                        ui,
                        &format!(
                            "Keyframe {} ({}) +{}",
                            id.raw(),
                            schema_name_label(state.schema_kind, state.schema),
                            state.components.len()
                        ),
                        state,
                    );
                }),
                None => None,
            };
            if found.is_none() {
                ui.heading("Select a live entity");
                ui.label("Move the timeline or choose an entity on the left. An entity disappears from the live list after its recorded deletion.");
            }
        });
    }
}

fn format_elapsed(microseconds: u64) -> String {
    let seconds = microseconds / 1_000_000;
    let milliseconds = (microseconds % 1_000_000) / 1_000;
    format!("{:02}:{:02}.{milliseconds:03}", seconds / 60, seconds % 60)
}

fn show_entity<Id>(ui: &mut egui::Ui, title: &str, state: &EntityState<Id>) {
    ui.heading(title);
    let created = state
        .created_at_us
        .map_or("keyframe baseline".to_owned(), |time| {
            format!("created at {:.3} s", time as f64 / 1e6)
        });
    let schema_label = match state.schema_kind {
        EntitySchemaKind::Unknown => format!("Schema {}", state.schema.raw()),
        kind => format!("Schema {} · {kind:?}", state.schema.raw()),
    };
    ui.label(format!(
        "{schema_label} · {created} · last update {:.3} s",
        state.updated_at_us as f64 / 1e6
    ));
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if !state.default_fields.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Creation defaults ({})",
                    state.default_fields.len()
                ))
                .show(ui, |ui| show_fields(ui, &state.default_fields));
            }
            if !state.fields().is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Latest recorded entity fields ({})",
                    state.fields().len()
                ))
                .show(ui, |ui| show_fields(ui, state.fields()));
            }
            if state.field_observations.len() > 1 {
                egui::CollapsingHeader::new(format!(
                    "Entity field observations ({})",
                    state.field_observations.len()
                ))
                .show(ui, |ui| {
                    for observation in &state.field_observations {
                        egui::CollapsingHeader::new(format!(
                            "At {:.3} s",
                            observation.timestamp_us as f64 / 1e6
                        ))
                        .show(ui, |ui| show_fields(ui, &observation.fields));
                    }
                });
            }
            ui.heading("Components");
            ui.label("Latest recorded bodies; encoded deltas remain raw.");
            for (slot, component) in &state.components {
                let label = format!("{} · slot {}", component.name, slot.index());
                ui.push_id(slot.index(), |ui| {
                    egui::CollapsingHeader::new(label).show(ui, |ui| {
                        show_fields(ui, component.fields());
                        if component.observations.len() > 1 {
                            egui::CollapsingHeader::new(format!(
                                "Recorded bodies ({})",
                                component.observations.len()
                            ))
                            .show(ui, |ui| {
                                for (index, observation) in
                                    component.observations.iter().enumerate()
                                {
                                    egui::CollapsingHeader::new(format!(
                                        "{} · {:.3} s",
                                        index + 1,
                                        observation.timestamp_us as f64 / 1e6
                                    ))
                                    .show(ui, |ui| show_fields(ui, &observation.fields));
                                }
                            });
                        }
                    });
                });
            }
            if state.components.is_empty() {
                ui.label("No decoded components at this time.");
            }
        });
}

fn matches_search<Id>(label: &str, raw_id: u32, state: &EntityState<Id>, search: &str) -> bool {
    search.is_empty()
        || label.to_lowercase().contains(search)
        || raw_id.to_string().contains(search)
        || schema_name_label(state.schema_kind, state.schema)
            .to_lowercase()
            .contains(search)
        || format!("schema {}", state.schema.raw()).contains(search)
        || state
            .default_fields
            .iter()
            .chain(state.fields())
            .chain(
                state
                    .field_observations
                    .iter()
                    .flat_map(|observation| &observation.fields),
            )
            .any(|field| field.name.to_lowercase().contains(search))
        || state.components.values().any(|component| {
            component.name.to_lowercase().contains(search)
                || component
                    .observations
                    .iter()
                    .flat_map(|observation| &observation.fields)
                    .any(|field| field.name.to_lowercase().contains(search))
        })
}

fn schema_name_label(kind: EntitySchemaKind, id: EntitySchemaId) -> String {
    match kind {
        EntitySchemaKind::Unknown => format!("Unknown:{}", id.raw()),
        kind => format!("{kind:?}"),
    }
}

fn show_fields(ui: &mut egui::Ui, fields: &[ReplicationField]) {
    if fields.is_empty() {
        ui.label("No decoded fields.");
    }
    for (index, field) in fields.iter().enumerate() {
        ui.horizontal_wrapped(|ui| {
            ui.strong(field.name.as_ref());
            match &field.value {
                ReplicationValue::Empty => {
                    ui.monospace("empty");
                }
                ReplicationValue::Bits(bits) => {
                    if let Some(value) = bits.as_u64() {
                        ui.monospace(format!("{value} (0x{value:x}, {} bits)", bits.bit_len));
                    } else {
                        ui.monospace(format!("{} bits: {}", bits.bit_len, hex_bytes(&bits.bytes)));
                    }
                }
                ReplicationValue::Tag { id, kind, .. } => {
                    let name = match kind {
                        TagKind::Weapon(item) => item.name(),
                        TagKind::Vehicle(item) => item.name(),
                        TagKind::Equipment(item) => item.name(),
                        TagKind::Grenade(item) => item.name(),
                        TagKind::Projectile => "projectile",
                        TagKind::Crate => "crate",
                        TagKind::Machine => "machine",
                        TagKind::Scenery => "scenery",
                        TagKind::Control => "control",
                        TagKind::Unknown => "unknown",
                    };
                    ui.monospace(format!("tag 0x{:08x} ({name})", id.raw()));
                }
            }
        });
        if let ReplicationValue::Tag {
            reference: Some(reference),
            ..
        } = &field.value
        {
            show_tag_reference(ui, reference, index);
        }
    }
}

fn show_tag_reference(ui: &mut egui::Ui, reference: &TagReference, field_index: usize) {
    egui::CollapsingHeader::new("Catalogue reference")
        .id_salt(field_index)
        .show(ui, |ui| {
            ui.label("Game reference data for this tag; not observed film state.");
            match reference {
                TagReference::Firing(stats) => {
                    ui.label(format!(
                        "Firing: {:.3} rounds/s · burst {} · recovery {:.3} s",
                        stats.rounds_per_second, stats.burst, stats.recovery
                    ));
                    if let Some([loaded, carried_start, carried_max]) = stats.magazine {
                        ui.label(format!(
                            "Reference magazine: {loaded} loaded · {carried_start} carried at start · {carried_max} max"
                        ));
                    }
                    if let Some(projectile) = &stats.projectile {
                        ui.label("Fires projectile:");
                        match projectile {
                            TagLink::Known(projectile) => {
                                show_projectile_reference(ui, projectile);
                            }
                            TagLink::Unknown(id) => {
                                ui.label(format!("Unknown projectile tag 0x{:08x}", id.raw()));
                            }
                        }
                    }
                }
                TagReference::Projectile(projectile) => show_projectile_reference(ui, projectile),
                TagReference::Damage(effect) => show_damage_reference(ui, effect),
                TagReference::VehicleBounds([x_min, x_max, y_min, y_max, z_min, z_max]) => {
                    ui.label(format!(
                        "Reference vehicle model bounds: x [{x_min:.3}, {x_max:.3}] · y [{y_min:.3}, {y_max:.3}] · z [{z_min:.3}, {z_max:.3}]"
                    ));
                }
            }
        });
}

fn show_projectile_reference(ui: &mut egui::Ui, projectile: &ProjectileReference) {
    ui.label(format!(
        "Projectile 0x{:08x}: initial velocity {:.3} units/s · max range {:.3} units",
        projectile.tag.raw(),
        projectile.velocity,
        projectile.range
    ));
    if let Some(impact) = &projectile.impact {
        ui.label("Direct-hit effect:");
        match impact {
            TagLink::Known(effect) => show_damage_reference(ui, effect),
            TagLink::Unknown(id) => {
                ui.label(format!("Unknown damage-effect tag 0x{:08x}", id.raw()));
            }
        }
    }
}

fn show_damage_reference(ui: &mut egui::Ui, effect: &DamageReference) {
    ui.label(format!(
        "Damage effect 0x{:08x}: {:?}",
        effect.tag.raw(),
        effect.cause
    ));
    if let Some(amount) = effect.amount {
        ui.label(format!(
            "Reference damage: lower {:.3} · upper [{:.3}, {:.3}] · radius {:.3} units",
            amount.lower, amount.upper[0], amount.upper[1], amount.radius
        ));
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".local/theater/films/ranked-arena/02-oddball")
}

fn main() -> Result<(), Error> {
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_film_directory);
    if !directory.is_dir() {
        return Err(format!(
            "film directory {} does not exist; pass a directory saved by download_film after --",
            directory.display()
        )
        .into());
    }
    let chunks = load_chunks(&directory)?;
    let film = Film::parse(chunks)?;
    let source = directory.display().to_string();
    eframe::run_native(
        "Film World Inspector",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1250.0, 800.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(Inspector::new(film, source)))),
    )?;
    Ok(())
}
