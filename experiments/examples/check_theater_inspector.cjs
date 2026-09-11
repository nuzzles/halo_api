/* Start theater_lab.py first. Optional CHROME_PATH and THEATER_LAB_URL. */
const {spawn} = require('node:child_process');
const fs = require('node:fs');
const assert = require('node:assert/strict');
const os = require('node:os');
const path = require('node:path');
const profile = fs.mkdtempSync(path.join(os.tmpdir(),'halo-inspector-chrome-'));
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
function cdp(method,params={},sid=session){return new Promise((resolve,reject)=>{const id=++nextId;const timer=setTimeout(()=>reject(new Error('Timed out: '+method)),20000);pending.set(id,{resolve,reject,timer});chrome.stdio[3].write(JSON.stringify({id,method,params,...(sid?{sessionId:sid}:{})})+'\0');});}
const delay=ms=>new Promise(r=>setTimeout(r,ms));
async function evaluate(expression){const r=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw new Error(JSON.stringify(r.exceptionDetails));return r.result.value;}
async function snap(name){await delay(150);const r=await cdp('Page.captureScreenshot',{format:'png'});fs.writeFileSync(path.join(os.tmpdir(),name+'.png'),Buffer.from(r.data,'base64'));}
(async()=>{
 try {
  const target=await cdp('Target.createTarget',{url:'about:blank'},null);session=(await cdp('Target.attachToTarget',{targetId:target.targetId,flatten:true},null)).sessionId;
  await cdp('Runtime.enable');await cdp('Page.enable');await cdp('Emulation.setDeviceMetricsOverride',{width:1600,height:1250,deviceScaleFactor:1,mobile:false});
  await cdp('Page.navigate',{url:process.env.THEATER_LAB_URL || 'http://127.0.0.1:8766/'});
  async function waitFor(expression){for(let n=0;n<200;n++){const value=await evaluate(expression);if(value)return value;await delay(100);}throw Error('Timed out waiting for '+expression);}
  let state=await waitFor('window.theaterInspectorState?.view && window.theaterInspectorState');
  assert.equal(state.film,'ranked-arena/02-oddball');assert.equal(await evaluate('document.querySelectorAll("#film option").length'),23);
  assert(state.view.spans.some(s=>s.label==='X raw'));assert.equal(state.view.issues.length,0);
  assert.equal(Object.values(state.view.coverage).reduce((a,b)=>a+b),state.view.scope[1]-state.view.scope[0]);
  const field=state.view.spans.find(s=>s.label==='X raw');
  await evaluate(`document.querySelector('.field[data-start="${field.start}"]').click()`);
  state=await waitFor(`window.theaterInspectorState.selected.start===${field.start} && window.theaterInspectorState`);
  assert.equal(state.selected.end-state.selected.start,18);
  assert.equal(await evaluate('document.querySelectorAll(".byte.active").length'),2*(Math.ceil(field.end/8)-Math.floor(field.start/8)));
  const bit=await evaluate('Number(document.querySelector(".bit").dataset.bit)');
  await evaluate('document.querySelector(".bit").click()');state=await evaluate('window.theaterInspectorState');assert.equal(state.selected.end-state.selected.start,1);
  await evaluate('document.querySelector(".ascii-bytes .byte").click()');state=await evaluate('window.theaterInspectorState');assert.equal(state.selected.end-state.selected.start,8);
  assert(await evaluate('document.querySelectorAll(".byte.mixed").length>0'));
  await snap('theater-inspector-desktop');
  await evaluate(`document.getElementById('field-filter').value='unparsed';document.getElementById('field-filter').dispatchEvent(new Event('change'))`);
  assert(await evaluate(`Array.from(document.querySelectorAll('.field')).every(x=>x.dataset.status==='unparsed')`));
  await evaluate(`document.getElementById('field-filter').value='all';document.getElementById('field-filter').dispatchEvent(new Event('change'))`);
  await evaluate(`document.getElementById('chunk').value='0';document.getElementById('chunk').dispatchEvent(new Event('change'))`);
  await waitFor('window.theaterInspectorState.chunk===0');
  await evaluate(`document.getElementById('search').value='object-body-vitality-component';document.getElementById('search-form').requestSubmit()`);
  state=await waitFor(`window.theaterInspectorState.view.spans.some(s=>s.value==='object-body-vitality-component') && window.theaterInspectorState`);
  assert(state.view.spans.some(s=>s.status==='unparsed'));assert.equal(state.selected.end-state.selected.start,'object-body-vitality-component'.length*8);
  assert.match(await evaluate('document.getElementById("source").textContent'),/decode_registry/);
  await snap('theater-inspector-registry');
  await evaluate(`document.getElementById('search-mode').value='hex';document.getElementById('search').value='zz';document.getElementById('search-form').requestSubmit()`);
  await waitFor('document.getElementById("notice").classList.contains("error")');
  await evaluate(`document.getElementById('offset').value='0x00000008';document.getElementById('offset-form').requestSubmit()`);
  await waitFor('window.theaterInspectorState.view.offset===8');
  await evaluate(`document.getElementById('time').value='1188';document.getElementById('time-form').requestSubmit()`);
  state=await waitFor('window.theaterInspectorState.view.packet?.time>1187 && window.theaterInspectorState');assert.equal(state.view.issues.length,0);
  await evaluate(`document.getElementById('film').value='octagon/02-ar-kill';document.getElementById('film').dispatchEvent(new Event('change'))`);
  await waitFor(`window.theaterInspectorState.film==='octagon/02-ar-kill'`);
  await evaluate(`document.getElementById('time').value='24.086002';document.getElementById('time-form').requestSubmit()`);
  state=await waitFor('Math.abs(window.theaterInspectorState.view.packet?.time-24.086002)<.001 && window.theaterInspectorState');
  assert(state.view.spans.some(s=>s.label==='Shields raw'&&s.value===0));assert.equal(state.view.issues.length,0);
  await snap('theater-inspector-vitality');

  const combatCases=[];
  for(const kind of ['firing','melee']){
    const rows=JSON.parse(fs.readFileSync(path.resolve(__dirname,'../films/analysis/'+kind+'/evidence.json'))).records;
    for(const film of ['natural-end/13-melee','bandit/01-evo','ranked-arena/02-oddball']){
      const row=rows.findLast(r=>r.film===film);
      if(row)combatCases.push({kind,...row});
    }
  }
  const combatViews=await evaluate(`(async()=>{
    const results=[];
    for(const row of ${JSON.stringify(combatCases)}){
      const q=new URLSearchParams({film:row.film,chunk:row.chunk,offset:row.payload_byte});
      const view=await (await fetch('/api/view?'+q)).json();
      results.push({kind:row.kind,film:row.film,serial:row.serial,view});
    }
    return results;
  })()`);
  for(const {kind,view,serial} of combatViews){
    assert.deepEqual(view.issues,[]);
    assert(view.spans.some(s=>s.label===(kind==='melee'?'Melee signature':'Event signature')));
    assert(view.spans.some(s=>s.label==='Generation tag'&&s.value===(serial>=256?2:1)));
    assert(view.spans.some(s=>s.label==='Weapon window (opaque)'&&s.status==='opaque'));
    assert.equal(Object.values(view.coverage).reduce((a,b)=>a+b),view.scope[1]-view.scope[0]);
  }


  const grenadeProof=JSON.parse(fs.readFileSync(path.resolve(__dirname,'../films/analysis/grenades/evidence.json')));
  const grenadeCases=[grenadeProof.records.find(r=>r.film==='natural-end/09-grenade'),grenadeProof.records.findLast(r=>r.film==='ranked-arena/02-oddball'),...['spawn','position','terminal'].map(kind=>grenadeProof.projectiles.find(r=>r.kind===kind && r.film==='natural-end/09-grenade'))];
  const grenadeViews=await evaluate(`(async()=>{
    const views=[];
    for(const row of ${JSON.stringify(grenadeCases)}){
      const q=new URLSearchParams({film:row.film,chunk:row.chunk,offset:row.payload_byte});
      views.push(await (await fetch('/api/view?'+q)).json());
    }
    return views;
  })()`);
  for(const view of grenadeViews){assert.deepEqual(view.issues,[]);assert.equal(Object.values(view.coverage).reduce((a,b)=>a+b),view.scope[1]-view.scope[0]);}
  assert(grenadeViews[0].spans.some(s=>s.label==='Grenade throw signature'));
  assert(grenadeViews[1].spans.some(s=>s.label==='Roster index'));
  assert(grenadeViews[2].spans.some(s=>s.label==='Grenade X raw'));
  assert(grenadeViews[3].spans.some(s=>s.label==='Grenade Z raw'));
  assert(grenadeViews[4].spans.some(s=>s.label==='Projectile terminal signature'));

  // Exercise the HTTP boundary independently: path restrictions, invalid hex,
  // page limits, unknown packet payloads and read-only methods.
  const apiAudit=await evaluate(`(async()=>{
    const prefix='/api/view?film=octagon%2F02-ar-kill&chunk=2&';
    const bad=await Promise.all([fetch('/api/film?film=../outside'),fetch(prefix+'offset=-1'),fetch(prefix+'offset=0&count=99999'),fetch('/api/catalog',{method:'POST'})]);
    const c=await (await fetch('/api/chunk?film=octagon%2F02-ar-kill&chunk=2')).json();
    const packet=c.packets.find(p=>p.kind===10),v=await (await fetch(prefix+'offset='+packet.offset)).json();
    return {statuses:bad.map(r=>r.status),coverage:v.coverage,size:v.packet.size};
  })()`);
  assert.deepEqual(apiAudit.statuses,[400,400,400,501]);assert.equal(apiAudit.coverage.unparsed,apiAudit.size*8);
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:1100,deviceScaleFactor:1,mobile:true});await snap('theater-inspector-mobile');
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  assert(await evaluate(`(()=>{const a=document.querySelector('.hex-scroll');return a.scrollWidth>a.clientWidth})()`));
  await cdp('Emulation.setDeviceMetricsOverride',{width:1400,height:1100,deviceScaleFactor:1,mobile:false});
  await evaluate(`document.querySelector('nav a[href="/replay"]').click()`);
  await waitFor(`window.theaterViewerState?.clip==='ranked-arena/02-oddball'`);
  assert.equal(await evaluate(`document.querySelector('.experiment-nav a:not([aria-current])').getAttribute('href')`),'/');
  await evaluate(`document.querySelector('.experiment-nav a:not([aria-current])').click()`);
  await waitFor(`window.theaterInspectorState?.film==='ranked-arena/02-oddball'`);
  await evaluate(`document.getElementById('film').value='weapons/02-reload-comparison';document.getElementById('film').dispatchEvent(new Event('change'))`);
  await waitFor(`window.theaterInspectorState.film==='weapons/02-reload-comparison'`);
  for(const [t,label,value] of [[26.875065,'Reload start guard','0001000'],[39.371112,'Reload start guard','0001000'],[39.33774,'Magazine rounds',0],[40.171841,'Magazine rounds',12],[32.246683,'Selected weapon slot',1]]){
    await evaluate(`document.getElementById('time').value='${t}';document.getElementById('time-form').requestSubmit()`);
    state=await waitFor(`Math.abs(window.theaterInspectorState.view.packet?.time-${t})<.00001 && window.theaterInspectorState`);
    assert(state.view.spans.some(s=>s.label===label&&s.value===value));assert.equal(state.view.issues.length,0);
    assert(state.view.coverage.unparsed>0);assert.equal(Object.values(state.view.coverage).reduce((a,b)=>a+b),state.view.scope[1]-state.view.scope[0]);
  }
  await snap('theater-inspector-weapons');
  console.log('PASS reload starts, variable-width empty magazine, refill and selection byte annotations.');
  await evaluate(`document.getElementById('film').value='weapons/03-br-sniper-zoom';document.getElementById('film').dispatchEvent(new Event('change'))`);
  await waitFor(`window.theaterInspectorState.film==='weapons/03-br-sniper-zoom'`);
  for(const [t,level] of [[22.036707,1],[26.674662,0],[32.080040,1],[36.801706,2],[41.889813,0]]){
    await evaluate(`document.getElementById('time').value='${t}';document.getElementById('time-form').requestSubmit()`);
    state=await waitFor(`Math.abs(window.theaterInspectorState.view.packet?.time-${t})<.00001 && window.theaterInspectorState`);
    const field=state.view.spans.find(s=>s.label==='Zoom stage');assert(field);assert.equal(field.value,level);assert.equal(field.end-field.start,2);
    assert.equal(state.view.issues.length,0);assert(state.view.coverage.unparsed>0);assert.equal(Object.values(state.view.coverage).reduce((a,b)=>a+b),state.view.scope[1]-state.view.scope[0]);
    await evaluate(`document.querySelector('.field[data-start="${field.start}"]').click()`);
    assert.equal(await evaluate('window.theaterInspectorState.selected.end-window.theaterInspectorState.selected.start'),2);
  }
  await snap('theater-inspector-scope');console.log('PASS all five scope-stage byte annotations and two-bit selection.');
  assert.equal(errors.length,0,JSON.stringify(errors));
  console.log('PASS: 23 recordings; Oddball fields; exact bit coverage; linked field/hex/ASCII/bit selection; unknown filtering; registry text search; invalid hex handling; offset/time navigation; post-wrap frames; zero shield value; firing/melee annotations including reused IDs and opaque weapon fields; API validation and read-only methods; desktop/mobile layout; navigation to the replay and back; no runtime errors.');
 }finally{chrome.kill('SIGTERM');for(const p of pending.values())clearTimeout(p.timer);}
})().catch(e=>{console.error(e);process.exitCode=1;});
