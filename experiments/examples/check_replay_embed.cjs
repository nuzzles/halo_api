/* Single-film export and browser regression. Requires a cached decoded film and Chrome.
   Optional CHROME_PATH. Default fixture: octagon/03-first-to-50/decoded-film.json. */
const { spawn, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const assert = require('node:assert/strict');
const os = require('node:os');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { gunzipSync } = require('node:zlib');

const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'halo-embed-check-'));
const filmPath = path.resolve(process.argv[2] || path.join(__dirname, '../films/octagon/03-first-to-50/decoded-film.json'));
const decoded = JSON.parse(fs.readFileSync(filmPath, 'utf8'));
const fixedLoadout = filmPath.includes('03-first-to-50');
const builder = path.join(__dirname, 'build_decoded_replay.cjs');
const embedPath = path.join(workspace, 'embed.html'), labPath = path.join(workspace, 'lab.html');
const build = args => spawnSync(process.execPath, [builder, ...args], { encoding: 'utf8' });
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

async function run() {
  for (const [output, flags] of [[embedPath, ['--embed', '--octagon-walls', ...(fixedLoadout ? ['--fixed-loadout', 'Bandit EVO,S7 Sniper'] : [])]], [labPath, []]]) {
    const result = build([...flags, filmPath, '--output', output]);
    assert.equal(result.status, 0, result.stderr);
  }
  const exported = fs.readFileSync(embedPath, 'utf8');
  assert.deepEqual(gunzipSync(fs.readFileSync(path.join(workspace, 'embed.json.gz'))), fs.readFileSync(filmPath));
  assert(!exported.includes('type="file"'));
  assert(!exported.includes('class="experiment-nav"'));
  assert(exported.includes('Copyright © 2010-2023 three.js authors'));
  assert(exported.includes('Copyright (c) 2026 Spencer C. Imbleau'));
  // Failed exports cannot replace a usable embed.
  const badFilm = path.join(workspace, 'bad.json');
  fs.writeFileSync(badFilm, JSON.stringify({ schema_version: 99, players: [] }));
  for (const args of [[filmPath, filmPath], ['--corpus', path.join(__dirname, '../films/aim')], [badFilm]]) {
    assert.notEqual(build(['--embed', ...args, '--output', embedPath]).status, 0);
    assert.equal(fs.readFileSync(embedPath, 'utf8'), exported);
  }

  const chrome = spawn(process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', [
    '--headless', '--remote-debugging-pipe', '--no-first-run', '--disable-background-networking',
    '--disable-default-apps', '--disable-sync', '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
    '--user-data-dir=' + path.join(workspace, 'profile'),
  ], { stdio: ['ignore', 'ignore', 'ignore', 'pipe', 'pipe'] });
  const exited = new Promise(resolve => { chrome.once('exit', resolve); chrome.once('error', resolve); });
  let nextId = 0, buffer = '', session;
  const pending = new Map(), errors = [], requests = [];
  chrome.stdio[4].on('data', chunk => {
    buffer += chunk.toString(); let i;
    while ((i = buffer.indexOf('\0')) >= 0) {
      const raw = buffer.slice(0, i); buffer = buffer.slice(i + 1); if (!raw) continue;
      const msg = JSON.parse(raw);
      if (msg.id && pending.has(msg.id)) {
        const p = pending.get(msg.id); pending.delete(msg.id); clearTimeout(p.timer);
        msg.error ? p.reject(new Error(JSON.stringify(msg.error))) : p.resolve(msg.result);
      }
      if (msg.method === 'Runtime.exceptionThrown') errors.push(msg.params.exceptionDetails);
      if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') errors.push(msg.params.args);
      if (msg.method === 'Network.requestWillBeSent') requests.push(msg.params.request.url);
    }
  });
  function cdp(method, params = {}, sid = session) {
    return new Promise((resolve, reject) => {
      const id = ++nextId, timer = setTimeout(() => { pending.delete(id); reject(new Error('Timed out: ' + method)); }, 30000);
      pending.set(id, { resolve, reject, timer });
      chrome.stdio[3].write(JSON.stringify({ id, method, params, ...(sid ? { sessionId: sid } : {}) }) + '\0');
    });
  }
  async function evaluate(expression) {
    const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.exceptionDetails) throw new Error(JSON.stringify(r.exceptionDetails));
    return r.result.value;
  }
  const click = id => evaluate(`document.getElementById(${JSON.stringify(id)}).click(); window.theaterViewerState`);
  const scrub = t => evaluate(`document.getElementById('timeline').value=${t}; document.getElementById('timeline').dispatchEvent(new Event('input')); window.theaterViewerState`);
  async function waitForState(expression) {
    for (let i = 0; i < 150; i++) { if (await evaluate(expression)) return; await delay(100); }
    assert.fail(`Replay did not reach ${expression}: ${JSON.stringify(errors)}`);
  }
  async function open(file, hash = '') {
    await cdp('Page.navigate', { url: 'about:blank' });
    await cdp('Page.navigate', { url: pathToFileURL(file).href + hash });
    for (let i = 0; i < 100; i++) {
      const state = await evaluate('window.theaterViewerState'); if (state) return state;
      await delay(100);
    }
    throw new Error('Replay did not initialize');
  }
  try {
    const target = await cdp('Target.createTarget', { url: 'about:blank' }, null);
    session = (await cdp('Target.attachToTarget', { targetId: target.targetId, flatten: true }, null)).sessionId;
    await cdp('Runtime.enable'); await cdp('Page.enable'); await cdp('Network.enable');
    await cdp('Emulation.setDeviceMetricsOverride', { width: 900, height: 620, deviceScaleFactor: 1, mobile: false });
    const initial = await open(embedPath);
    assert.equal(initial.playing, false);
    assert.equal(await evaluate('CLIPS.length'), 1);
    assert.equal(await evaluate('document.querySelectorAll("input[type=file], .experiment-nav, #clip").length'), 0);
    assert.equal(await evaluate('document.querySelectorAll("#loop, #speed, #playback-window, #previous, #next").length'), 0);
    assert.equal(await evaluate('typeof filmToClip'), 'undefined');
    assert.equal(await evaluate('typeof TheaterRecordings'), 'undefined');
    assert.equal(await evaluate('document.getElementById("focus-view")'), null);
    assert.equal(initial.placeholderArena.walls, 8);
    assert.equal(initial.placeholderArena.placeholder, true);
    assert.equal(initial.viewCamera.mode, 'shoulder');
    assert.equal(initial.viewCamera.player, initial.players.find(p => p.name === 'Nuzzles')?.id || initial.id);
    const fullUnknown = await evaluate(`[...document.querySelectorAll('.player-vitality .assumed-full')].map(e => ({fill:e.querySelector('.meter-bar i').style.width, value:e.querySelector('.meter-heading strong').textContent, raw:e.querySelector('.meter-bar').getAttribute('aria-valuenow')}))`);
    assert(fullUnknown.length); assert(fullUnknown.every(m => m.fill === '100%' && ['64', '126'].includes(m.value) && m.raw === null));
    const checkpoints = await evaluate(`CLIPS[0].players.flatMap(p => p.lives.flatMap(l =>
      [l.start, Math.min(l.start + .2, l.end), ...(l.death == null ? [] : [l.death + .001])])).filter(t => t <= CLIPS[0].duration)`);
    const times = checkpoints.filter((_, i) => i % Math.max(1, Math.floor(checkpoints.length / 18)) === 0);
    times.push((initial.start + initial.end) / 2, initial.end);
    const observations = [], scores = [];
    let sawPast = false, sawFuture = false, sawVelocity = false;
    for (const t of times) {
      const state = await scrub(t); observations.push(state.players);
      scores.push(state.score);
      for (const p of state.score) assert.equal(p.kills, decoded.summary_events.filter(e => e.kind === 'Kill' && e.name === p.name && e.time_us / 1e6 <= state.time).length);
      for (const p of state.players) {
        const { past, future } = p.motionTrails;
        assert.equal(p.motionTrails.width, 5);
        sawPast ||= past.count > 0; sawFuture ||= future.count > 0; sawVelocity ||= p.velocityVisible;
        for (const range of [past, future]) if (range.count) {
          assert(range.firstTime >= range.from); assert(range.lastTime <= range.to);
          assert(range.lastTime - range.firstTime <= 10.000001);
          const rows = await evaluate(`CLIPS[0].players.find(p=>p.id===${JSON.stringify(p.id)}).samples.filter(r=>r[0]>=${range.firstTime} && r[0]<=${range.lastTime})`);
          assert(rows.every(row => row[6] === p.life), 'Trail must stay in its current life');
        }
        assert(past.to <= state.time); assert(future.from >= state.time);
      }
    }
    assert(sawPast && sawFuture, 'Both trail directions should render'); assert(sawVelocity, 'Recorded velocity arrows should render');
    for (let i = times.length - 1; i >= 0; i--) { const state = await scrub(times[i]); assert.deepEqual(state.players, observations[i]); assert.deepEqual(state.score, scores[i]); }
    // Recorded shots and melee drive illustrative articulation, including reverse seeks.
    const actions = await evaluate(`CLIPS[0].players.flatMap(p => ['firing','melee'].flatMap(kind =>
      (p[kind] || []).filter(r => {const l=p.lives.find(l=>l.id===r[1]);return l && r[0]+.08<(l.death??l.end)}).slice(0, 4).map(r=>({id:p.id,kind,time:r[0]+.075}))))`);
    const poses = [];
    for (const action of actions) {
      const player = (await scrub(action.time)).players.find(p=>p.id===action.id);
      assert(player.presentation[action.kind === 'firing' ? 'recoil' : 'melee'] > .1);
      poses.push(player.presentation);
    }
    for (let i=actions.length-1;i>=0;i--) assert.deepEqual((await scrub(actions[i].time)).players.find(p=>p.id===actions[i].id).presentation, poses[i]);
    const weapons = new Set();
    for (const t of [147.3, 150.79, 156.49, 151.4, 152.4, 147.3]) {
      const players = (await scrub(t)).players;
      const labels = await evaluate(`[...document.querySelectorAll('.player-name .weapon-summary')].map(e=>({text:e.textContent,display:getComputedStyle(e).display}))`);
      for (const [i, p] of players.entries()) {
        if (p.weapon.name === 'Bandit EVO') { assert.equal(p.presentation.weapon, 'bandit'); weapons.add('bandit'); }
        if (p.weapon.name === 'S7 Sniper') { assert.equal(p.presentation.weapon, 'sniper'); weapons.add('sniper'); }
        assert.equal(labels[i].text, p.dead ? '' : p.presentation.heldWeapon.name || 'Weapon unknown');
        assert.equal(labels[i].display, p.dead ? 'none' : 'block');
      }
    }
    if (filmPath.includes('03-first-to-50')) assert.equal(weapons.size, 2);
    if (fixedLoadout) {
      // Check actual change timestamps before the next shot, including two
      // rapid swaps, then repeat in reverse to catch stateful toggle bugs.
      const swaps = [['1', 149.98, 'bandit'], ['1', 150.01, 'sniper'],
        ['0', 155.35, 'bandit'], ['0', 155.4, 'sniper'],
        ['0', 158.9, 'bandit'], ['0', 159.1, 'sniper']];
      for (const [id, t, expected] of [...swaps, ...swaps.toReversed()]) {
        const player = (await scrub(t)).players.find(p=>p.id===id);
        assert.equal(player.presentation.weapon, expected);
        assert.equal(player.firing, false, 'Switch presentation must not need a shot');
        const label = await evaluate(`[...document.querySelectorAll('.player-name')].find(e=>e.querySelector('.player-name-text').textContent===${JSON.stringify(player.name)}).querySelector('.weapon-summary').textContent`);
        assert.equal(label, expected === 'sniper' ? 'S7 Sniper' : 'Bandit EVO');
      }
      const respawn = (await scrub(152.4)).players.find(p=>p.id==='1');
      assert.equal(respawn.presentation.weapon,'bandit');
      assert.equal(respawn.presentation.heldWeapon.assumed,true);
      const confirmed = (await scrub(156.49)).players.find(p=>p.id==='0');
      assert.equal(confirmed.presentation.heldWeapon.assumed,false);
    }
    // This match has no grenade/reload events. Synthetic input tests the model
    // gestures without adding actions to the recording.
    const gestures = await evaluate(`(() => {
      const rig=TheaterSpartan.create(THREE,new THREE.Group(),1,'#2047d6');
      const state={id:'0',weapon:{name:'Bandit EVO'},crouch:{value:false},dead:false,stale:false};
      const pose=(patch,t)=>{const p=rig.pose({...state,...patch},t);rig.root.updateMatrixWorld(true);const matrices=[];rig.root.traverse(c=>matrices.push(c.matrixWorld.elements.slice()));return {p,matrices}};
      const neutral=pose({},10), throwing=pose({grenade:true,grenadeTime:10},10.225), reload=pose({reload:true,reloadTime:10},10.25);
      const reversed=pose({},10);
      return {neutral,throwing,reload,reversed};
    })()`);
    assert(gestures.throwing.p.throwing>.99); assert(gestures.reload.p.reload>.99);
    assert.notDeepEqual(gestures.neutral.matrices,gestures.throwing.matrices);
    assert.notDeepEqual(gestures.neutral.matrices,gestures.reload.matrices);
    assert.deepEqual(gestures.neutral,gestures.reversed);
    await scrub(151.4);
    if (filmPath.includes('03-first-to-50')) {
      assert.match(await evaluate('document.getElementById("event-status").textContent'), /^Nuzzles killed timesknightt/);
      assert.equal(await evaluate('document.querySelectorAll("#event-status s").length'),0);
      assert.equal(await evaluate('getComputedStyle(document.querySelector(".player-name.defeated .player-name-text")).textDecorationLine'),'line-through');
      assert.equal(await evaluate('getComputedStyle(document.querySelector(".score-player > span")).textDecorationLine'),'none');
    }
    await scrub(initial.start); await click('play'); await waitForState(`window.theaterViewerState.time > ${initial.start}`);
    const played = await click('play'); assert(played.time > initial.start); assert(!played.playing);
    await scrub(initial.end - .05); await click('play'); await waitForState('!window.theaterViewerState.playing');
    const ended = await evaluate('window.theaterViewerState');
    assert.equal(ended.time, initial.end); assert.equal(ended.playing, false);
    await click('play'); await waitForState(`window.theaterViewerState.time > ${initial.start} && window.theaterViewerState.time < ${initial.end}`); const restarted = await click('play');
    assert(restarted.time < initial.end); assert(restarted.time >= initial.start);
    await scrub((initial.start + initial.end) / 2);
    const cameraTime = await evaluate('window.theaterViewerState.time');
    const cameraOptions = await evaluate('[...document.getElementById("camera-view").options].map(o=>o.value)');
    for (const option of cameraOptions) {
      const state = await evaluate(`document.getElementById('camera-view').value=${JSON.stringify(option)};document.getElementById('camera-view').dispatchEvent(new Event('change'));window.theaterViewerState`);
      assert.equal(state.time, cameraTime); assert.equal(state.viewCamera.mode, option === 'overview' ? 'overview' : 'shoulder');
      if (option !== 'overview') assert.equal(state.viewCamera.player, option);
    }
    const wheel = await evaluate(`(() => {
      const host = document.getElementById('scene');
      const plain = new WheelEvent('wheel', {deltaY:100, cancelable:true}); host.dispatchEvent(plain);
      const zoom = new WheelEvent('wheel', {deltaY:100, ctrlKey:true, cancelable:true}); host.dispatchEvent(zoom);
      return [plain.defaultPrevented, zoom.defaultPrevented];
    })()`);
    assert.deepEqual(wheel, [false, true]);
    for (const width of [900, 390, 320, 280]) {
      await cdp('Emulation.setDeviceMetricsOverride', { width, height: 620, deviceScaleFactor: 1, mobile: width < 500 });
      await scrub(154.2);
      await evaluate(`document.getElementById('camera-view').value=CLIPS[0].players[0].id;document.getElementById('camera-view').dispatchEvent(new Event('change'));`);
      await delay(150);
      const layout = await evaluate(`(() => {
        const rect = q => document.querySelector(q).getBoundingClientRect();
        return {width:document.documentElement.scrollWidth,height:document.documentElement.scrollHeight,
          scene:rect('#scene').height, timelineTop:rect('.transport').top, sceneBottom:rect('.viewport').bottom,
          visibleChildren:[...document.querySelector('main').children].filter(e=>getComputedStyle(e).display!=='none' && e.getBoundingClientRect().height>0).map(e=>e.className),
          scoreRight:rect('#replay-score').right, scoreBottom:rect('#replay-score').bottom, sceneRight:rect('.viewport').right,
          feedLeft:rect('#event-status').left, feedBottom:rect('#event-status').bottom,
          cardCenter:rect('.camera-player').left+rect('.camera-player').width/2, cardTop:rect('.camera-player').top,
          inspectorVisible:getComputedStyle(document.querySelector('.position-card')).display !== 'none'};
      })()`);
      assert(layout.width <= width, `Overflow at ${width}px`);
      assert(layout.height <= 621, `Vertical overflow at ${width}px: ${layout.height}`);
      assert(layout.scene >= 240); assert(layout.timelineTop >= layout.sceneBottom);
      assert.deepEqual(layout.visibleChildren, ['viewport', 'transport']); assert(!layout.inspectorVisible);
      assert(layout.scoreRight <= layout.sceneRight && layout.scoreRight > layout.sceneRight - 20);
      assert(layout.scoreBottom <= layout.sceneBottom && layout.scoreBottom > layout.sceneBottom - 20);
      assert(layout.feedLeft >= 0 && layout.feedLeft <= 12);
      assert(layout.feedBottom <= layout.sceneBottom && layout.feedBottom > layout.sceneBottom - 20);
      assert(Math.abs(layout.cardCenter-width/2)<1); assert(layout.cardTop>=0 && layout.cardTop<=12);
      if (width !== 320) {
        const shot = await cdp('Page.captureScreenshot', { format: 'png' });
        fs.writeFileSync(path.join(os.tmpdir(), `motion-replay-embed-${width}.png`), Buffer.from(shot.data, 'base64'));
      }
    }
    assert.deepEqual(requests.filter(url => /^https?:/.test(url)), [], 'Embed must not request the lab or external assets');
    assert.deepEqual(errors, []);
    const from = Math.round(initial.end * .4), to = from + .5;
    const configured = await open(embedPath, `#start=${from}&end=${to}`);
    assert.equal(configured.start, from); assert.equal(configured.end, to); assert.equal(configured.time, from);
    for (const [a, b] of [['4:99', '5:00'], ['20', '10'], ['-1', '10'], ['0', String(initial.end + 1)]]) {
      const fallback = await open(embedPath, `#start=${a}&end=${b}`);
      assert.equal(fallback.start, 0); assert.equal(fallback.end, initial.end); assert(fallback.playbackWindowError);
    }
    const bounded = await open(embedPath, '#start=0:10.000&end=10.5');
    assert.equal(bounded.start, 10); assert.equal(bounded.end, 10.5);
    await scrub(10.49); await click('play'); await waitForState('!window.theaterViewerState.playing');
    assert.equal(await evaluate('window.theaterViewerState.time'), 10.5);
    await delay(250); assert.equal(await evaluate('window.theaterViewerState.playing'), false);
    const fallback = await open(embedPath, '#start=bad&end=-1'); assert.equal(fallback.start, 0); assert.equal(fallback.end, initial.end);
    assert(fallback.playbackWindowError);
    // Full viewer and compact embed must use identical player observations.
    await open(labPath);
    assert(await evaluate('Boolean(document.getElementById("film-file"))'));
    const recorded = players => players.map(({ motionTrails, presentation, ...p }) => p);
    for (let i = 0; i < times.length; i++) assert.deepEqual(recorded((await scrub(times[i])).players), recorded(observations[i]));
    assert.deepEqual(errors, []);
    await cdp('Page.addScriptToEvaluateOnNewDocument', { source: `const originalContext = HTMLCanvasElement.prototype.getContext;
      HTMLCanvasElement.prototype.getContext = function(type, ...args) {
        return type.startsWith('webgl') || type === 'experimental-webgl' ? null : originalContext.call(this, type, ...args);
      };` });
    const unavailable = await open(embedPath);
    assert.equal(unavailable.error, 'WebGL unavailable');
    assert.equal(await evaluate('document.getElementById("webgl-error").hidden'), false);
    assert.equal(await evaluate('document.getElementById("play").disabled'), true);
    console.log(`Single-film embed passed: ${times.length} observation checkpoints, 5px past/future trails, recorded score and reverse seeking, velocity, unknown bars, code-only windows, placeholder walls, minimal responsive layout, full-viewer parity, WebGL fallback.`);
  } finally {
    for (const p of pending.values()) clearTimeout(p.timer);
    chrome.kill(); await exited;
  }
}
run().catch(error => { console.error(error); process.exitCode = 1; }).finally(() => fs.rmSync(workspace, { recursive: true, force: true }));
