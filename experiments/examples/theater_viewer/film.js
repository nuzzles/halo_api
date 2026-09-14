/* Presentation adapter for halo_api::theater::Film schema 1. No byte parsing. */
(function (root) {
  'use strict';
  const colors = ['#89ead0', '#f3a383', '#9fbbff', '#dfb0ee', '#e1d584', '#84c6d9', '#ed91b3', '#b8db8a'];
  const seconds = us => {
    if (!Number.isSafeInteger(us) || us < 0) throw new Error('Invalid film timestamp');
    return us / 1e6;
  };
  function filmToClip(film, metadata = {}) {
    if (film.schema_version !== 1 || film.major_version !== 41 || !Array.isArray(film.players) || !film.players.length)
      throw new Error('Expected a decoded version-41 Film (schema 1) with player tracks');
    if (metadata.match_id && metadata.match_id !== film.match_id) throw new Error('Film metadata belongs to another match');
    if (film.players.every(p => Array.isArray(p.positions) && p.positions.length === 0)) {
      const error = new Error('No decoded player positions are available for replay.');
      error.code = 'NO_DECODED_POSITIONS';
      throw error;
    }
    const duration = seconds(film.duration_us);
    const rows = (samples, value) => samples.map(s => [seconds(s.time_us), ...value(s.value, s.life)]);
    const players = film.players.map((p, i) => ({
      id: String(p.id), name: p.name, color: colors[i % colors.length],
      appearance: (p.appearance || []).map(s => [seconds(s.time_us), s.value, s.source]),
      lives: p.lives.map(l => ({ id: l.id, start: seconds(l.start_us), end: seconds(l.end_us), death: l.death_us == null ? null : seconds(l.death_us) })),
      samples: rows(p.positions, (v, life) => [...v.raw, v.input?.forward ?? null, v.input?.left ?? null, life]),
      // Compact presentation rows keep the hour-long replay practical to load.
      // Stationary has no invented direction/magnitude; raw codes stay exact.
      velocity: rows(p.velocities || [], (v, life) => v.form === 'Stationary' ? [life] :
        [life, ...v.direction, v.magnitude_code, v.direction_code]),
      aim: rows(p.aim, (v, life) => [v.yaw, v.pitch, life]),
      crouch: rows(p.crouch_input || [], (v, life) => {
        if (typeof v !== 'boolean') throw new Error('Invalid crouch input');
        return [life, v];
      }),
      firing: rows(p.firing, (v, life) => [life, v.sequence]),
      melee: rows(p.melee, (_, life) => [life]), grenade: rows(p.grenades, (_, life) => [life]),
      reload: rows(p.reloads, (_, life) => [life]), ammo: rows(p.magazines, (v, life) => [life, v.slot, v.rounds]),
      switch: rows(p.selections, (v, life) => [life, v]),
      weapon: rows(p.weapons, (v, life) => [life, v.slot, v.weapon_window == null ? null : v.weapon_window.toString(16).padStart(10, '0')]),
      zoom: rows(p.zoom, (v, life) => {
        const stage = { Unscoped: 0, First: 1, Second: 2 }[v];
        if (stage == null) throw new Error('Unknown scope stage');
        return [life, stage, 'recorded'];
      }),
      vitality: {
        body: rows(p.body, (v, life) => [v.raw, v.state.toString(2).padStart(3, '0'), life]),
        shield: rows(p.shields, (v, life) => [v.raw, v.delay_ticks, v.state.toString(2).padStart(4, '0'), life]),
      },
    }));
    const spawns = players.flatMap(p => p.lives.map(l => ({ time: l.start, kind: 'Spawn', player: p.name })));
    const events = film.summary_events.filter(e => ['Kill', 'Death'].includes(e.kind))
      .map(e => ({ time: seconds(e.time_us), kind: e.kind, player: e.name })).concat(spawns).sort((a, b) => a.time - b.time);
    const start = spawns.length ? Math.min(...spawns.map(s => s.time)) : 0;
    return {
      id: metadata.id || (metadata.group && metadata.slug ? `${metadata.group}/${metadata.slug}` : film.match_id || 'imported-film'),
      match_id: film.match_id,
      name: metadata.title?.trim() || (metadata.slug && metadata.slug.replace(/^\d+-/, '').replaceAll('-', ' ').replace(/^./, c => c.toUpperCase())) || (film.match_id ? `Film ${film.match_id.slice(0, 8)}` : 'Imported film'),
      description: metadata.description || '',
      group: metadata.group || 'Decoded films', category: metadata.category || 'Decoded films',
      duration, action: [start, Math.max(duration, start + .001)], players, events,
      selectedPlayer: players.find(p => p.name === 'Nuzzles')?.id || players[0].id,
      trailWindow: players.length > 2 ? 8 : undefined,
      projectiles: film.projectiles.map(p => ({ id: `${p.id}:${p.generation ?? 1}:${p.start_us}`, player: String(p.player), serial: p.life,
        start: seconds(p.start_us), end: seconds(p.end_us), samples: rows(p.positions, xyz => xyz),
        velocity: rows(p.velocities || [], v => v.form === 'Stationary' ? [] : [...v.direction, v.magnitude_code]),
        rest: rows(p.at_rest || [], v => [v]), terminal: p.terminal != null })),
      note: 'Decoded Film · raw coordinate codes. Missing aim, health, ammo and scope remain unknown. Projectile paths use recorded samples; type and explosions remain unknown.',
    };
  }
  if (typeof module !== 'undefined' && module.exports) module.exports = { filmToClip };
  else root.filmToClip = filmToClip;
})(typeof globalThis !== 'undefined' ? globalThis : this);
