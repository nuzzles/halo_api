/* Three.js is bundled locally under its MIT license. No network requests. */
(() => {
  'use strict';
  const $ = id => document.getElementById(id), host = $('scene');
  let renderer;
  try { renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true }); }
  catch (error) { $('webgl-error').hidden = false; console.error(error); return; }
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.25;
  host.appendChild(renderer.domElement);
  const labels = document.createElement('div'); labels.className = 'player-labels'; host.appendChild(labels);
  const scene = new THREE.Scene(), camera = new THREE.PerspectiveCamera(42, 1, .01, 10000);
  scene.add(new THREE.HemisphereLight(0xc1efff, 0x13343a, 2.6));
  const light = new THREE.DirectionalLight(0xe5fff7, 3); light.position.set(60, 100, 40); scene.add(light);
  const fill = new THREE.DirectionalLight(0x659fff, 2); fill.position.set(-60, 25, -40); scene.add(fill);
  let world = new THREE.Group(); scene.add(world);
  let clip, views = [], projectileViews = [], selected = 0, origin, timelineSamples = [], deathEvents = [];
  let time = 0, start = 0, end = 1, playing = false, fullMode = false, showTrail = true, showLook = true, showCards = true;
  let windowStart = 0, windowEnd = 1;
  let radius = 100, size = 100, theta = .8, phi = 1.0, markerSize = 3, drag = null;
  const target = new THREE.Vector3(), overviewTarget = new THREE.Vector3(), clock = new THREE.Clock(), MAX_GAP = .1, FIRING_PULSE = .15, MELEE_PULSE = .35, GRENADE_PULSE = .45, TAU = Math.PI * 2;
  const FORWARD = new THREE.Vector3(0, 0, 1), RELOAD_PULSE = .5;
  // Names are calibrated against weapon references in controls. The leading
  // firing bits encode the carried slot, independently of the 32-bit key.
  const WEAPON_NAMES = { '6acdc44d': 'Bandit EVO', 'f408190f': 'Pistol', '0a1992bc': 'S7 Sniper', '2b1824d5': 'BR75', '9387a8b9': 'Shock Rifle', '48c19d2d': 'Assault Rifle', 'daf193c7': 'Stalker Rifle' };
  const weaponFingerprint = window => typeof window === 'string' && /^[13][0-9a-f]{8}4$/.test(window) ? window.slice(1, 9) : null;
  let armorTrack = null, armorRow = null, armorState = null;
  function paintArmor(track) {
    const rows = track.appearance || [], row = rows[sampleIndex(time, rows)] || null;
    if (track === armorTrack && row === armorRow) return armorState;
    armorTrack = track; armorRow = row;
    $('armor-player').textContent = track.name;
    $('armor-panel').style.setProperty('--player-color', track.color);
    $('armor-status').textContent = row ? `Recorded at ${formatTime(row[0])}` : 'Appearance unknown';
    $('armor-empty').hidden = Boolean(row); $('armor-items').hidden = !row; $('armor-evidence').hidden = !row;
    $('armor-items').replaceChildren(); $('armor-identifiers').textContent = '';
    const items = row ? resolveArmor(row[1], ARMOR_CATALOG) : [];
    for (const item of items) {
      const cell = document.createElement('div'), label = document.createElement('dt'), value = document.createElement('dd');
      cell.dataset.slot = item.kind; cell.classList.toggle('unknown', !item.name);
      label.textContent = item.label; value.textContent = item.name || (item.available ? 'Unidentified' : 'Not decoded');
      if (item.paths.length) cell.title = 'Matched recorded identifiers to official item metadata:\n' + item.paths.join('\n');
      cell.append(label, value); $('armor-items').append(cell);
    }
    if (row) $('armor-identifiers').textContent = JSON.stringify({ sample_seconds: row[0], ...row[1], source: row[2] }, null, 2);
    armorState = { player: track.id, name: track.name, time: row?.[0] ?? null, items, value: row?.[1] ?? null };
    return armorState;
  }
  // Observed endpoints used only as provisional visual scales, not calibrated HP.
  const VITALITY_SCALES = { shield: 64, body: 126 };
  function createMeter(kind, compact = false) {
    const root = document.createElement('div'); root.className = `vitality-meter ${kind}${compact ? ' compact' : ''}`;
    const heading = document.createElement('div'); heading.className = 'meter-heading';
    const title = document.createElement('span'); title.textContent = compact ? (kind === 'shield' ? 'S' : 'H') : (kind === 'shield' ? 'Shields' : 'Health');
    const value = document.createElement('strong'); heading.append(title, value);
    const bar = document.createElement('div'); bar.className = 'meter-bar'; bar.setAttribute('role', 'progressbar');
    bar.setAttribute('aria-label', kind === 'shield' ? 'Shields, provisional raw scale' : 'Health, provisional raw scale');
    bar.setAttribute('aria-valuemin', '0'); bar.setAttribute('aria-valuemax', String(VITALITY_SCALES[kind]));
    const fill = document.createElement('i'); bar.append(fill);
    const detail = document.createElement('div'); detail.className = 'meter-detail'; detail.hidden = compact;
    root.append(heading, bar, detail);
    return { root, kind, compact, value, bar, fill, detail };
  }
  const readoutMeters = [createMeter('shield', true), createMeter('body', true)];
  $('vitality-meters').append(...readoutMeters.map(m => m.root));
  function paintMeter(meter, sample, dead, supported) {
    const known = sample !== null;
    meter.root.classList.toggle('unknown', !known); meter.root.classList.toggle('held', Boolean(sample?.stale));
    meter.root.classList.toggle('recovering', Boolean(sample?.recovering && !sample.stale));
    meter.value.textContent = known ? `${sample.raw}${meter.compact ? '' : ' / ' + VITALITY_SCALES[meter.kind]}` : (dead ? '—' : (meter.compact ? '?' : '—'));
    meter.fill.style.width = `${known ? sample.fraction * 100 : 0}%`;
    if (known) meter.bar.setAttribute('aria-valuenow', String(sample.raw)); else meter.bar.removeAttribute('aria-valuenow');
    let description = dead ? 'Eliminated · values hidden' : (supported ? 'Unknown · no sample this life' : 'No vitality export for this clip');
    if (known) {
      const state = meter.kind === 'shield' ? (sample.delayTicks > 0 ? ` · delay ${(sample.delayTicks / 60).toFixed(2)}s` : (sample.recovering ? ' · recovering' : (sample.raw === 0 ? ' · depleted' : ''))) : (sample.recovering ? ' · recovering' : '');
      description = `${sample.time.toFixed(3)}s${state} · ${sample.stale ? `last sample ${sample.age.toFixed(1)}s ago` : 'recorded sample'}`;
    }
    meter.detail.textContent = description;
    meter.bar.setAttribute('aria-valuetext', known ? `${sample.raw} raw, ${description}` : description);
    meter.root.title = `${meter.kind === 'shield' ? 'Shields' : 'Health'}: ${known ? sample.raw + ' raw. ' : ''}${description}`;
  }
  function createActivity() {
    const root = document.createElement('span'); root.className = 'combat-badges';
    const shooting = document.createElement('span'), melee = document.createElement('span'), grenade = document.createElement('span'), reload = document.createElement('span');
    shooting.className = 'combat-badge shooting'; melee.className = 'combat-badge melee';
    grenade.className = 'combat-badge grenade';
    reload.className = 'combat-badge reload';
    const crouch = document.createElement('span'); crouch.className = 'combat-badge crouch';
    root.append(shooting, melee, grenade, reload, crouch); return { root, shooting, melee, grenade, reload, crouch };
  }
  function paintActivity(badges, state) {
    badges.crouch.textContent = 'CROUCH INPUT' + (state.crouch.value == null ? ' ?' : state.crouch.value ? ' ●' : ' ·');
    badges.crouch.classList.toggle('active', state.crouch.value === true);
    badges.crouch.classList.toggle('unknown', state.crouch.value == null);
    badges.crouch.title = state.crouch.value == null ? 'No recent decoded crouch input' :
      `Recorded crouch input: ${state.crouch.value ? 'held' : 'released'} at ${state.crouch.time.toFixed(3)} s. Physical posture and slide state remain unknown.`;
    for (const [kind, key, title] of [['shooting', 'firing', 'SHOOT'], ['melee', 'melee', 'MELEE'], ['grenade', 'grenade', 'THROW'], ['reload', 'reload', 'LOAD']]) {
      const badge = badges[kind], supported = state[key + 'Supported'], active = state[key];
      badge.textContent = title + (!supported ? ' ?' : active ? ' ●' : ' ·');
      badge.classList.toggle('active', active); badge.classList.toggle('unknown', !supported);
      badge.title = state.dead ? 'Eliminated' : !supported ? 'No decoded ' + kind + ' stream' :
        active ? title + ' event at ' + state[key + 'Time'].toFixed(3) + ' s' :
        state[key + 'Time'] != null ? 'Last event: ' + state[key + 'Time'].toFixed(3) + ' s' : 'No decoded event yet';
    }
  }
  function weaponState(track, life, dead) {
    const state = { supported: Array.isArray(track.weapon), slot: null, name: null, window: null, time: null, ammo: null, source: null };
    if (dead || (track.lives && (!life || time >= life.end))) return state;
    const rows = track.weapon || [], index = sampleIndex(time, rows), row = rows[index];
    if (!row || (track.lives && row[1] !== life?.id)) return state;
    state.slot = row[2] ?? null; state.window = row[3]; state.name = WEAPON_NAMES[weaponFingerprint(row[3])] || null; state.time = row[0];
    state.source = row[3] == null ? 'weapon-set update; held weapon unknown' : 'recorded firing';
    // Selection invalidates held slot inventory. Pickups may replace a slot.
    const switches = track.switch || [], selection = switches[sampleIndex(time, switches)];
    const selectionTime = selection && selection[1] === row[1] ? selection[0] : (life?.start ?? 0);
    let since = row[0];
    // A firing packet can identify a slot without updating its ammo. Hold
    // the last recorded quantity for this same weapon and slot, stopping at every
    // weapon-set/identity change, selection or life boundary. Never count down
    // rounds from firing events; the magazine stream supplies every quantity.
    if (row[3] != null) {
      const key = weaponFingerprint(row[3]);
      for (let i = index - 1; i >= 0; i--) {
        const previous = rows[i];
        if (previous[1] !== row[1] || previous[0] < selectionTime ||
            (key ? weaponFingerprint(previous[3]) !== key : previous[3] !== row[3]) ||
            (state.slot != null && previous[2] != null && previous[2] !== state.slot)) break;
        since = previous[0];
        if (state.slot === null && previous[2] != null) state.slot = previous[2];
      }
    }
    since = Math.max(since, selectionTime);
    const ammo = (track.ammo || []).findLast(a => a[0] <= time && a[0] >= since && a[1] === row[1] && a[2] === state.slot);
    if (ammo) state.ammo = { time: ammo[0], raw: ammo[3], serial: ammo[1], age: time - ammo[0], stale: time - ammo[0] > MAX_GAP };
    return state;
  }
  function paintWeapon(element, state) {
    const w = state.weapon;
    element.textContent = state.dead ? 'Weapon — · Ammo —' : `${w.name || (w.slot === null ? 'Weapon ?' : 'Slot ' + (w.slot + 1))} · Ammo ${w.ammo?.raw ?? '?'}`;
    element.classList.toggle('held', Boolean(w.ammo?.stale));
    element.title = state.dead ? 'Eliminated · values hidden' : `${w.name || 'Weapon unknown'}${w.source ? ' · ' + w.source : ''}. ` +
      (w.ammo ? `Recorded magazine: ${w.ammo.raw} at ${w.ammo.time.toFixed(3)} s. Last observed ${w.ammo.age.toFixed(2)} s ago; reserve ammo unknown.` : 'No recorded magazine for the selected weapon in this life. Reserve ammo unknown.');
  }
  function zoomState(track, life, dead) {
    const state = { supported: Array.isArray(track.zoom), level: null, time: null, age: null, stale: false };
    if (dead || (track.lives && (!life || time >= life.end))) return state;
    const rows = track.zoom || [], row = rows[sampleIndex(time, rows)];
    if (!row || (track.lives && row[1] !== life?.id)) return state;
    const switches = track.switch || [], selection = switches[sampleIndex(time, switches)];
    if (selection && selection[1] === row[1] && selection[0] > row[0]) return state;
    const change = (track.weapon || []).findLast(w => w[0] <= time && w[1] === row[1] && w[3] == null);
    if (change && change[0] > row[0]) return state;
    return { ...state, level: row[2], time: row[0], age: time - row[0], stale: time - row[0] > MAX_GAP, source: row[3] || 'recorded' };
  }
  function crouchState(track, life, dead) {
    const rows = track.crouch || [], row = rows[sampleIndex(time, rows)];
    if (dead || !life || time >= life.end || !row || row[1] !== life.id || time - row[0] > MAX_GAP)
      return { value: null, time: null };
    return { value: row[2], time: row[0] };
  }
  function paintZoom(element, state) {
    const z = state.zoom;
    element.textContent = state.dead ? 'Scope —' : z.level === null ? 'Scope ?' : `Scope sample ${z.level}`;
    element.classList.toggle('active', z.level > 0); element.classList.toggle('held', z.stale);
    element.title = state.dead ? 'Eliminated · scope hidden' : z.level === null ? 'No scope observation for this life and weapon selection.' : `Last observed zoom stage ${z.level} at ${z.time.toFixed(3)} s (${z.age.toFixed(2)} s ago). Partial coverage; stage is not a magnification multiplier.`;
  }
  const formatTime = value => {
    const hundredths = Math.round(value * 100);
    return `${String(Math.floor(hundredths / 6000)).padStart(2, '0')}:${((hundredths % 6000) / 100).toFixed(2).padStart(5, '0')}`;
  };
  const local = row => new THREE.Vector3(row[1] - origin[0], row[3] - origin[2], -(row[2] - origin[1]));
  const material = (color, extra = {}) => new THREE.MeshStandardMaterial({ color, roughness: .55, metalness: .22, ...extra });
  function mesh(geometry, mat, parent, x = 0, y = 0, z = 0) {
    const result = new THREE.Mesh(geometry, mat); result.position.set(x, y, z); parent.add(result); return result;
  }
  function disposeWorld() {
    world.traverse(object => {
      object.geometry?.dispose();
      if (object.material) for (const m of [].concat(object.material)) { m.map?.dispose(); m.dispose(); }
    });
    scene.remove(world); world = new THREE.Group(); scene.add(world); labels.replaceChildren();
  }
  function resetCamera(top = false) {
    target.copy(overviewTarget);
    theta = .8; phi = top ? .015 : 1.02;
    radius = size * (top ? (views.length > 2 ? 1.7 : 2.1) : (views.length > 2 ? 1.35 : (views.length > 1 ? 2 : (views[0]?.track.samples.length ? 2.45 : 1.85))));
    updateCamera();
  }
  function updateCamera() {
    camera.position.set(target.x + radius * Math.sin(phi) * Math.cos(theta), target.y + radius * Math.cos(phi), target.z + radius * Math.sin(phi) * Math.sin(theta));
    camera.lookAt(target); camera.updateMatrixWorld();
  }
  function addAxis(direction, color) {
    world.add(new THREE.ArrowHelper(direction, new THREE.Vector3(0, .12, 0), markerSize * 5, color, markerSize * .8, markerSize * .35));
  }
  function createPlayer(track) {
    const samples = track.samples, points = samples.map(local), segments = [], vertices = [];
    for (let i = 1; i < samples.length; i++) if (samples[i][0] - samples[i - 1][0] <= MAX_GAP && samples[i][6] === samples[i - 1][6]) {
      vertices.push(points[i - 1].x, points[i - 1].y + .18, points[i - 1].z, points[i].x, points[i].y + .18, points[i].z);
      segments.push([samples[i][0]]);
    }
    const geometry = new THREE.BufferGeometry(); geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    const fullTrail = new THREE.LineSegments(geometry, new THREE.LineBasicMaterial({ color: track.color, transparent: true, opacity: .19 }));
    const trail = new THREE.LineSegments(geometry.clone(), new THREE.LineBasicMaterial({ color: track.color, transparent: true, opacity: .9 }));
    world.add(fullTrail, trail);
    const avatar = new THREE.Group(), body = new THREE.Group(); world.add(avatar); avatar.add(body);
    const shell = material(track.color), dark = material(0x173d48), visor = material(0xedc785, { emissive: 0x9d5c15, emissiveIntensity: .25 });
    mesh(new THREE.CapsuleGeometry(markerSize * .38, markerSize * .66, 5, 12), shell, body, 0, markerSize * 1.02, 0);
    const head = new THREE.Group(); head.position.y = markerSize * 1.94; body.add(head);
    mesh(new THREE.SphereGeometry(markerSize * .32, 16, 12), shell, head);
    mesh(new THREE.BoxGeometry(markerSize * .51, markerSize * .17, markerSize * .16), visor, head, 0, markerSize * .03, markerSize * .26);
    for (const sign of [-1, 1]) {
      mesh(new THREE.CapsuleGeometry(markerSize * .13, markerSize * .48, 4, 8), dark, body, sign * markerSize * .22, markerSize * .4, 0);
      mesh(new THREE.CapsuleGeometry(markerSize * .15, markerSize * .5, 4, 8), shell, body, sign * markerSize * .56, markerSize * 1.02, 0);
    }
    const gun = new THREE.Group(); gun.position.set(markerSize * .3, markerSize * 1.3, markerSize * .25); body.add(gun);
    mesh(new THREE.BoxGeometry(markerSize * .22, markerSize * .24, markerSize * .95), dark, gun, 0, 0, markerSize * .4);
    mesh(new THREE.BoxGeometry(markerSize * .12, markerSize * .12, markerSize * .4), visor, gun, 0, 0, markerSize);
    const muzzle = new THREE.Group(); muzzle.position.z = markerSize * 1.3; gun.add(muzzle);
    const flame = mesh(new THREE.OctahedronGeometry(markerSize * .34), new THREE.MeshBasicMaterial({ color: 0xffab48, fog: false }), muzzle);
    flame.scale.set(.65, .65, 1.8);
    mesh(new THREE.SphereGeometry(markerSize * .13, 8, 6), new THREE.MeshBasicMaterial({ color: 0xfff2c5, fog: false }), muzzle);
    muzzle.visible = false;
    const look = new THREE.Group(); look.renderOrder = 10; look.position.y = markerSize * 1.94; avatar.add(look);
    const lookLength = size * (clip.players.length > 2 ? .11 : .46);
    const gold = new THREE.MeshBasicMaterial({ color: 0xffd38a, transparent: true, opacity: .95, depthWrite: false, fog: false });
    mesh(new THREE.CylinderGeometry(markerSize * .045, markerSize * .045, lookLength, 8), gold, look, 0, 0, lookLength / 2).rotation.x = Math.PI / 2;
    mesh(new THREE.ConeGeometry(markerSize * .22, markerSize * .65, 16), gold, look, 0, 0, lookLength).rotation.x = Math.PI / 2;
    const ring = mesh(new THREE.RingGeometry(markerSize * .44, markerSize * .5, 32), new THREE.MeshBasicMaterial({ color: 0xffd38a, transparent: true, opacity: .95, side: THREE.DoubleSide, depthWrite: false, fog: false }), look, 0, 0, lookLength + markerSize * .6);
    const halo = mesh(new THREE.RingGeometry(markerSize * .92, markerSize * 1.02, 48), new THREE.MeshBasicMaterial({ color: track.color, transparent: true, opacity: .75, side: THREE.DoubleSide }), avatar, 0, .12, 0); halo.rotation.x = -Math.PI / 2;
    const shadow = mesh(new THREE.CircleGeometry(markerSize * .8, 32), new THREE.MeshBasicMaterial({ color: 0x07161d, transparent: true, opacity: .65, depthWrite: false }), world); shadow.rotation.x = -Math.PI / 2;
    const dropLine = new THREE.Line(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), new THREE.Vector3()]), new THREE.LineDashedMaterial({ color: track.color, dashSize: markerSize * .4, gapSize: markerSize * .25, transparent: true, opacity: .5 })); world.add(dropLine);
    const deathMark = new THREE.Group(); avatar.add(deathMark);
    const red = new THREE.MeshBasicMaterial({ color: 0xff867f });
    for (const angle of [-Math.PI / 4, Math.PI / 4]) mesh(new THREE.BoxGeometry(markerSize * 1.4, markerSize * .13, markerSize * .13), red, deathMark, 0, markerSize * .1, 0).rotation.y = angle;
    const label = document.createElement('button'); label.type = 'button'; label.className = 'player-name'; label.style.color = track.color; label.style.setProperty('--player-color', track.color);
    label.setAttribute('aria-label', `Select ${track.name}`);
    label.onclick = () => selectPlayer(views.findIndex(v => v.track.id === track.id)); labels.append(label);
    const labelName = document.createElement('span'); labelName.className = 'player-name-text'; label.append(labelName);
    const activity = createActivity(); label.append(activity.root);
    const weaponLabel = document.createElement('span'); weaponLabel.className = 'weapon-summary'; label.append(weaponLabel);
    const zoomLabel = document.createElement('span'); zoomLabel.className = 'zoom-summary'; label.append(zoomLabel);
    const meleeRing = mesh(new THREE.RingGeometry(markerSize * 1.1, markerSize * 1.28, 32, 1, 0, Math.PI * 1.5), new THREE.MeshBasicMaterial({ color: 0xc9a2ff, side: THREE.DoubleSide, transparent: true, opacity: .9, depthWrite: false }), avatar, 0, .2, 0);
    meleeRing.rotation.x = -Math.PI / 2; meleeRing.visible = false;
    const meters = [createMeter('shield', true), createMeter('body', true)], meterGroup = document.createElement('span'); meterGroup.className = 'player-vitality';
    meterGroup.append(...meters.map(m => m.root)); label.append(meterGroup);
    const vitalityChanges = [...new Set(Object.values(track.vitality || {}).flatMap(rows => rows.filter((r, i) => !i || r[1] !== rows[i - 1][1] || r.at(-1) !== rows[i - 1].at(-1)).map(r => r[0])))].sort((a, b) => a - b).map(t => [t]);
    for (const m of [shell, dark, visor]) m.transparent = true;
    return { track, segments, trail, fullTrail, avatar, body, head, gun, muzzle, bodyMaterials: [shell, dark, visor], look, lookMaterials: [gold, ring.material], halo, shadow, dropLine, deathMark, label, labelName, activity, weaponLabel, zoomLabel, meleeRing, meters, meterGroup, vitalityChanges, state: null };
  }
  function createProjectile(track) {
    const ball = mesh(new THREE.SphereGeometry(markerSize * .4, 14, 10), material('#77dfed', { emissive: '#278393', emissiveIntensity: .8 }), world);
    const band = mesh(new THREE.TorusGeometry(markerSize * .44, markerSize * .055, 5, 20), new THREE.MeshBasicMaterial({ color: '#dbffff' }), ball);
    band.rotation.x = Math.PI / 2;
    const vertices = [], segments = [];
    for (let i = 1; i < track.samples.length; i++) if (track.samples[i][0] - track.samples[i - 1][0] <= MAX_GAP) {
      for (const row of [track.samples[i - 1], track.samples[i]]) vertices.push(...local(row).toArray());
      segments.push([track.samples[i][0]]);
    }
    const geometry = new THREE.BufferGeometry(); geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    const trail = new THREE.LineSegments(geometry, new THREE.LineBasicMaterial({ color: '#77dfed', transparent: true, opacity: .8 })); world.add(trail);
    const label = document.createElement('span'); label.className = 'projectile-name'; label.textContent = 'GRENADE'; labels.append(label);
    return { track, ball, trail, segments, label };
  }
  function updateProjectile(v) {
    const {track, ball, trail, segments} = v, index = sampleIndex(time, track.samples), row = track.samples[index], next = track.samples[index + 1];
    const visible = time >= track.start && time < track.end && index >= 0;
    ball.visible = visible; trail.visible = showTrail && time >= track.start && time < track.end + 1;
    trail.geometry.setDrawRange(0, (sampleIndex(time, segments) + 1) * 2);
    if (visible) {
      ball.position.copy(local(row));
      if (next && next[0] - row[0] <= MAX_GAP) ball.position.lerp(local(next), (time - row[0]) / (next[0] - row[0]));
    }
    return {id: track.id, player: track.player, visible, sample: visible ? index : null,
      position: visible ? [ball.position.x, -ball.position.z, ball.position.y] : null,
      stale: visible && time - row[0] > MAX_GAP, trailVisible: trail.visible};
  }
  function selectPlayer(index) {
    if (index < 0 || index >= views.length) return;
    selected = index; drawEvents(); drawCoverage(); updateTime(); updateLabels();
  }
  function loadClip(id) {
    clip = CLIPS.find(c => c.id === id) || CLIPS[0]; playing = false; selected = Math.max(0, clip.players.findIndex(p => p.id === clip.selectedPlayer)); $('clip').value = clip.id;
    const tracks = clip.players, allPositions = tracks.flatMap(p => p.samples);
    const events = clip.events || [], deaths = events.filter(e => e.kind === 'Death'), kills = events.filter(e => e.kind === 'Kill');
    deathEvents = deaths.map(death => {
      // Pair only an unambiguous 1:1 timestamp neighborhood; this film has 1 ms
      // rounding differences between corresponding kill and death summaries.
      const candidates = kills.filter(k => Math.abs(k.time - death.time) <= .0011);
      const kill = candidates.length === 1 && deaths.filter(d => Math.abs(d.time - candidates[0].time) <= .0011).length === 1 ? candidates[0] : null;
      return { ...death, killer: kill?.player };
    });
    timelineSamples = [...new Set(tracks.flatMap(p => [...p.samples, ...p.aim, ...(p.firing || []), ...(p.melee || []), ...(p.reload || []), ...(p.ammo || []), ...(p.switch || []), ...(p.zoom || []), ...Object.values(p.vitality || {}).flat()].map(r => r[0])).concat((clip.events || []).map(e => e.time), (clip.projectiles || []).flatMap(p => [...p.samples.map(r => r[0]), p.end]), tracks.flatMap(p => (p.grenade || []).map(r => r[0]))))].sort((a, b) => a - b).map(t => [t]);
    origin = allPositions.length ? [allPositions[0][1], allPositions[0][2], allPositions.reduce((low, s) => Math.min(low, s[3]), Infinity)] : [0, 0, 0];
    disposeWorld();
    const scenePositions = allPositions.concat((clip.projectiles || []).flatMap(p => p.samples));
    const points = scenePositions.length ? scenePositions.map(local) : [new THREE.Vector3()];
    const bounds = new THREE.Box3().setFromPoints(points), span = bounds.getSize(new THREE.Vector3());
    size = Math.max(span.x, span.y, span.z, 25) * 1.15; markerSize = size / (tracks.length > 2 ? 90 : (allPositions.length ? 34 : 20));
    bounds.getCenter(target); target.y = span.y > size * .3 ? span.y * .48 : (tracks.some(p => p.aim.length || p.initialAim) ? size * .12 : markerSize * 1.4);
    overviewTarget.copy(target);
    camera.near = .01; camera.far = size * 25; camera.updateProjectionMatrix(); scene.fog = new THREE.Fog(0x10232e, size * 2.5, size * 7);
    const gridStep = 10 ** Math.round(Math.log10(size / 12)), gridSize = Math.ceil(size * 3 / gridStep) * gridStep;
    const grid = new THREE.GridHelper(gridSize, Math.round(gridSize / gridStep), 0x3c626e, 0x24414d); grid.position.set(target.x, -.12, target.z); grid.material.transparent = true; grid.material.opacity = .48; world.add(grid);
    mesh(new THREE.PlaneGeometry(gridSize, gridSize), material(0x102732, { transparent: true, opacity: .68, metalness: .05 }), world, target.x, -.17, target.z).rotation.x = -Math.PI / 2;
    addAxis(new THREE.Vector3(1, 0, 0), 0xf6a183); addAxis(new THREE.Vector3(0, 0, -1), 0x9fbbff); addAxis(new THREE.Vector3(0, 1, 0), 0x89ead0);
    views = tracks.map(createPlayer);
    projectileViews = (clip.projectiles || []).map(createProjectile);
    $('roster').replaceChildren(...views.map((v, i) => {
      const button = document.createElement('button'); button.style.setProperty('--player-color', v.track.color);
      v.rosterName = document.createElement('span'); v.rosterName.className = 'roster-name'; v.rosterName.textContent = v.track.name;
      v.rosterActivity = createActivity(); v.rosterMeters = [createMeter('shield', true), createMeter('body', true)];
      v.rosterWeapon = document.createElement('span'); v.rosterWeapon.className = 'weapon-summary';
      v.rosterZoom = document.createElement('span'); v.rosterZoom.className = 'zoom-summary';
      const meters = document.createElement('span'); meters.className = 'roster-vitality'; meters.append(...v.rosterMeters.map(m => m.root));
      button.append(v.rosterName, v.rosterActivity.root, v.rosterWeapon, v.rosterZoom, meters);
      button.onclick = () => selectPlayer(i); return button;
    }));
    $('roster').hidden = false;
    $('clip-info').textContent = `${views.length > 1 ? views.length + ' players · ' : ''}${allPositions.length.toLocaleString()} positions · ${tracks.reduce((n, p) => n + p.aim.length, 0).toLocaleString()} aim samples`;
    if (projectileViews.length) $('clip-info').textContent += ` · ${projectileViews.reduce((n, p) => n + p.track.samples.length, 0)} grenade samples`;
    $('scene-note').textContent = clip.note || 'Schematic player and floor. World units are not calibrated.';
    $('aim-note').hidden = !tracks.some(p => p.aim.length || p.initialAim);
    setRange(false); resetCamera(); updateTime(); updateLabels();
  }
  function sampleIndex(t, rows) {
    let lo = 0, hi = rows.length;
    while (lo < hi) { const mid = (lo + hi) >> 1; if (rows[mid][0] <= t) lo = mid + 1; else hi = mid; }
    return lo - 1;
  }
  function setRange(full) {
    fullMode = full; [start, end] = full ? [0, clip.duration] : clip.action; time = start;
    windowStart = start; windowEnd = end;
    for (const [id, on] of [['action-mode', !full], ['full-mode', full]]) { $(id).classList.toggle('selected', on); $(id).setAttribute('aria-pressed', String(on)); }
    drawTimelineWindow(); updateTime();
  }
  // Display bounds are independent of playback/loop bounds and recorded times.
  function drawTimelineWindow() {
    const width = windowEnd - windowStart, duration = end - start;
    $('timeline').min = windowStart; $('timeline').max = windowEnd;
    $('timeline').value = time;
    $('timeline-detail').classList.toggle('playhead-outside', time < windowStart || time > windowEnd);
    $('ticks').replaceChildren(...Array.from({ length: 6 }, (_, i) => {
      const label = document.createElement('span'); label.textContent = formatTime(windowStart + width * i / 5); return label;
    }));
    $('timeline-window-label').textContent = `${formatTime(windowStart)} – ${formatTime(windowEnd)}`;
    $('timeline-span').value = Math.abs(width - duration) < .001 ? 'all' : [300, 60, 10, 1].find(v => Math.abs(v - width) < .001)?.toString() || 'custom';
    $('timeline-zoom-in').disabled = width <= Math.min(1, duration) + .0001;
    $('timeline-zoom-out').disabled = $('timeline-fit').disabled = width >= duration - .0001;
    $('timeline-pan-control').hidden = width >= duration - .0001;
    $('timeline-pan').min = start; $('timeline-pan').max = Math.max(start, end - width);
    $('timeline-pan').value = windowStart;
    $('timeline-pan').setAttribute('aria-valuetext', `${formatTime(windowStart)} to ${formatTime(windowEnd)}`);
    $('timeline-pan-total').textContent = formatTime(end);
    drawEvents(); drawCoverage();
  }
  function setTimelineWindow(left, width) {
    width = Math.max(Math.min(1, end - start), Math.min(end - start, width));
    windowStart = Math.max(start, Math.min(end - width, left)); windowEnd = windowStart + width;
    drawTimelineWindow();
  }
  function zoomTimeline(width, anchor = time, fraction = .5) {
    width = Math.max(Math.min(1, end - start), Math.min(end - start, width));
    setTimelineWindow(anchor - width * fraction, width); updateTime(false);
  }
  function panTimeline(left) {
    playing = false; setTimelineWindow(left, windowEnd - windowStart); updateTime(false);
  }
  function revealPlayhead() {
    if (time < windowStart || time > windowEnd) {
      const width = windowEnd - windowStart;
      setTimelineWindow(time - width * (playing ? .2 : .5), width);
    }
  }
  function drawEvents() {
    $('events').classList.toggle('dense', views.length > 2);
    const events = (clip.events || []).filter(e => ['Death', 'Spawn'].includes(e.kind) && e.time >= start && e.time <= end);
    const markers = events.filter(e => e.time >= windowStart && e.time <= windowEnd && (views.length <= 2 || e.player === views[selected].track.name));
    $('events').replaceChildren(...markers.map(e => {
      const button = document.createElement('button'); button.className = 'event-marker' + (e.kind === 'Spawn' ? ' spawn' : ''); button.style.left = `${(e.time - windowStart) / (windowEnd - windowStart) * 100}%`;
      button.title = `${e.player} ${e.kind.toLowerCase()} · ${e.time.toFixed(3)} s`; button.setAttribute('aria-label', button.title); button.textContent = e.kind === 'Spawn' ? '+' : '×';
      button.onclick = () => { playing = false; time = e.time; updateTime(); }; return button;
    }));
    const placeholder = document.createElement('option'); placeholder.value = ''; placeholder.textContent = 'Choose a death or spawn…';
    $('event-select').replaceChildren(placeholder, ...events.map(e => {
      const option = document.createElement('option'); option.value = e.time; option.textContent = `${formatTime(e.time)} · ${e.player} · ${e.kind.toLowerCase()}`; return option;
    }));
    $('event-picker').hidden = !events.length;
    $('event-hint').textContent = views.length > 2 ? `Timeline: ${views[selected].track.name} · × death · + spawn` : '× death';
  }
  function drawCoverage() {
    const c = $('coverage'), width = c.clientWidth, height = c.clientHeight, dpr = devicePixelRatio;
    c.width = Math.round(width * dpr); c.height = Math.round(height * dpr);
    const ctx = c.getContext('2d'); ctx.scale(dpr, dpr);
    for (const v of (views.length > 2 ? [views[selected]] : views)) for (const [rows, color, top] of [[v.track.samples, '#457d7b', 0], [v.track.aim, '#bc965a', height / 2 + 1]]) {
      ctx.fillStyle = color;
      for (let i = Math.max(0, sampleIndex(windowStart, rows)); i < rows.length && rows[i][0] <= windowEnd; i++) {
        const a = rows[i][0], next = rows[i + 1]?.[0], b = next && next - a <= MAX_GAP ? next : a + .016;
        if (b < windowStart || a > windowEnd) continue;
        const x = (Math.max(a, windowStart) - windowStart) / (windowEnd - windowStart) * width, x2 = (Math.min(b, windowEnd) - windowStart) / (windowEnd - windowStart) * width;
        ctx.fillRect(x, top, Math.max(1, x2 - x), height / 2 - 1);
      }
    }
    for (const [kind, color, top] of [['firing', '#ff9c4f', height - 4], ['melee', '#c9a2ff', 0], ['grenade', '#77dfed', height / 2 - 2], ['reload', '#b9dc82', 4], ['zoom', '#f2d17c', height / 2 + 3]]) {
      ctx.fillStyle = color;
      const rows = views[selected].track[kind] || [];
      for (let i = Math.max(0, sampleIndex(windowStart, rows)); i < rows.length && rows[i][0] <= windowEnd; i++) {
        if (rows[i][0] >= windowStart) ctx.fillRect((rows[i][0] - windowStart) / (windowEnd - windowStart) * width, top, 2, 4);
      }
    }
  }
  function updatePlayer(v) {
    const { track, avatar, body, head, gun, muzzle, bodyMaterials, look, lookMaterials, halo, shadow, dropLine, deathMark, trail, fullTrail, segments } = v;
    const samples = track.samples, aims = track.aim;
    const life = track.lives?.findLast(l => l.start <= time);
    const death = life ? life.death : track.deathTime;
    const dead = death != null && time >= death;
    const sampleTime = dead ? death : time;
    let index = sampleIndex(sampleTime, samples), aimIndex = sampleIndex(sampleTime, aims);
    // Every observation is tied to its spawn serial. In particular, aim is
    // unknown at respawn until this new life has its own observation.
    if (track.lives) {
      if (!life || samples[index]?.[6] !== life.id) index = -1;
      if (!life || aims[aimIndex]?.[3] !== life.id) aimIndex = -1;
    }
    const current = new THREE.Vector3();
    let positionStatus = track.lives && !life ? 'Not spawned yet' : 'No decoded position yet', positionAge = null;
    if (index >= 0) {
      const row = samples[index], next = samples[index + 1], age = sampleTime - row[0]; current.copy(local(row)); positionAge = time - row[0];
      if (!dead && next && next[6] === row[6] && next[0] - row[0] <= MAX_GAP && next[0] > row[0] && (death == null || next[0] <= death)) current.lerp(local(next), age / (next[0] - row[0]));
      positionStatus = dead ? 'Last observed position · death recorded' : (track.stationary ? 'Decoded spawn · stationary recording' : (age > MAX_GAP ? `Last observed ${age.toFixed(1)}s ago` : `Recorded sample · ${index + 1} / ${samples.length}`));
      halo.material.opacity = age > MAX_GAP && !track.stationary ? .25 : .75;
    } else { halo.material.opacity = .35; if (aimIndex >= 0) positionStatus = 'Position unavailable · schematic pivot'; }
    const stale = index >= 0 && !dead && !track.stationary && positionAge > MAX_GAP;
    for (const m of bodyMaterials) m.opacity = stale && track.lives ? .28 : 1;
    avatar.visible = shadow.visible = index >= 0 || aimIndex >= 0;
    avatar.position.copy(current); shadow.position.set(current.x, .02, current.z); shadow.material.opacity = .65 / (1 + current.y / (markerSize * 5));
    const verts = dropLine.geometry.attributes.position; verts.setXYZ(0, current.x, .05, current.z); verts.setXYZ(1, current.x, current.y, current.z); verts.needsUpdate = true;
    dropLine.computeLineDistances(); dropLine.visible = index >= 0 && current.y > markerSize && !dead;
    let aim = null, aimStatus = aims.length ? 'No decoded aim yet' : 'No decoded aim in this clip';
    const initial = aimIndex < 0 && index >= 0 && track.initialAim;
    if (aimIndex >= 0 || initial) {
      const row = aimIndex >= 0 ? aims[aimIndex] : [samples[index][0], ...track.initialAim];
      const next = aims[aimIndex + 1], age = time - row[0]; let yawRaw = row[1], pitchRaw = row[2];
      if (!dead && !initial && next && next[3] === row[3] && next[0] - row[0] <= MAX_GAP && next[0] > row[0] && (death == null || next[0] <= death)) {
        const alpha = age / (next[0] - row[0]), delta = ((next[1] - row[1] + 2048) % 4096 + 4096) % 4096 - 2048;
        yawRaw = (yawRaw + alpha * delta + 4096) % 4096; pitchRaw += alpha * (next[2] - row[2]);
      }
      // Provisional display model; initial facing, when present, is explicitly reported.
      const yaw = yawRaw * TAU / 4096, pitch = (pitchRaw - 1024) * TAU / 2048;
      const direction = new THREE.Vector3(Math.cos(yaw) * Math.cos(pitch), Math.sin(pitch), -Math.sin(yaw) * Math.cos(pitch));
      body.rotation.y = yaw + Math.PI / 2; head.rotation.x = gun.rotation.x = -pitch; look.quaternion.setFromUnitVectors(FORWARD, direction);
      for (const m of lookMaterials) m.opacity = initial ? .65 : (age > MAX_GAP || stale ? .25 : .95);
      aimStatus = initial ? 'Initial facing · reported' : (age > MAX_GAP ? `Last aim ${age.toFixed(1)}s ago` : `Aim sample · ${aimIndex + 1} / ${aims.length}`);
      aim = { yawRaw, pitchRaw, age, source: initial ? 'reported' : 'decoded', direction: [direction.x, -direction.z, direction.y] };
    } else body.rotation.y = head.rotation.x = gun.rotation.x = 0;
    body.visible = !dead; deathMark.visible = dead; halo.material.color.set(dead ? '#ff867f' : track.color);
    look.visible = Boolean(aim) && showLook && !dead; gun.visible = Boolean(aim);
    if (dead) aimStatus = 'Death recorded · aim hidden';
    const firingRows = track.firing || [], firingIndex = sampleIndex(time, firingRows), firingRow = firingRows[firingIndex];
    const firingAge = firingRow ? time - firingRow[0] : null;
    const firing = !dead && Boolean(firingRow) && firingAge < FIRING_PULSE && (!track.lives || firingRow[1] === life?.id);
    // A directionless badge still works when aim is missing. A muzzle flash
    // requires a current aim (or explicitly reported stationary facing).
    muzzle.visible = firing && avatar.visible && Boolean(aim) && (aim.source === 'reported' || aim.age <= MAX_GAP) && !stale;
    muzzle.scale.setScalar(firing ? 1 - firingAge / FIRING_PULSE * .45 : 1);
    if (firing) { halo.material.color.set('#ff9c4f'); halo.material.opacity = 1; }
    const meleeRows = track.melee || [], meleeIndex = sampleIndex(time, meleeRows), meleeRow = meleeRows[meleeIndex];
    const meleeAge = meleeRow ? time - meleeRow[0] : null;
    const melee = !dead && Boolean(meleeRow) && meleeAge < MELEE_PULSE && (!track.lives || meleeRow[1] === life?.id);
    v.meleeRing.visible = melee && avatar.visible;
    if (melee) { v.meleeRing.scale.setScalar(1 + meleeAge / MELEE_PULSE * .65); v.meleeRing.material.opacity = 1 - meleeAge / MELEE_PULSE * .7; }
    const grenadeRows = track.grenade || [], grenadeIndex = sampleIndex(time, grenadeRows), grenadeRow = grenadeRows[grenadeIndex];
    const grenade = !dead && Boolean(grenadeRow) && time - grenadeRow[0] < GRENADE_PULSE && (!track.lives || grenadeRow[1] === life?.id);
    const reloadRows = track.reload || [], reloadRow = reloadRows[sampleIndex(time, reloadRows)];
    const reload = !dead && Boolean(reloadRow) && time - reloadRow[0] < RELOAD_PULSE && (!track.lives || reloadRow[1] === life?.id);
    const weapon = weaponState(track, life, dead);
    const zoom = zoomState(track, life, dead);
    const vitality = { supported: Boolean(track.vitality), body: null, shield: null };
    if (!dead) for (const kind of ['body', 'shield']) {
      const rows = track.vitality?.[kind] || [], row = rows[sampleIndex(time, rows)];
      if (!row || (track.lives && row.at(-1) !== life?.id)) continue;
      const age = time - row[0], stateBits = row[kind === 'body' ? 2 : 3];
      vitality[kind] = { time: row[0], raw: row[1], serial: row.at(-1), age, stale: age > MAX_GAP,
        fraction: Math.max(0, Math.min(1, row[1] / VITALITY_SCALES[kind])), stateBits,
        recovering: stateBits === (kind === 'body' ? '110' : '1100'),
        delayTicks: kind === 'shield' ? row[2] : null };
    }
    v.meterGroup.hidden = false;
    for (const meter of v.meters) paintMeter(meter, vitality[meter.kind], dead, vitality.supported);
    const count = sampleIndex(time, segments) + 1;
    const first = clip.trailWindow ? sampleIndex(Math.max(time - clip.trailWindow, life?.start || 0), segments) + 1 : 0;
    trail.geometry.setDrawRange(first * 2, Math.max(0, count - first) * 2); trail.visible = showTrail;
    fullTrail.visible = showTrail && !clip.trailWindow;
    v.state = { id: track.id, name: track.name, life: life?.id ?? null, sample: index, position: index < 0 ? null : [current.x, -current.z, current.y], positionAge, stale,
      aimSample: aimIndex, aim, lookVisible: look.visible, avatarVisible: avatar.visible, schematicPivot: index < 0 && aimIndex >= 0, dead, positionStatus, aimStatus,
      trailSegments: Math.max(0, count - first), firing, firingIndex, firingTime: firingRow?.[0] ?? null,
      firingSupported: Array.isArray(track.firing), muzzleVisible: muzzle.visible, weapon, zoom, crouch: crouchState(track, life, dead),
      reload, reloadTime: reloadRow && (!track.lives || reloadRow[1] === life?.id) ? reloadRow[0] : null, reloadSupported: Array.isArray(track.reload),
      melee, meleeIndex, meleeTime: meleeRow?.[0] ?? null, meleeSupported: Array.isArray(track.melee), meleeRingVisible: v.meleeRing.visible, grenade, grenadeIndex, grenadeTime: grenadeRow?.[0] ?? null, grenadeSupported: Array.isArray(track.grenade), vitality };
    return v.state;
  }
  function updateTime(followPlayhead = true) {
    if (followPlayhead) revealPlayhead();
    $('timeline-detail').classList.toggle('playhead-outside', time < windowStart || time > windowEnd);
    $('timeline').value = time; $('current-time').textContent = formatTime(time); $('play').textContent = playing ? 'Ⅱ' : '▶'; $('play').setAttribute('aria-label', playing ? 'Pause' : 'Play');
    $('status').textContent = playing ? 'PLAYING' : 'PAUSED'; $('status-dot').classList.toggle('playing', playing);
    const states = views.map(updatePlayer), s = states[selected];
    $('position-title').textContent = views.length > 1 ? s.name.toUpperCase() : 'PLAYER POSITION';
    for (const [i, axis] of ['x', 'y', 'z'].entries()) $(`coord-${axis}`).textContent = s.position ? s.position[i].toFixed(1) : '—';
    $('observation').textContent = s.positionStatus;
    $('aim-yaw').textContent = s.aim ? String(Math.round(s.aim.yawRaw) % 4096) : '—'; $('aim-pitch').textContent = s.aim ? String(Math.round(s.aim.pitchRaw)) : '—';
    $('aim-observation').textContent = s.aimStatus;
    paintZoom($('zoom-state'), s);
    $('zoom-observation').textContent = s.zoom.time === null ? 'No scope observation' : `Last sample · ${s.zoom.time.toFixed(3)} s · stage, not magnification`;
    const zoomRows = views[selected].track.zoom || [];
    $('previous-zoom').disabled = sampleIndex(time - .0005, zoomRows) < 0;
    $('next-zoom').disabled = sampleIndex(time + .0005, zoomRows) + 1 >= zoomRows.length;
    [...$('roster').children].forEach((button, i) => {
      const state = states[i], v = views[i];
      button.classList.toggle('selected', i === selected); button.classList.toggle('firing', state.firing); button.classList.toggle('melee', state.melee); button.classList.toggle('grenade', state.grenade); button.classList.toggle('dead', state.dead);
      button.setAttribute('aria-pressed', String(i === selected)); v.rosterName.textContent = state.name + (state.dead ? ' · eliminated' : '');
      paintActivity(v.rosterActivity, state); paintActivity(v.activity, state);
      paintWeapon(v.rosterWeapon, state); paintWeapon(v.weaponLabel, state);
      paintZoom(v.rosterZoom, state); paintZoom(v.zoomLabel, state);
      for (const meter of v.rosterMeters) paintMeter(meter, state.vitality[meter.kind], state.dead, state.vitality.supported);
    });
    $('firing-state').textContent = s.firing ? '● FIRING' : (s.firingSupported ? '○ FIRING' : 'FIRING UNAVAILABLE');
    $('firing-state').classList.toggle('active', s.firing);
    $('firing-observation').textContent = s.firingTime != null ? `${s.firing ? 'Event' : 'Last event'} · ${s.firingTime.toFixed(3)} s` : (s.firingSupported ? 'No decoded firing event yet' : 'No firing export for this clip');
    const firingRows = views[selected].track.firing || [];
    $('previous-firing').disabled = !firingRows.some(r => r[0] < time - .0005);
    $('next-firing').disabled = sampleIndex(time + .0005, firingRows) + 1 >= firingRows.length;
    $('melee-state').textContent = s.melee ? '● MELEE' : (s.meleeSupported ? '○ MELEE' : 'MELEE UNKNOWN');
    $('melee-state').classList.toggle('active', s.melee);
    $('melee-observation').textContent = s.meleeTime != null ? `${s.melee ? 'Event' : 'Last event'} · ${s.meleeTime.toFixed(3)} s` : (s.meleeSupported ? 'No decoded melee event yet' : 'No melee export for this clip');
    const meleeRows = views[selected].track.melee || [];
    $('previous-melee').disabled = sampleIndex(time - .0005, meleeRows) < 0;
    $('next-melee').disabled = sampleIndex(time + .0005, meleeRows) + 1 >= meleeRows.length;
    $('grenade-state').textContent = s.grenade ? '● GRENADE THROW' : (s.grenadeSupported ? '○ GRENADE THROW' : 'GRENADE UNKNOWN');
    $('grenade-state').classList.toggle('active', s.grenade);
    $('grenade-observation').textContent = s.grenadeTime != null ? `${s.grenade ? 'Event' : 'Last event'} · ${s.grenadeTime.toFixed(3)} s` : (s.grenadeSupported ? 'No decoded throw yet' : 'No grenade export for this clip');
    const grenadeRows = views[selected].track.grenade || [];
    $('previous-grenade').disabled = sampleIndex(time - .0005, grenadeRows) < 0;
    $('next-grenade').disabled = sampleIndex(time + .0005, grenadeRows) + 1 >= grenadeRows.length;
    $('reload-state').textContent = s.reload ? '● RELOAD START' : (s.reloadSupported ? '○ RELOAD START' : 'RELOAD UNKNOWN');
    $('reload-state').classList.toggle('active', s.reload);
    $('reload-observation').textContent = s.reloadTime != null ? `Start event · ${s.reloadTime.toFixed(3)} s` : 'No decoded reload start yet';
    const reloadRows = views[selected].track.reload || [];
    $('previous-reload').disabled = sampleIndex(time - .0005, reloadRows) < 0;
    $('next-reload').disabled = sampleIndex(time + .0005, reloadRows) + 1 >= reloadRows.length;
    paintWeapon($('weapon-state'), s);
    for (const meter of readoutMeters) paintMeter(meter, s.vitality[meter.kind], s.dead, s.vitality.supported);
    const vitalityRows = views[selected].vitalityChanges;
    $('previous-vitality').disabled = sampleIndex(time - .0005, vitalityRows) < 0;
    $('next-vitality').disabled = sampleIndex(time + .0005, vitalityRows) + 1 >= vitalityRows.length;
    const death = deathEvents.findLast(e => e.time <= time && (views.length <= 2 || time - e.time < 5));
    $('event-status').hidden = !death;
    $('event-status').textContent = death ? `${death.killer ? death.killer + ' eliminated ' : 'Death: '}${death.player} · ${death.time.toFixed(3)} s` : '';
    $('focus-view').disabled = !s.avatarVisible;
    const armor = paintArmor(views[selected].track);
    window.theaterViewerState = { clip: clip.id, time, start, end, windowStart, windowEnd, playing, fullMode, ...s, armor, players: states, projectiles: projectileViews.map(updateProjectile) };
  }
  function updateLabels() {
    for (const v of projectileViews) {
      const p = v.ball.position.clone(); p.y += markerSize * .8; p.project(camera);
      v.label.hidden = !v.ball.visible || p.z < -1 || p.z > 1 || Math.abs(p.x) > 1 || Math.abs(p.y) > 1;
      v.label.style.left = `${(p.x + 1) * host.clientWidth / 2}px`;
      v.label.style.top = `${(1 - p.y) * host.clientHeight / 2}px`;
    }
    if (!showCards) {
      for (const v of views) v.label.hidden = true;
      return;
    }
    const width = host.clientWidth, height = host.clientHeight, small = width < 600;
    const items = [];
    for (const [i, v] of views.entries()) {
      const p = v.avatar.position.clone(); p.y += markerSize * (v.state?.dead ? 1.5 : 3.1);
      const distance = p.distanceToSquared(camera.position);
      p.project(camera);
      const visible = v.avatar.visible && p.z >= -1 && p.z <= 1 && Math.abs(p.x) <= 1 && Math.abs(p.y) <= 1;
      v.label.hidden = !visible;
      v.label.classList.toggle('selected', i === selected); v.label.setAttribute('aria-pressed', String(i === selected));
      if (!visible) continue;
      const name = v.track.name + (v.state?.dead ? ' · eliminated' : '');
      if (v.labelName.textContent !== name) v.labelName.textContent = name;
      v.label.title = `${v.track.name}${v.state?.dead ? ' · eliminated' : v.state?.stale ? ` · last position ${v.state.positionAge?.toFixed(1)}s ago` : ''}. Select for full details.`;
      v.label.classList.toggle('compact', views.length > 4 && i !== selected);
      v.label.classList.toggle('minimal', small);
      v.label.classList.toggle('stale', false);
      for (const state of ['firing', 'melee', 'grenade']) v.label.classList.toggle(state, Boolean(v.state?.[state]));
      items.push({ v, distance, index: i, x: (p.x + 1) * width / 2, y: (1 - p.y) * height / 2 });
    }
    // Keep cards anchored to their players and stack nearer cards on top.
    items.sort((a, b) => a.distance - b.distance || a.index - b.index);
    for (const [rank, { v, x, y }] of items.entries()) {
      v.label.style.left = `${x}px`; v.label.style.top = `${y}px`;
      v.label.style.zIndex = String(items.length - rank);
    }
    // Read bounds together after positioning. Only overlap inside the scene
    // counts, since its edges clip the labels.
    const sceneRect = host.getBoundingClientRect();
    const bounds = items.map(({ v }) => {
      const r = v.label.getBoundingClientRect();
      return { left: Math.max(r.left, sceneRect.left), right: Math.min(r.right, sceneRect.right),
        top: Math.max(r.top, sceneRect.top), bottom: Math.min(r.bottom, sceneRect.bottom), depth: 0 };
    });
    for (const [rank, box] of bounds.entries()) {
      for (let i = 0; i < rank; i++) {
        const front = bounds[i];
        if (box.left < front.right && box.right > front.left && box.top < front.bottom && box.bottom > front.top)
          box.depth = Math.max(box.depth, front.depth + 1);
      }
      items[rank].v.label.style.opacity = String(.15 + .85 / (box.depth + 1));
    }
  }

  function playPause() { if (time >= end) time = start; playing = !playing; updateTime(); }
  function step(direction) {
    playing = false; const index = sampleIndex(time, timelineSamples);
    const next = direction > 0 ? Math.min(timelineSamples.length - 1, index + 1) : Math.max(0, index - (index >= 0 && Math.abs(timelineSamples[index][0] - time) < .0005 ? 1 : 0));
    time = Math.max(start, Math.min(end, timelineSamples[next][0])); updateTime();
  }
  function jumpFiring(direction, kind = 'firing') {
    const rows = views[selected].track[kind] || [];
    const index = direction > 0 ? sampleIndex(time + .0005, rows) + 1 : sampleIndex(time - .0005, rows);
    if (index < 0 || index >= rows.length) return;
    const destination = rows[index][0];
    if (destination < start || destination > end) setRange(true);
    playing = false; time = destination; updateTime();
  }
  function jumpVitality(direction) {
    const rows = views[selected].vitalityChanges;
    const index = direction > 0 ? sampleIndex(time + .0005, rows) + 1 : sampleIndex(time - .0005, rows);
    if (index < 0 || index >= rows.length) return;
    const destination = rows[index][0];
    if (destination < start || destination > end) setRange(true);
    playing = false; time = destination; updateTime();
  }
  function updateClipPicker() {
    $('clip').replaceChildren();
    for (const groupName of [...new Set(CLIPS.map(c => c.category || c.group))]) {
      const group = document.createElement('optgroup'); group.label = groupName;
      for (const c of CLIPS.filter(c => (c.category || c.group) === groupName)) { const option = document.createElement('option'); option.value = c.id; option.textContent = c.name; group.append(option); }
      $('clip').append(group);
    }
  }
  updateClipPicker();
  $('film-file').onchange = async event => {
    const file = event.target.files[0]; if (!file) return;
    $('film-import-status').textContent = 'Opening film…';
    try {
      const film = JSON.parse(await file.text());
      const existing = CLIPS.find(c => c.match_id && c.match_id === film.match_id);
      const imported = filmToClip(film, existing ? { id: existing.id, title: existing.name, description: existing.description, category: existing.category, group: existing.group } : {});
      const index = CLIPS.findIndex(c => c.id === imported.id);
      if (index < 0) CLIPS.push(imported); else CLIPS[index] = imported;
      updateClipPicker(); loadClip(imported.id);
      $('film-import-status').textContent = 'Film opened';
    } catch (error) { $('film-import-status').textContent = error.message; }
    event.target.value = '';
  };
  $('clip').onchange = () => loadClip($('clip').value); $('play').onclick = playPause; $('previous').onclick = () => step(-1); $('next').onclick = () => step(1);
  $('timeline').oninput = () => { playing = false; time = +$('timeline').value; updateTime(); };
  $('timeline-zoom-in').onclick = () => zoomTimeline((windowEnd - windowStart) / 2);
  $('timeline-zoom-out').onclick = () => zoomTimeline((windowEnd - windowStart) * 2);
  $('timeline-fit').onclick = () => zoomTimeline(end - start, start, 0);
  $('timeline-focus').onclick = () => zoomTimeline(windowEnd - windowStart);
  $('timeline-span').onchange = () => zoomTimeline($('timeline-span').value === 'all' ? end - start : Number($('timeline-span').value));
  $('timeline-pan').oninput = () => panTimeline(Number($('timeline-pan').value));
  $('timeline-detail').addEventListener('wheel', e => {
    e.preventDefault();
    const width = windowEnd - windowStart;
    if (e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      panTimeline(windowStart + (e.deltaX || e.deltaY) * width / 500); return;
    }
    const rect = $('timeline').getBoundingClientRect();
    const fraction = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    zoomTimeline(width * Math.exp(Math.max(-200, Math.min(200, e.deltaY)) * .005), windowStart + width * fraction, fraction);
  }, { passive: false });
  $('action-mode').onclick = () => setRange(false); $('full-mode').onclick = () => setRange(true); $('reset-view').onclick = () => resetCamera(); $('top-view').onclick = () => resetCamera(true);
  $('focus-view').onclick = () => { const v = views[selected]; if (!v.avatar.visible) return; target.copy(v.avatar.position); target.y += markerSize; radius = size * .55; updateCamera(); };
  $('event-select').onchange = () => { if ($('event-select').value === '') return; playing = false; time = +$('event-select').value; updateTime(); $('event-select').value = ''; };
  $('previous-firing').onclick = () => jumpFiring(-1); $('next-firing').onclick = () => jumpFiring(1);
  $('previous-melee').onclick = () => jumpFiring(-1, 'melee'); $('next-melee').onclick = () => jumpFiring(1, 'melee');
  $('previous-grenade').onclick = () => jumpFiring(-1, 'grenade'); $('next-grenade').onclick = () => jumpFiring(1, 'grenade');
  $('previous-reload').onclick = () => jumpFiring(-1, 'reload'); $('next-reload').onclick = () => jumpFiring(1, 'reload');
  $('previous-zoom').onclick = () => jumpFiring(-1, 'zoom'); $('next-zoom').onclick = () => jumpFiring(1, 'zoom');
  $('previous-vitality').onclick = () => jumpVitality(-1); $('next-vitality').onclick = () => jumpVitality(1);
  $('trail-toggle').onclick = () => { showTrail = !showTrail; $('trail-toggle').setAttribute('aria-pressed', String(showTrail)); updateTime(); };
  $('look-toggle').onclick = () => { showLook = !showLook; $('look-toggle').setAttribute('aria-pressed', String(showLook)); updateTime(); };
  $('cards-toggle').onclick = () => { showCards = !showCards; $('cards-toggle').setAttribute('aria-pressed', String(showCards)); updateLabels(); };
  host.addEventListener('pointerdown', e => { if (e.button !== 0 || e.target.closest('.player-name')) return; drag = [e.clientX, e.clientY]; host.setPointerCapture(e.pointerId); host.classList.add('dragging'); });
  host.addEventListener('pointermove', e => { if (!drag) return; theta -= (e.clientX - drag[0]) * .006; phi = Math.max(.015, Math.min(1.5, phi + (e.clientY - drag[1]) * .006)); drag = [e.clientX, e.clientY]; updateCamera(); });
  for (const name of ['pointerup', 'pointercancel', 'lostpointercapture']) host.addEventListener(name, () => { drag = null; host.classList.remove('dragging'); });
  host.addEventListener('wheel', e => { e.preventDefault(); radius = Math.max(size * .35, Math.min(size * 8, radius * Math.exp(e.deltaY * .001))); updateCamera(); }, { passive: false });
  document.addEventListener('keydown', e => {
    if (['SELECT', 'INPUT', 'BUTTON', 'SUMMARY'].includes(e.target.tagName)) return;
    if (e.code === 'Space') { e.preventDefault(); playPause(); }
    if (e.code === 'ArrowRight' || e.code === 'ArrowLeft') { e.preventDefault(); step(e.code === 'ArrowRight' ? 1 : -1); }
    if (e.code === 'KeyR') resetCamera();
  });
  const resize = () => { camera.aspect = host.clientWidth / host.clientHeight; camera.updateProjectionMatrix(); renderer.setSize(host.clientWidth, host.clientHeight); if (clip) drawCoverage(); };
  new ResizeObserver(resize).observe(host); loadClip(new URLSearchParams(location.search).get('clip') || CLIPS[0].id); resize();
  renderer.setAnimationLoop(() => {
    const dt = Math.min(clock.getDelta(), .1);
    if (playing) { time += dt * +$('speed').value; if (time > end) { if ($('loop').checked) time = start + (time - start) % (end - start); else { time = end; playing = false; } } updateTime(); }
    updateLabels(); renderer.render(scene, camera);
  });
})();
