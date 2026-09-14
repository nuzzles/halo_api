/* Focused recorded velocity replay checks. Requires a rebuilt local corpus. */
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
  await cdp('Runtime.enable');await cdp('Page.enable');
  await cdp('Emulation.setDeviceMetricsOverride',{width:1400,height:1100,deviceScaleFactor:1,mobile:false});
  await cdp('Page.navigate',{url:pathToFileURL(path.resolve(__dirname,'../films/analysis/theater_viewer.html')).href});
  let state;for(let n=0;n<500;n++){state=await evaluate('window.theaterViewerState');if(state)break;await delay(100);}
  assert(state);
  await evaluate(`document.getElementById('timeline').step='any'`);
  const choose = id => evaluate(`document.getElementById('clip').value=${JSON.stringify(id)};document.getElementById('clip').dispatchEvent(new Event('change'));document.getElementById('full-mode').click();window.theaterViewerState`);
  for(const id of ['controller/01-stick-circles','maps/01-bazaar-idle','octagon/03-first-to-50','ranked-arena/02-oddball','raids/01-hour-long-raid']) {
    await choose(id);
    const observation=await evaluate(`(()=>{
      const c=CLIPS.find(c=>c.id===${JSON.stringify(id)}),p=c.players.find(p=>p.id===c.selectedPlayer);
      const r=p.velocity.find(r=>r.length===7&&r[5]>150&&p.samples.some(s=>s[0]===r[0]&&s[6]===r[1]));
      return r;
    })()`);
    assert(observation,id+' has a simultaneous recorded direction and position');
    const expected={form:'Directed',direction:observation.slice(2,5),magnitude_code:observation[5],direction_code:observation[6],speed:Math.exp((observation[5]+.5)*Math.log(350.97)/1024)-.97};
    state=await scrub(observation[0]);assert.deepEqual(state.velocity.value,expected);assert(state.velocityVisible,id);assert(!state.velocity.stale);
    const position=state.position;
    state=await click('velocity-toggle');assert(!state.velocityVisible);assert.deepEqual(state.position,position);assert.deepEqual(state.velocity.value,expected);
    state=await click('velocity-toggle');assert(state.velocityVisible);
    if(id==='controller/01-stick-circles'){
      await snap('theater-velocity-circles');
      const bounds=await evaluate(`(()=>{const a=document.querySelector('.position-card').getBoundingClientRect(),b=document.querySelector('.view-tools').getBoundingClientRect();return {cardBottom:a.bottom,toolsTop:b.top,height:a.height};})()`);
      assert(bounds.cardBottom < bounds.toolsTop, 'Readout leaves camera controls accessible');
    }
  }
  for (const id of ['bandit/01-evo','ranked-arena/02-oddball']) {
    await choose(id);
    const track=await evaluate(`CLIPS.find(c=>c.id===${JSON.stringify(id)}).projectiles[0]`);
    assert(track.samples.length>=3); assert(track.velocity.length>=3);
    state=await scrub(track.samples[1][0]);
    const projectile=state.projectiles.find(p=>p.id===track.id);
    assert(projectile.visible); assert(projectile.speed>0); assert.equal(projectile.player,track.player);
    await snap('theater-projectile-'+id.split('/')[0]);
    state=await scrub(track.end+.101);assert(!state.projectiles.find(p=>p.id===track.id).visible);
  }
  await choose('maps/02-aquarius'); state=await scrub(3.04); assert.equal(state.players.length,1);
  assert.deepEqual(await evaluate(`CLIPS.find(c=>c.id==='maps/02-aquarius').players[0].samples[0].slice(1,4)`),[1980,2469,727]);
  await choose('natural-end/02-jump');
  const jump=await evaluate(`CLIPS.find(c=>c.id==='natural-end/02-jump').players[0].velocity`);
  for(const sign of [1,-1]){
    const row=jump.find(r=>r.length===7&&r[4]===sign);
    state=await scrub(row[0]);assert.equal(state.velocity.value.direction[2],sign);assert(state.velocityVisible);
  }
  const zero=jump.find(r=>r.length===7&&r[5]===0);
  assert(zero);state=await scrub(zero[0]);assert.equal(state.velocity.value.form,'Directed');
  const stopped=jump.find(r=>r.length===2);
  state=await scrub(stopped[0]);assert.equal(state.velocity.value.form,'Stationary');assert(!state.velocityVisible);
  state=await scrub(jump.at(-1)[0]+1);assert(state.velocity.stale);assert(!state.velocityVisible);
  assert.match(await evaluate(`document.getElementById('velocity-observation').textContent`),/Last sample/);
  await choose('octagon/03-first-to-50');
  const lives=await evaluate(`(()=>{const c=CLIPS.find(c=>c.id==='octagon/03-first-to-50');return c.players.find(p=>p.id===c.selectedPlayer).lives;})()`);
  const life=lives.find(l=>l.death!=null);
  state=await scrub(life.death);assert.equal(state.velocity.value,null);assert(!state.velocityVisible);
  const next=lives.find(l=>l.start>life.death);state=await scrub(next.start);assert(!state.velocityVisible);assert.equal(state.velocity.value,null);
  await choose('natural-end/01-do-nothing');state=await scrub(0);assert.equal(state.velocity.value,null);assert(!state.velocityVisible);
  // Old typed exports with no velocity stream remain unknown in the adapter.
  assert.deepEqual(await evaluate(`filmToClip({schema_version:1,major_version:41,duration_us:1000,players:[{id:0,name:'Old',lives:[],positions:[{time_us:0,life:0,value:{raw:[0,0,0]}}],aim:[],firing:[],melee:[],grenades:[],reloads:[],magazines:[],selections:[],weapons:[],zoom:[],body:[],shields:[]}],summary_events:[],projectiles:[]}).players[0].velocity`),[]);
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true});await delay(200);
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  assert.equal(errors.length,0,JSON.stringify(errors));
  console.log('PASS: recorded directions across maps/raid, toggle without position changes, jump apex, explicit stops, gaps, death/respawn, old exports and mobile layout');
 }finally{
  const stopped=new Promise(resolve=>chrome.once('exit',resolve));chrome.kill('SIGTERM');
  const force=setTimeout(()=>chrome.kill('SIGKILL'),5000);await stopped;clearTimeout(force);
  for(const p of pending.values())clearTimeout(p.timer);
  fs.rmSync(profile,{recursive:true,force:true,maxRetries:3,retryDelay:100});
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
