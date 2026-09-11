use std::collections::BTreeMap;

use super::bits::{Bits, pattern};
use super::{appearance, combat, input, motion, packets, projectile, *};
use crate::clients::hi::models::FilmChunkData;

fn span(p: &FilmPacket, bit: usize, end_bit: usize) -> SourceSpan {
    SourceSpan {
        chunk: p.chunk_index,
        payload_byte: p.payload_offset,
        bit,
        end_bit,
    }
}
fn sample<T>(
    p: &FilmPacket,
    time_us: u64,
    life: u16,
    bit: usize,
    end: usize,
    value: T,
) -> Sample<T> {
    Sample {
        time_us,
        life,
        value,
        source: span(p, bit, end),
    }
}
fn reject(d: &mut DecodeDiagnostics, reason: &str) {
    *d.rejected.entry(reason.to_owned()).or_default() += 1;
}
fn checked(d: &mut DecodeDiagnostics, retain: bool, source: SourceSpan, kind: &str) {
    if retain {
        d.checked_regions.push(CheckedRegion {
            source,
            kind: kind.to_owned(),
        });
    }
}
fn live(lives: &BTreeMap<u16, Life>, serial: u16, player: Option<u8>, time: u64) -> Option<&Life> {
    let life = lives.get(&serial)?;
    if player.is_some_and(|p| p != life.player)
        || time < life.start_us
        || time >= life.end_us
        || life.death_us.is_some_and(|d| time >= d)
    {
        return None;
    }
    Some(life)
}
fn event<T>(
    e: Option<combat::Event<T>>,
    lives: &BTreeMap<u16, Life>,
    t: u64,
    p: &FilmPacket,
    o: usize,
    coverage: (&mut DecodeDiagnostics, bool),
    kind: &str,
) -> Option<(u8, Sample<T>)> {
    let (d, retain) = coverage;
    let e = e?;
    let Some(l) = live(lives, e.serial, e.player, t) else {
        reject(d, "event outside checked life/roster");
        return None;
    };
    let source = span(
        p,
        match kind {
            "melee" => o.saturating_sub(31),
            "weapon damage" => o.saturating_sub(108),
            _ => o,
        },
        e.end,
    );
    checked(d, retain, source, kind);
    Some((
        l.player,
        Sample {
            time_us: t,
            life: l.id,
            value: e.value,
            source,
        },
    ))
}
fn normalize<T: PartialEq>(rows: &mut Vec<Sample<T>>) -> usize {
    rows.sort_by_key(|s| (s.time_us, s.life));
    let before = rows.len();
    let mut written = 0;
    for read in 0..before {
        let row = &rows[read];
        let duplicate = rows[..written]
            .iter()
            .rev()
            .take_while(|s| s.time_us == row.time_us && s.life == row.life)
            .any(|s| s.value == row.value);
        if !duplicate {
            rows.swap(read, written);
            written += 1;
        }
    }
    rows.truncate(written);
    before - written
}

fn registry(chunks: &[FilmChunkData]) -> Result<FilmRegistry, DecodeError> {
    let registry = decode_registry(chunks).ok_or(DecodeError::Missing("bootstrap registry"))?;
    let pawn = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("pawn component registry"))?;
    for (i, name) in [
        (0, "object-position-dynamic-precision-component"),
        (
            1,
            "object-translational-velocity-dynamic-precision-component",
        ),
        (4, "object-body-vitality-component"),
        (5, "object-shield-vitality-component"),
        (21, "unit-desired-aiming-vector-component"),
        (25, "unit-command-tick-component"),
        (30, "weapon-state-ammo"),
        (31, "weapon-state-rounds-inventory"),
        (33, "weapon-state-ammo"),
        (34, "weapon-state-rounds-inventory"),
        (35, "weapon-state-overheated"),
        (42, "biped-desired-weapon-set"),
    ] {
        if pawn.components.get(i).map(String::as_str) != Some(name) {
            return Err(DecodeError::Inconsistent(format!(
                "unsupported pawn component {i}; expected {name}"
            )));
        }
    }
    Ok(registry)
}
impl Film {
    /// Decode supported version-41 observations from decompressed chunks.
    ///
    /// Chunk order is irrelevant. Framing/version/registry errors return `Err`;
    /// unsupported record forms stay unparsed. Samples use integer microseconds
    /// relative to the earliest nonzero packet timestamp. No filesystem, network,
    /// platform timers, or experiment exports are used.
    pub fn try_from_chunks(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
    ) -> Result<Self, DecodeError> {
        if options.major_version != 41 {
            return Err(DecodeError::UnsupportedVersion(options.major_version));
        }
        let packets = packets::index(chunks)?;
        let registry = registry(chunks)?;
        let origin = packets
            .iter()
            .filter_map(|p| (p.timestamp_us != 0).then_some(p.timestamp_us))
            .min()
            .ok_or(DecodeError::Missing("nonzero packet timestamp"))?;
        let duration = options.duration_us.unwrap_or_else(|| {
            packets
                .iter()
                .map(|p| p.timestamp_us.saturating_sub(origin))
                .max()
                .unwrap_or(0)
                + 1
        });
        let bytes: BTreeMap<_, _> = chunks
            .iter()
            .map(|c| (c.metadata.index, c.data.as_slice()))
            .collect();
        let roster = decode_players(chunks);
        let indices = decode_player_indices(chunks, &roster);
        let mut players: BTreeMap<u8, PlayerTrack> = BTreeMap::new();
        for p in &roster {
            if let Some(&id) = indices.get(&p.xuid)
                && players
                    .insert(
                        id,
                        PlayerTrack {
                            id,
                            xuid: Some(p.xuid.to_string()),
                            name: p.gamertag.clone(),
                            ..PlayerTrack::default()
                        },
                    )
                    .is_some()
            {
                return Err(DecodeError::Inconsistent("ambiguous roster index".into()));
            }
        }
        let summary_events: Vec<_> = decode_events(chunks, &roster, 41)
            .into_iter()
            .map(|e| SummaryEvent {
                xuid: e.xuid.to_string(),
                player: indices.get(&e.xuid).copied(),
                name: e.gamertag,
                time_us: u64::from(e.timestamp_ms) * 1000,
                kind: match e.kind {
                    FilmEventKind::Mode => SummaryKind::Mode,
                    FilmEventKind::Death => SummaryKind::Death,
                    FilmEventKind::Kill => SummaryKind::Kill,
                    FilmEventKind::Medal => SummaryKind::Medal,
                    FilmEventKind::Other(k) => SummaryKind::Other(k),
                },
                metadata: e.metadata,
                medal_flag: e.medal_flag,
            })
            .collect();
        let mut diagnostics = DecodeDiagnostics::default();
        for p in packets
            .iter()
            .filter(|p| p.packet_type == 8 && p.timestamp_us >= origin)
        {
            let b =
                Bits(&bytes[&p.chunk_index][p.payload_offset..p.payload_offset + p.payload_size]);
            for (o, w) in b.windows() {
                if w >> 48 != 0x2dc0 {
                    continue;
                }
                let Some((xuid, name, value)) = appearance::snapshot(b, o) else {
                    continue;
                };
                let Some(player) = indices.get(&xuid).and_then(|id| players.get_mut(id)) else {
                    continue;
                };
                if player.name != name {
                    continue;
                }
                let source = span(p, o - 488, o + 2638);
                player.appearance.push(AppearanceSample {
                    time_us: p.timestamp_us - origin,
                    value,
                    source,
                });
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    source,
                    "appearance snapshot",
                );
            }
        }
        for player in players.values_mut() {
            player.appearance.sort_by_key(|s| s.time_us);
            player
                .appearance
                .dedup_by(|a, b| a.time_us == b.time_us && a.value == b.value);
        }
        let mut lives = BTreeMap::<u16, Life>::new();
        let mut transitions = Vec::new();
        let mut last_signature = None;
        // First pass binds identities before accepting any per-player observations.
        for p in packets
            .iter()
            .filter(|p| p.packet_type == 0 && p.timestamp_us >= origin)
        {
            let b =
                Bits(&bytes[&p.chunk_index][p.payload_offset..p.payload_offset + p.payload_size]);
            let t = p.timestamp_us - origin;
            diagnostics.frames += 1;
            if let Some((signature, _)) = combat::clock(b, 0, true)
                && last_signature != Some(signature)
            {
                transitions.push(t);
                last_signature = Some(signature);
            }
            for (o, w) in b.windows() {
                if w >> (64 - motion::SPAWN_BODY.len()) != pattern(motion::SPAWN_BODY) {
                    continue;
                }
                let Some(start) = o.checked_sub(20) else {
                    continue;
                };
                let Some(s) = motion::spawn(b, start) else {
                    reject(&mut diagnostics, "unsupported spawn candidate");
                    continue;
                };
                let life = Life {
                    id: s.serial,
                    wire: (s.serial % 256) as u8,
                    generation: (s.serial / 256 + 1) as u8,
                    player: s.player,
                    start_us: t,
                    end_us: duration,
                    death_us: None,
                    round_reset: false,
                    position: s.xyz,
                    layout: s.layout,
                    source: span(p, start, s.end),
                };
                if let Some(prior) = lives.get(&s.serial) {
                    if prior.start_us == t && prior.player == s.player && prior.position == s.xyz {
                        reject(&mut diagnostics, "duplicate spawn");
                        continue;
                    }
                    return Err(DecodeError::Inconsistent(format!(
                        "reused life identity {}",
                        s.serial
                    )));
                }
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    life.source,
                    "spawn",
                );
                players.entry(s.player).or_insert_with(|| PlayerTrack {
                    id: s.player,
                    name: format!("Player {}", s.player),
                    ..PlayerTrack::default()
                });
                lives.insert(s.serial, life);
            }
        }
        for (&id, player) in &mut players {
            let mut list: Vec<_> = lives.values().filter(|l| l.player == id).cloned().collect();
            list.sort_by_key(|l| l.start_us);
            for i in 0..list.len() {
                if let Some(next) = list.get(i + 1) {
                    list[i].end_us = next.start_us;
                }
                let l = &mut list[i];
                let deaths: Vec<_> = summary_events
                    .iter()
                    .filter(|e| {
                        e.player == Some(id)
                            && e.kind == SummaryKind::Death
                            && e.time_us >= l.start_us
                            && e.time_us < l.end_us
                    })
                    .collect();
                if deaths.len() > 1 {
                    return Err(DecodeError::Inconsistent(format!(
                        "multiple deaths for life {}",
                        l.id
                    )));
                }
                l.death_us = deaths.first().map(|e| e.time_us);
                l.round_reset = l.death_us.is_none()
                    && l.end_us < duration
                    && transitions
                        .iter()
                        .skip(1)
                        .any(|t| *t <= l.end_us && l.end_us - *t < 250_000);
                lives.insert(l.id, l.clone());
                player.positions.push(Sample {
                    time_us: l.start_us,
                    life: l.id,
                    value: Position {
                        raw: l.position,
                        spawn: true,
                        input: None,
                    },
                    source: l.source,
                });
            }
            player.lives = list;
        }
        if lives.is_empty() {
            diagnostics.limitations.push(
                "No supported spawn bindings; player observations remain unavailable.".into(),
            );
        }
        let mut clocks = Vec::new();
        let mut projectiles = projectile::Tracks::default();
        let projectile_layout = lives
            .values()
            .next()
            .map(|l| l.layout)
            .filter(|layout| lives.values().all(|l| l.layout == *layout))
            .filter(|_| {
                registry.archetype(41).is_some_and(|a| {
                    [
                        (0, "object-position-component"),
                        (1, "object-translational-velocity-component"),
                        (2, "object-forward-and-up-component"),
                        (5, "object-shield-vitality-component"),
                        (18, "projectile-at-rest-state"),
                        (20, "projectile-command_tick"),
                    ]
                    .iter()
                    .all(|(i, name)| a.components.get(*i).map(String::as_str) == Some(*name))
                })
            });
        // All motion, vitality, combat, weapon, clock and projectile candidates
        // share a single rolling byte window in this second pass.
        for p in packets
            .iter()
            .filter(|p| p.packet_type == 0 && p.timestamp_us >= origin)
        {
            let b =
                Bits(&bytes[&p.chunk_index][p.payload_offset..p.payload_offset + p.payload_size]);
            let t = p.timestamp_us - origin;
            let frame_tick = combat::clock(b, 0, true).map(|(_, tick)| tick);
            let mut local_tick = None;
            let mut first_clock_end = None;
            let mut previous_delta_end = 0;
            let mut accepted_deltas = Vec::new();
            if let Some(l) = live(&lives, 0, Some(0), t)
                && let Some(c) = motion::input_chain(b, l.layout)
            {
                let player = players
                    .get_mut(&l.player)
                    .ok_or(DecodeError::Missing("player track"))?;
                if let Some(xyz) = c.position {
                    player.positions.push(sample(
                        p,
                        t,
                        l.id,
                        37,
                        c.end,
                        Position {
                            raw: xyz,
                            spawn: false,
                            input: c.input,
                        },
                    ));
                }
                if let Some(aim) = c.aim {
                    player.aim.push(sample(p, t, l.id, 37, c.end, aim));
                }
                if let Some((start, end, value)) = c.velocity {
                    player
                        .velocities
                        .push(sample(p, t, l.id, start, end, value));
                }
                if let Some(input) = c.input {
                    player
                        .inputs
                        .push(sample(p, t, l.id, c.end - 25, c.end, input));
                    if let Some((value, end)) = motion::crouch_input(b, c.end) {
                        player
                            .crouch_input
                            .push(sample(p, t, l.id, c.end, end, value));
                        checked(
                            &mut diagnostics,
                            options.retain_coverage,
                            span(p, c.end, end),
                            "crouch input tail",
                        );
                    }
                }
                // This chain already owns its pawn observations and auxiliaries.
                // Do not decode the same prefix again without its attached input.
                previous_delta_end = c.end;
                if c.position.is_some() || c.aim.is_some() || c.velocity.is_some() {
                    accepted_deltas.push(l.id);
                }
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    span(p, 0, c.end),
                    "input chain (includes opaque auxiliaries)",
                );
            }
            for (o, w) in b.windows() {
                // Broad dispatch is cheap; every candidate has independent bounded guards.
                let clock_word = w >> 35;
                if combat::CLOCKS
                    .iter()
                    .any(|c| clock_word & ((1 << 26) - 1) == c & ((1 << 26) - 1))
                    && let Some((signature, counter)) = combat::clock(b, o, false)
                {
                    local_tick = Some(counter);
                    // Only strict leading bits establish a global clock observation.
                    if combat::clock(b, o, true).is_some() {
                        first_clock_end.get_or_insert(o + 37);
                        let source = span(p, o, o + 37);
                        clocks.push(ClockSample {
                            time_us: t,
                            signature,
                            counter,
                            source,
                        });
                        checked(&mut diagnostics, options.retain_coverage, source, "clock");
                    }
                }
                if (w >> 52) & 0x7ff == pattern("10100100110")
                    && let Some((id, s)) = event(
                        combat::firing(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "firing",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .firing
                        .push(s);
                }
                if (w >> 53) & 0x3ff == pattern("1010101010")
                    && let Some((id, s)) = event(
                        combat::weapon_damage(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "weapon damage",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .damage
                        .push(s);
                }
                if (w >> 54) & 0x1ff == pattern("101010001")
                    && let Some((id, s)) = event(
                        combat::melee(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "melee",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .melee
                        .push(s);
                }
                if (w >> 54) & 0x1ff == pattern("101001111")
                    && let Some((id, s)) = event(
                        combat::grenade(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "grenade throw",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .grenades
                        .push(s);
                }
                if (w >> 54) & 0x1ff == pattern("101001101")
                    && let Some((id, s)) = event(
                        combat::reload(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "reload",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .reloads
                        .push(s);
                }
                if (w >> 53) & 0x3ff == pattern("1001010110")
                    && let Some((id, s)) = event(
                        combat::zoom(b, o),
                        &lives,
                        t,
                        p,
                        o,
                        (&mut diagnostics, options.retain_coverage),
                        "zoom",
                    )
                {
                    players
                        .get_mut(&id)
                        .ok_or(DecodeError::Missing("player track"))?
                        .zoom
                        .push(s);
                }
                if w >> 58 == pattern("100010") {
                    let Some((serial, _, _)) = motion::pawn_header(b, o) else {
                        continue;
                    };
                    let Some(l) = live(&lives, serial, None, t) else {
                        continue;
                    };
                    let d =
                        if o >= 37 && o >= previous_delta_end && !accepted_deltas.contains(&serial)
                        {
                            frame_tick.and_then(|tick| motion::clocked_delta(b, o, l.layout, tick))
                        } else {
                            None
                        }
                        .or_else(|| motion::isolated_vitality(b, o));
                    let player = players
                        .get_mut(&l.player)
                        .ok_or(DecodeError::Missing("player track"))?;
                    if let Some(d) = d {
                        previous_delta_end = d.end;
                        accepted_deltas.push(serial);
                        // Presence of component 42 establishes a weapon-set update,
                        // even when its value/continuation is not understood.
                        if d.ids.contains(&42) {
                            player.weapons.push(sample(
                                p,
                                t,
                                l.id,
                                o,
                                d.end,
                                WeaponObservation {
                                    slot: None,
                                    weapon_window: None,
                                },
                            ));
                        }
                        if let Some(raw) = d.position {
                            player.positions.push(sample(
                                p,
                                t,
                                l.id,
                                o,
                                d.end,
                                Position {
                                    raw,
                                    spawn: false,
                                    input: None,
                                },
                            ));
                        }
                        if let Some(aim) = d.aim {
                            player.aim.push(sample(p, t, l.id, o, d.end, aim));
                        }
                        if let Some((start, end, value)) = d.velocity {
                            player
                                .velocities
                                .push(sample(p, t, l.id, start, end, value));
                        }
                        if let Some(body) = d.body {
                            player.body.push(sample(p, t, l.id, o, d.end, body));
                        }
                        if let Some(shield) = d.shield {
                            player.shields.push(sample(p, t, l.id, o, d.end, shield));
                        }
                        checked(
                            &mut diagnostics,
                            options.retain_coverage,
                            span(p, o, d.end),
                            "pawn delta prefix",
                        );
                    }
                    if let Some(d) = motion::weapon_delta(b, o, l.layout, local_tick) {
                        // A clocked prefix may already own this exact velocity.
                        // normalize() removes that duplicate source observation.
                        if let Some((start, end, value)) = d.velocity {
                            player
                                .velocities
                                .push(sample(p, t, l.id, start, end, value));
                        }
                        for m in d.magazines {
                            player.magazines.push(sample(p, t, l.id, o, d.end, m));
                        }
                        if let Some(slot) = d.selection {
                            player.selections.push(sample(p, t, l.id, o, d.end, slot));
                        }
                        checked(
                            &mut diagnostics,
                            options.retain_coverage,
                            span(
                                p,
                                o,
                                d.end
                                    + if b.is(d.end, combat::INPUT_END) {
                                        combat::INPUT_END.len()
                                    } else {
                                        0
                                    },
                            ),
                            "weapon delta",
                        );
                    }
                }
                if let Some(layout) = projectile_layout {
                    if (w >> 58) & 15 == 2
                        && let Some((value, end)) = projectile::birth(b, o, layout)
                    {
                        projectiles
                            .births
                            .push(sample(p, t, value.life, o, end, value));
                    }
                    if matches!(w >> 56, 0x90..=0x93)
                        && let Some(value) = projectile::record(b, o, layout)
                    {
                        let end = value.end;
                        projectiles.records.push(sample(p, t, 0, o, end, value));
                    }
                }
            }
            // This suffix grammar is validated on one/two-player recordings.
            // Bind command roster IDs to the unique current life, not wire zero
            // or the pawn whose replication record happens to precede the input.
            if players.len() <= 2
                && let Some(rows) = input::terminal(b)
                && first_clock_end.is_some_and(|end| end <= rows[0].start.saturating_sub(3))
                && rows.iter().all(|r| players.contains_key(&r.player))
            {
                for r in rows {
                    let player = players
                        .get_mut(&r.player)
                        .ok_or(DecodeError::Missing("player track"))?;
                    let mut current = player
                        .lives
                        .iter()
                        .filter(|l| live(&lives, l.id, Some(r.player), t).is_some());
                    let Some(l) = current.next() else { continue };
                    let life = l.id;
                    if current.next().is_some() {
                        continue;
                    }
                    player
                        .inputs
                        .push(sample(p, t, life, r.start, r.buttons, r.axes));
                    player
                        .crouch_input
                        .push(sample(p, t, life, r.buttons, r.end, r.crouch));
                    checked(
                        &mut diagnostics,
                        options.retain_coverage,
                        span(p, r.start, r.end),
                        "terminal input command (opaque header tag)",
                    );
                }
            }
            if b.is(0, projectile::PROJECTILE_END) {
                projectiles
                    .terminals
                    .push((t, span(p, 0, projectile::PROJECTILE_END.len())));
            }
        }
        for player in players.values_mut() {
            normalize(&mut player.damage);
            normalize(&mut player.crouch_input);
            normalize(&mut player.positions);
            normalize(&mut player.velocities);
            normalize(&mut player.aim);
            normalize(&mut player.inputs);
            let duplicates = normalize(&mut player.firing);
            if duplicates > 0 {
                *diagnostics
                    .rejected
                    .entry("duplicate firing".into())
                    .or_default() += duplicates;
            }
            normalize(&mut player.melee);
            normalize(&mut player.grenades);
            normalize(&mut player.reloads);
            normalize(&mut player.magazines);
            normalize(&mut player.selections);
            normalize(&mut player.zoom);
            normalize(&mut player.body);
            normalize(&mut player.shields);
            for pair in player.firing.windows(2) {
                if pair[1].value.sequence != pair[0].value.sequence.wrapping_add(1) {
                    reject(&mut diagnostics, "firing sequence gap (not inferred shots)");
                }
            }
            let mut firing = BTreeMap::<(i32, usize, u16), Vec<&Sample<Firing>>>::new();
            let mut magazines = BTreeMap::<(i32, usize, u16), Vec<&Sample<Magazine>>>::new();
            for s in &player.firing {
                firing
                    .entry((s.source.chunk, s.source.payload_byte, s.life))
                    .or_default()
                    .push(s);
            }
            for s in &player.magazines {
                magazines
                    .entry((s.source.chunk, s.source.payload_byte, s.life))
                    .or_default()
                    .push(s);
            }
            for (key, shots) in firing {
                let magazine_slot = match (shots.as_slice(), magazines.get(&key).map(Vec::as_slice))
                {
                    ([_], Some([m])) => Some(m.value.slot),
                    _ => None,
                };
                for shot in shots {
                    let slot = match (shot.value.weapon_slot(), magazine_slot) {
                        (Some(a), Some(b)) if a != b => {
                            reject(&mut diagnostics, "conflicting firing/magazine slots");
                            None
                        }
                        (Some(slot), _) | (_, Some(slot)) => Some(slot),
                        _ => None,
                    };
                    player.weapons.push(Sample {
                        time_us: shot.time_us,
                        life: shot.life,
                        value: WeaponObservation {
                            slot,
                            weapon_window: Some(shot.value.weapon_window),
                        },
                        source: shot.source,
                    });
                }
            }
            for s in &player.selections {
                player.weapons.push(Sample {
                    time_us: s.time_us,
                    life: s.life,
                    value: WeaponObservation {
                        slot: Some(s.value),
                        weapon_window: None,
                    },
                    source: s.source,
                });
            }
            player
                .weapons
                .sort_by_key(|s| (s.time_us, s.life, s.source.bit));
            normalize(&mut player.weapons);
        }
        let players: Vec<_> = players.into_values().collect();
        let has_projectile_candidates =
            !projectiles.births.is_empty() || !projectiles.records.is_empty();
        let projectiles = projectiles.finish(&players);
        if has_projectile_candidates && projectiles.is_empty() {
            reject(
                &mut diagnostics,
                "projectile candidates outside spawn/identity/continuity guards",
            );
        }
        for projectile in &projectiles {
            for s in &projectile.positions {
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    s.source,
                    "projectile position (spawn includes opaque fields)",
                );
            }
            for s in &projectile.velocities {
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    s.source,
                    "projectile velocity",
                );
            }
            if let Some(source) = projectile.terminal {
                checked(
                    &mut diagnostics,
                    options.retain_coverage,
                    source,
                    "controlled projectile terminal",
                );
            }
        }
        diagnostics.limitations.extend([
            "Partial signature-guarded v41 grammar; unsupported records/components remain unparsed. Checked ranges can include opaque bits.",
            "Raw positions require independently supplied map bounds for world coordinates. Velocity is in world units/s; aim and vitality display scales remain provisional.",
            "No inferred initial aim, health, ammo or scope. Input axes/crouch commands support the checked wire-0 chain and terminal roster-0/1 records in one/two-player films. Other command forms, larger rosters, physical crouch and slide remain unsupported.",
            "Firing is activity, not an exact bullet count; reload cause/duration, reserves and full inventory are unknown.",
            "Projectile paths require checked spawns, thrower references and continuous supported updates. Grenade type and explosion locations remain unknown; track ends are observation boundaries.",
        ].map(str::to_owned));
        Ok(Film {
            schema_version: 1,
            major_version: 41,
            match_id: options.match_id,
            duration_us: duration,
            origin_timestamp_us: origin,
            registry,
            packets: if options.retain_coverage {
                packets
            } else {
                Vec::new()
            },
            players,
            summary_events,
            clocks,
            projectiles,
            diagnostics,
        })
    }
}
