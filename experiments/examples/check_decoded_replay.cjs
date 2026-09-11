/* Offline replay regression. Optional CHROME_PATH and THEATER_REPLAY_URL. */
const {spawn} = require('node:child_process');
const fs = require('node:fs');
const assert = require('node:assert/strict');
const os = require('node:os');
const path = require('node:path');
const {pathToFileURL} = require('node:url');
const profile = fs.mkdtempSync(path.join(os.tmpdir(),'halo-viewer-chrome-'));
const chrome = spawn(process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', ['--headless', '--remote-debugging-pipe', '--no-first-run', '--disable-background-networking', '--disable-default-apps', '--disable-sync', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--user-data-dir='+profile], {stdio:['ignore','ignore','pipe','pipe','pipe']});
let nextId=0, buffer='', session, errors=[];
const pending=new Map();
chrome.stderr.on('data',()=>{});
chrome.stdio[4].on('data',chunk=>{
 buffer+=chunk.toString();
 let i;
 while((i=buffer.indexOf('\0'))>=0){
  const raw=buffer.slice(0,i);buffer=buffer.slice(i+1);
  if(!raw)continue;
  const msg=JSON.parse(raw);
  if(msg.id){const p=pending.get(msg.id);if(p){pending.delete(msg.id);clearTimeout(p.timer);msg.error?p.reject(new Error(JSON.stringify(msg.error))):p.resolve(msg.result);}}
  if(msg.method==='Runtime.exceptionThrown')errors.push(msg.params.exceptionDetails);
  if(msg.method==='Runtime.consoleAPICalled' && msg.params.type==='error')errors.push(msg.params.args);
 }
});
function cdp(method,params={},sid=session){return new Promise((resolve,reject)=>{const id=++nextId;const timer=setTimeout(()=>reject(new Error('Timed out: '+method)),120000);pending.set(id,{resolve,reject,timer});chrome.stdio[3].write(JSON.stringify({id,method,params,...(sid?{sessionId:sid}:{})})+'\0');});}
const delay=ms=>new Promise(r=>setTimeout(r,ms));
async function evaluate(expression){const r=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw new Error(JSON.stringify(r.exceptionDetails));return r.result.value;}
async function snap(name){await delay(150);const r=await cdp('Page.captureScreenshot',{format:'png'});fs.writeFileSync(path.join(os.tmpdir(),name+'.png'),Buffer.from(r.data,'base64'));}
async function click(id){return evaluate(`document.getElementById(${JSON.stringify(id)}).click();window.theaterViewerState`);}
async function scrub(t){return evaluate(`document.getElementById('timeline').value=${t};document.getElementById('timeline').dispatchEvent(new Event('input'));window.theaterViewerState`);}
(async()=>{
 try {
  const target=await cdp('Target.createTarget',{url:'about:blank'},null);session=(await cdp('Target.attachToTarget',{targetId:target.targetId,flatten:true},null)).sessionId;
  await cdp('Runtime.enable');await cdp('Page.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1400,height:1100,deviceScaleFactor:1,mobile:false});
  await cdp('Page.navigate',{url:pathToFileURL(path.resolve(__dirname,'../films/analysis/theater_viewer.html')).href});
  let state;for(let n=0;n<100;n++){state=await evaluate('window.theaterViewerState');if(state)break;await delay(100);}
  assert(state);assert.equal(await evaluate('CLIPS.length'),32);
  await evaluate(`document.getElementById('timeline').step='any'`);
  async function choose(id){return evaluate(`document.getElementById('clip').value=${JSON.stringify(id)};document.getElementById('clip').dispatchEvent(new Event('change'));window.theaterViewerState`)}
  await choose('raids/01-hour-long-raid');await click('full-mode');
  const raid = await evaluate(`(()=>{
    const c=CLIPS.find(c=>c.id==='raids/01-hour-long-raid');
    return {duration:c.duration,players:c.players.length,lives:c.players.reduce((n,p)=>n+p.lives.length,0),
      positions:c.players.reduce((n,p)=>n+p.samples.length,0),aim:c.players.reduce((n,p)=>n+p.aim.length,0),
      title:document.getElementById('clip').selectedOptions[0].textContent,
      category:document.getElementById('clip').selectedOptions[0].parentElement.label};
  })()`);
  assert.deepEqual(raid,{duration:3953.667,players:20,lives:942,positions:958688,aim:814749,title:'Raid · 1 hour',category:'Raid gameplay'});
  state=await scrub(1800);const raidMiddle=state.players;
  assert.equal(state.players.length,20);assert(state.players.filter(p=>p.position).length>=15);
  assert(state.players.every(p=>p.crouch.value==null)); // Larger input rosters remain unsupported.
  state=await scrub(3900);assert(state.players.filter(p=>p.life>=768).length>=10);
  assert(state.players.find(p=>p.name==='Nuzzles').position);
  assert.equal(state.end,3953.667);
  // Zoom and pan keep recorded time and the full playback/loop range intact.
  const setSpan = value => evaluate(`document.getElementById('timeline-span').value=${JSON.stringify(value)};document.getElementById('timeline-span').dispatchEvent(new Event('change'));window.theaterViewerState`);
  state=await setSpan('60');assert.equal(state.windowEnd-state.windowStart,60);assert.equal(state.time,3900);assert.equal(state.end,3953.667);
  state=await click('timeline-zoom-in');assert.equal(state.windowEnd-state.windowStart,30);assert.equal(state.time,3900);
  state=await click('timeline-zoom-out');assert.equal(state.windowEnd-state.windowStart,60);
  const anchored=await evaluate(`(()=>{
    const before=window.theaterViewerState,r=document.getElementById('timeline').getBoundingClientRect();
    document.getElementById('timeline-detail').dispatchEvent(new WheelEvent('wheel',{deltaY:-120,clientX:r.left+r.width*.25,cancelable:true}));
    const after=window.theaterViewerState;return {before:[before.windowStart,before.windowEnd],after:[after.windowStart,after.windowEnd],time:after.time};
  })()`);
  assert(Math.abs(anchored.before[0]+(anchored.before[1]-anchored.before[0])*.25-anchored.after[0]-(anchored.after[1]-anchored.after[0])*.25)<.00001);
  assert.equal(anchored.time,3900);
  state=await evaluate(`document.getElementById('timeline-pan').value=100;document.getElementById('timeline-pan').dispatchEvent(new Event('input'));window.theaterViewerState`);
  assert.equal(state.windowStart,100);assert.equal(state.time,3900);
  assert(await evaluate(`document.getElementById('timeline-detail').classList.contains('playhead-outside')`));
  state=await click('timeline-focus');assert(state.windowStart<=3900 && state.windowEnd>=3900);
  state=await scrub(3900.123);assert.equal(state.time,3900.123);
  const shifted=await evaluate(`(()=>{const before=window.theaterViewerState.windowStart;document.getElementById('timeline-detail').dispatchEvent(new WheelEvent('wheel',{deltaY:100,shiftKey:true,cancelable:true}));return [before,window.theaterViewerState.windowStart,window.theaterViewerState.time];})()`);
  assert(shifted[1]>shifted[0]);assert.equal(shifted[2],3900.123);
  // An event outside the visible window brings its recorded time into view.
  state=await evaluate(`(()=>{const select=document.getElementById('event-select'),option=[...select.options].find(o=>o.value && Number(o.value)<100);select.value=option.value;select.dispatchEvent(new Event('change'));return window.theaterViewerState;})()`);
  assert(state.time<100 && state.windowStart<=state.time && state.windowEnd>=state.time);
  state=await click('timeline-fit');assert.equal(state.windowStart,0);assert.equal(state.windowEnd,3953.667);
  await scrub(3900);await setSpan('1');await scrub(3900.49);await click('play');await delay(450);state=await click('play');
  assert(state.time>3900.5);assert(state.windowEnd>3900.5);assert.equal(state.end,3953.667);
  await click('timeline-fit');await scrub(3900);await setSpan('60');
  await evaluate(`document.querySelector('.viewport').scrollIntoView({block:'start'})`);await delay(200);
  async function checkCardOcclusion() {
    const cards=await evaluate(`(()=>{
      const scene=document.getElementById('scene').getBoundingClientRect();
      return [...document.querySelectorAll('.player-name')].filter(e=>!e.hidden).map(e=>{
        const r=e.getBoundingClientRect();return {name:e.getAttribute('aria-label'),opacity:Number(getComputedStyle(e).opacity),order:Number(getComputedStyle(e).zIndex),selected:e.classList.contains('selected'),left:Math.max(r.left,scene.left),right:Math.min(r.right,scene.right),top:Math.max(r.top,scene.top),bottom:Math.min(r.bottom,scene.bottom)};
      }).sort((a,b)=>b.order-a.order);
    })()`);
    for(const [i,card] of cards.entries()) {
      const covered=cards.slice(0,i).some(front=>card.left<front.right && card.right>front.left && card.top<front.bottom && card.bottom>front.top);
      assert(card.opacity>0 && card.opacity<=1);
      assert.equal(card.opacity<1,covered,`Incorrect occlusion fade: ${card.name}`);
    }
    return cards;
  }
  const stacked=await checkCardOcclusion();
  assert(stacked.length>=15);assert(stacked.some(c=>c.selected && c.name==='Select Nuzzles'));
  assert(stacked.filter(c=>c.opacity===1).length>1);assert(stacked.some(c=>c.opacity<1));
  const picked = await evaluate(`(()=>{const label=[...document.querySelectorAll('.player-name')].find(e=>!e.hidden && !e.classList.contains('selected'));const name=label.getAttribute('aria-label').slice(7);label.click();return {name,selected:label.classList.contains('selected'),current:window.theaterViewerState.name};})()`);
  assert.equal(picked.current,picked.name);assert(picked.selected);
  await evaluate(`document.querySelector('.player-name[aria-label="Select Nuzzles"]').click()`);
  await snap('native-theater-raid-timeline-labels');
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:1200,deviceScaleFactor:1,mobile:true});
  await evaluate(`new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve(true))))`);
  await evaluate(`document.querySelector('.viewport').scrollIntoView({block:'start'})`);
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  const smallCards=await evaluate(`[...document.querySelectorAll('.player-name')].filter(e=>!e.hidden).map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,w:r.width,h:r.height};})`);
  assert(smallCards.length>=8);
  await checkCardOcclusion();
  await snap('native-theater-raid-mobile-labels');
  await cdp('Emulation.setDeviceMetricsOverride',{width:1400,height:1100,deviceScaleFactor:1,mobile:false});
  await click('timeline-fit');await evaluate('scrollTo(0,0)');
  state=await scrub(1800);assert.deepEqual(state.players,raidMiddle);
  await click('play');await delay(350);state=await click('play');
  assert(state.time>1800 && state.time<1802);assert(!state.playing);
  await choose('posture/01-crouch-slide');await click('full-mode');
  for(const [t,value] of [[0,null],[22.2,true],[23,false],[27,true],[32.2,false],[37.9,true],[38.5,false],[22.2,true]]) {
    state=await scrub(t);assert.equal(state.players[0].crouch.value,value);
    assert.equal(await evaluate(`document.querySelector('#roster .crouch').classList.contains('active')`),value===true);
  }
  await snap('native-theater-crouch');
  for(const [id,title] of [['appearance/01-cadet-blue','Cadet Blue'],['appearance/02-cadet-brick','Cadet Brick']]) {
    await choose(id);await click('full-mode');state=await scrub(12);
    assert.equal(state.players.length,1);assert.equal(state.players[0].name,'Nuzzles');
    assert(state.players[0].position);assert.equal(state.players[0].aim,null);assert(!state.players[0].firing);
    assert.equal(await evaluate(`document.getElementById('clip').selectedOptions[0].textContent`),title);
  }
  await choose('octagon/03-first-to-50');await click('full-mode');state=await scrub(200);
  assert.equal(state.players.length,2);
  assert.equal(await evaluate(`document.getElementById('clip').selectedOptions[0].textContent`),'Octagon · First to 50');
  const octagon = await evaluate(`(()=>{
    const c=CLIPS.find(c=>c.id==='octagon/03-first-to-50'),input=document.getElementById('timeline');
    const seek=(t,id)=>{input.value=t;input.dispatchEvent(new Event('input'));return window.theaterViewerState.players.find(p=>p.id===id);};
    return {players:c.players.map(p=>{
      const l=p.lives[10];
      return {name:p.name,lives:p.lives.length,positions:p.samples.length,aim:p.aim.length,firing:p.firing.length,shields:p.vitality.shield.length,
        kills:c.events.filter(e=>e.kind==='Kill' && e.player===p.name).length,
        spawn:seek(l.start,p.id),death:seek(l.death+.001,p.id),back:seek(l.start,p.id)};
    })};
  })()`);
  assert.deepEqual(octagon.players.map(p=>[p.name,p.lives,p.kills]),[['Nuzzles',50,49],['timesknightt',50,50]]);
  for(const p of octagon.players){assert(p.positions>9000 && p.aim>7000 && p.firing>250 && p.shields>4500);assert.equal(p.spawn.aim,null);assert(p.death.dead);assert.deepEqual(p.spawn,p.back);}
  // Recorded roster inputs continue across pawn respawns. Nuzzles toggles
  // crouch repeatedly while his recorded position is in the jump arc.
  const crouchCounts = await evaluate(`CLIPS.find(c=>c.id==='octagon/03-first-to-50').players.map(p=>[p.crouch.length,p.crouch.filter(s=>s[2]).length])`);
  assert.deepEqual(crouchCounts,[[17773,520],[10478,3253]]);
  for(const [t,value] of [[149.1,null],[154.84,true],[154.90,false],[154.98,true],[155.15,false],[155.22,true],[155.36,false],[154.98,true]]) {
    state=await scrub(t);
    const p=state.players.find(p=>p.name==='Nuzzles');
    assert.equal(p.crouch.value,value,`Nuzzles crouch at ${t}`);
    if(value!=null) assert.equal(p.life,2);
    assert.equal(await evaluate(`document.querySelector('#roster .crouch').classList.contains('active')`),value===true);
  }
  await snap('native-theater-octagon-crouch');
  state=await scrub(154.14);
  assert.equal(state.players.find(p=>p.name==='Nuzzles').crouch.value,false);
  assert.equal(state.players.find(p=>p.name==='timesknightt').crouch.value,true); // Combined jump/crouch form.
  // Hits must not synthesize scope-out; only the clip's scope stream supplies stages.
  state=await scrub(160.86);
  let nuzzles=state.players.find(p=>p.name==='Nuzzles');
  assert.equal(nuzzles.zoom.level,1);assert.equal(nuzzles.zoom.time,159.822834);
  assert.equal(nuzzles.weapon.name,'S7 Sniper');assert.equal(nuzzles.weapon.slot,1);assert.equal(nuzzles.weapon.ammo,null);
  assert.equal(await evaluate(`document.getElementById('zoom-state').textContent`),'Scope sample 1');
  const recordedOnly = await evaluate(`(()=>{
    const c=CLIPS.find(c=>c.id==='octagon/03-first-to-50');
    return c.players.map(p=>({zoom:p.zoom.length,ammo:p.ammo.length,changes:p.weapon.filter(w=>w[3]==null).length,
      names:[...new Set(p.weapon.filter(w=>w[3]!=null).map(w=>w[3]))]}));
  })()`);
  assert.deepEqual(recordedOnly.map(p=>p.zoom),[67,33]);
  for(const p of recordedOnly){assert.equal(p.ammo,0);assert(p.changes>0);assert.deepEqual(p.names.sort(),['16acdc44d4','30a1992bc4']);}
  state=await scrub(159.06);nuzzles=state.players.find(p=>p.name==='Nuzzles');
  assert.equal(nuzzles.weapon.name,null);assert.equal(nuzzles.zoom.level,null);
  state=await scrub(3.39);nuzzles=state.players.find(p=>p.name==='Nuzzles');
  assert.equal(nuzzles.weapon.name,null);assert.equal(nuzzles.weapon.ammo,null);
  await scrub(200);await snap('native-theater-octagon-gameplay');
  await choose('maps/01-bazaar-idle');await click('full-mode');state=await scrub(10);
  assert.equal(state.players.length,1);assert.equal(state.players[0].aim,null);
  assert.equal(await evaluate(`document.getElementById('clip').selectedOptions[0].textContent`),'Bazaar idle');
  state=await scrub(42);assert(state.players[0].aim);assert.equal(state.players[0].vitality.body,null);
  await choose('ranked-arena/02-oddball');await click('full-mode');state=await scrub(1188);
  assert.equal(state.players.length,8);assert(state.players.filter(p=>p.life>=256).length>=6);
  const audit=await evaluate(`(()=>{
    const c=CLIPS.find(c=>c.id==='ranked-arena/02-oddball'),p=c.players[0],input=document.getElementById('timeline');
    const seek=t=>{input.value=t;input.dispatchEvent(new Event('input'));return window.theaterViewerState.players.find(v=>v.id===p.id);};
    const l=p.lives[1],spawn=seek(l.start),death=seek(l.death+.001),back=seek(l.start);
    return {count:c.players.reduce((n,p)=>n+p.firing.length,0),spawn,death,back};
  })()`);
  assert.equal(audit.count,3360);assert.equal(audit.spawn.aim,null);assert.equal(audit.spawn.vitality.body,null);assert.equal(audit.spawn.vitality.shield,null);assert(!audit.spawn.firing);assert(audit.death.dead);assert.deepEqual(audit.spawn,audit.back);
  assert.equal(audit.spawn.weapon.ammo,null);assert.equal(audit.death.weapon.ammo,null);
  // Multiplayer magazines come from recorded components, including refills;
  // missing shot updates hold the last quantity rather than counting down.
  const arenaAmmo=await evaluate(`CLIPS.find(c=>c.id==='ranked-arena/02-oddball').players.map(p=>p.ammo.length)`);
  assert.equal(arenaAmmo.reduce((a,b)=>a+b,0),2306);assert(arenaAmmo.every(n=>n>0));
  for(const [t,rounds] of [[30.04056,14],[30.424402,13],[30.975107,12],[31.408623,11],[32.377023,11],[36.113054,15],[30.04056,14]]) {
    state=await scrub(t);const p=state.players.find(p=>p.name==='Nuzzles');
    assert.equal(p.weapon.name,'Bandit EVO');assert.equal(p.weapon.ammo?.raw??null,rounds,`Nuzzles magazine at ${t}`);
    if(t===32.377023){assert(p.weapon.ammo.stale);assert.equal(p.weapon.ammo.time,31.408623);}
  }
  state=await scrub(666.102668);assert.equal(state.players.find(p=>p.name==='Yet').weapon.ammo.raw,0);
  await scrub(36.113054);await snap('native-theater-oddball-ammo');
  await choose('bandit/01-evo');await click('full-mode');state=await scrub(29.945961);
  assert.equal(state.players.find(p=>p.name==='Nuzzles').weapon.ammo.raw,14);
  // The same BR75 fingerprint is recorded in either carried slot.
  for(const [t,name,slot,window] of [[332.933079,'Miiindful',0,'12b1824d54'],[582.000242,'Yet',1,'32b1824d54']]) {
    state=await scrub(t);const w=state.players.find(p=>p.name===name).weapon;
    assert.equal(w.name,'BR75');assert.equal(w.slot,slot);assert.equal(w.window,window);
  }
  await snap('native-theater-br75');
  await choose('weapons/04-br-shock-rifle');await click('full-mode');
  assert.equal(await evaluate(`document.getElementById('clip').selectedOptions[0].textContent`),'BR75 & Shock Rifle');
  const weaponControl=await evaluate(`(()=>{const p=CLIPS.find(c=>c.id==='weapons/04-br-shock-rifle').players[0];return {shots:p.firing.length,ammo:p.ammo.length,switches:p.switch};})()`);
  assert.equal(weaponControl.shots,0);assert.equal(weaponControl.ammo,0);
  assert.deepEqual(weaponControl.switches.map(s=>[s[0],s[2]]),[[57.01143,1],[66.971899,0]]);
  // Spawn reference evidence does not supply a decoded initial inventory.
  for(const [t,slot] of [[10,null],[57.01143,1],[66.971899,0],[57.01143,1]]) {
    state=await scrub(t);const w=state.players[0].weapon;
    assert.equal(w.slot,slot);assert.equal(w.name,null);assert.equal(w.ammo,null);
  }
  await snap('native-theater-br-shock-control');
  assert.equal(state.armor.name,'Nuzzles');
  assert.equal(state.armor.items.find(i=>i.kind==='chest').name,'TAC/Packrat Rig');
  await choose('weapons/05-ar-stalker-rifle');await click('full-mode');
  assert.equal(await evaluate(`document.getElementById('clip').selectedOptions[0].textContent`),'AR & Stalker Rifle');
  const stalkerControl=await evaluate(`(()=>{const p=CLIPS.find(c=>c.id==='weapons/05-ar-stalker-rifle').players[0];return {shots:p.firing.length,ammo:p.ammo.map(s=>s[3]),switches:p.switch.map(s=>[s[0],s[2]])};})()`);
  assert.deepEqual(stalkerControl,{shots:4,ammo:[35,34,33],switches:[[56.957497,1],[67.050945,0]]});
  for(const [t,slot,name,ammo] of [[10,null,null,null],[56.957497,1,null,null],[59.310053,1,'Stalker Rifle',null],[67.050945,0,null,null],[68.569291,0,'Assault Rifle',35],[68.652670,0,'Assault Rifle',34],[68.736064,0,'Assault Rifle',33],[59.310053,1,'Stalker Rifle',null]]) {
    state=await scrub(t);const p=state.players[0];
    assert.equal(p.weapon.slot,slot);assert.equal(p.weapon.name,name);assert.equal(p.weapon.ammo?.raw??null,ammo);
    if(name)assert(p.firing,`Missing firing indicator at ${t}`);
  }
  await snap('native-theater-stalker-control');
  await scrub(68.736064);await snap('native-theater-ar-control');
  const armorExpected={armor:'Mark VII',coating:'Cadet Brick',helmet:'CAVALLINO',visor:'Arcadian Green',gloves:'Capaxx',knees:'UA/Type SA',chest:'TAC/Packrat Rig',left_shoulder:'Alpha Augmentor',right_shoulder:'Alpha Augmentor',wrist:'TAC/Holodyne Milspec',utility:'Myesel Ammo Pouch',mythic:'Beyond the Burrow'};
  assert.deepEqual(Object.fromEntries(state.armor.items.map(i=>[i.kind,i.name])),armorExpected);
  assert.equal(await evaluate(`document.querySelectorAll('#armor-panel').length`),1);
  assert.equal(await evaluate(`document.querySelectorAll('#roster [data-slot],.player-labels [data-slot]').length`),0);
  await evaluate(`document.getElementById('armor-panel').scrollIntoView({block:'center'})`);
  await snap('native-theater-selected-armor');
  state=await scrub(0);assert.equal(state.armor.value,null);
  assert(await evaluate(`document.getElementById('armor-empty').hidden===false && document.getElementById('armor-identifiers').textContent===''`));
  state=await scrub(10);assert.deepEqual(Object.fromEntries(state.armor.items.map(i=>[i.kind,i.name])),armorExpected);
  await choose('appearance/01-cadet-blue');await click('full-mode');state=await scrub(10);
  assert.equal(state.armor.items.find(i=>i.kind==='coating').name,'Cadet Blue');
  assert.equal(state.armor.items.find(i=>i.kind==='chest').name,null);
  await choose('octagon/02-ar-kill');await click('full-mode');state=await scrub(25);
  for(const index of [1,0,1]) {
    state=await evaluate(`document.querySelectorAll('#roster>button')[${index}].click();window.theaterViewerState`);
    assert.equal(state.armor.player,state.players[index].id);assert.equal(state.armor.name,state.players[index].name);
    assert.equal(await evaluate(`document.getElementById('armor-player').textContent`),state.players[index].name);
    const recorded=await evaluate(`CLIPS.find(c=>c.id==='octagon/02-ar-kill').players[${index}].appearance.findLast(s=>s[0]<=window.theaterViewerState.time)?.[1]??null`);
    assert.deepEqual(state.armor.value,recorded);
  }
  state=await scrub(23.5);const armorBeforeDeath=state.armor.value;
  state=await scrub(25);assert.deepEqual(state.armor.value,armorBeforeDeath); // Appearance persists independently of lives.
  await evaluate(`document.querySelector('.viewport').scrollIntoView({block:'start'})`);
  await choose('raids/01-hour-long-raid');await click('full-mode');state=await scrub(491.415079);
  const raidStalker=state.players.find(p=>p.name==='Jared Teller').weapon;
  assert.equal(raidStalker.name,'Stalker Rifle');assert.equal(raidStalker.slot,1);
  await choose('weapons/03-br-sniper-zoom');await click('full-mode');
  for(const [t,level] of [[22.04,1],[26.68,0],[32.09,1],[36.81,2],[41.90,0]]){state=await scrub(t);assert.equal(state.players[0].zoom.level,level);}
  await choose('weapons/02-reload-comparison');await click('full-mode');state=await scrub(39.34);assert.equal(state.players[0].weapon.ammo.raw,0);
  state=await scrub(39.38);assert(state.players[0].reload);state=await scrub(40.18);assert.equal(state.players[0].weapon.ammo.raw,12);
  await choose('natural-end/09-grenade');await click('full-mode');
  const projectile=await evaluate(`CLIPS.find(c=>c.id==='natural-end/09-grenade').projectiles[0]`);
  assert.equal(projectile.samples.length,87);state=await scrub(projectile.start+.05);assert(state.projectiles[0].visible);
  await choose('octagon/02-ar-kill');await click('full-mode');
  const health=await evaluate(`(()=>{const p=CLIPS.find(c=>c.id==='octagon/02-ar-kill').players.find(p=>p.vitality.shield.length);return {id:p.id,row:p.vitality.shield[5]};})()`);
  state=await scrub(health.row[0]);assert.equal(state.players.find(p=>p.id===health.id).vitality.shield.raw,health.row[1]);
  await snap('native-theater-health');
  // Exercise the actual browser file picker using the typed JSON, not the scene.
  const doc=await cdp('DOM.getDocument');const input=await cdp('DOM.querySelector',{nodeId:doc.root.nodeId,selector:'#film-file'});
  await cdp('DOM.setFileInputFiles',{nodeId:input.nodeId,files:[path.resolve(__dirname,'../films/weapons/03-br-sniper-zoom/decoded-film.json')]});
  for(let n=0;n<100;n++){if(await evaluate(`document.getElementById('film-import-status').textContent==='Film opened'`))break;await delay(50);}
  assert.equal(await evaluate(`document.getElementById('film-import-status').textContent`),'Film opened');
  await click('full-mode');state=await scrub(36.81);assert.equal(state.players[0].zoom.level,2);
  // Aquarius now imports its single recorded spawn and no invented movement.
  await cdp('DOM.setFileInputFiles',{nodeId:input.nodeId,files:[path.resolve(__dirname,'../films/maps/02-aquarius/decoded-film.json')]});
  for(let n=0;n<100;n++){if(await evaluate(`document.getElementById('film-import-status').textContent==='Film opened'`))break;await delay(50);}
  assert.equal(await evaluate(`document.getElementById('film-import-status').textContent`),'Film opened');
  assert.deepEqual(await evaluate(`CLIPS.find(c=>c.match_id==='16d67b8c-09c2-4ab8-8b94-892f7edb6212').players[0].samples[0].slice(1,4)`),[1980,2469,727]);
  assert.equal(await evaluate('CLIPS.length'),32);
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:1200,deviceScaleFactor:1,mobile:true});await delay(150);assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  assert(await evaluate(`['.clip-picker', '.film-import', '#film-import-status'].every(selector => {const r=document.querySelector(selector).getBoundingClientRect();return r.left>=0 && r.right<=innerWidth;})`));
  await evaluate(`document.getElementById('armor-panel').scrollIntoView({block:'center'})`);
  assert(await evaluate(`document.getElementById('armor-panel').getBoundingClientRect().right<=innerWidth`));
  await snap('native-theater-armor-mobile');
  await snap('native-theater-import');assert.deepEqual(errors,[]);
  console.log('PASS: timeline zoom/pan/precision/auto-follow, anchored player cards with occlusion-only fading and stacking by camera distance, selected-player armor with recorded accessory/mythic names and unknown states, and 32 native films including Stalker/AR firing, AR ammo, both switches and raid Stalker naming, the BR75/Shock switch control, BR75 naming in either slot, the 20-player hour-long raid, generation-4 respawns and late/reverse seeking, Octagon crouch toggles during jumps, two-player input/respawn binding, crouch-control holds/releases/reverse seek, both High Ground appearance controls, full Octagon gameplay, death/reverse seek, Bazaar, Aquarius spawn import, vitality, scope, ammo/reload, projectile path, JSON import, mobile layout');
 } finally {chrome.kill();fs.rmSync(profile,{recursive:true,force:true});}
})().catch(error=>{console.error(error);process.exitCode=1;});
