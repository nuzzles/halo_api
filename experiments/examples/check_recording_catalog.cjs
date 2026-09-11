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
function cdp(method,params={},sid=session){return new Promise((resolve,reject)=>{const id=++nextId;const timer=setTimeout(()=>reject(new Error('Timed out: '+method)),120000);pending.set(id,{resolve,reject,timer});chrome.stdio[3].write(JSON.stringify({id,method,params,...(sid?{sessionId:sid}:{})})+'\0');});}
const delay=ms=>new Promise(r=>setTimeout(r,ms));
async function evaluate(expression){const r=await cdp('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw new Error(JSON.stringify(r.exceptionDetails));return r.result.value;}
async function snap(name){await delay(150);const r=await cdp('Page.captureScreenshot',{format:'png'});fs.writeFileSync(path.join(os.tmpdir(),name+'.png'),Buffer.from(r.data,'base64'));}
(async()=>{
 try {
  const target=await cdp('Target.createTarget',{url:'about:blank'},null);
  session=(await cdp('Target.attachToTarget',{targetId:target.targetId,flatten:true},null)).sessionId;
  await cdp('Runtime.enable');await cdp('Page.enable');
  await cdp('Emulation.setDeviceMetricsOverride',{width:1400,height:1000,deviceScaleFactor:1,mobile:false});
  const base=process.env.THEATER_LAB_URL || 'http://127.0.0.1:8766/';
  async function waitFor(expression){for(let n=0;n<1000;n++){const value=await evaluate(expression);if(value)return value;await delay(100);}throw Error('Timed out waiting for '+expression);}
  const menu=id=>evaluate(`[...document.getElementById(${JSON.stringify(id)}).querySelectorAll('option')].map(o=>[o.value,o.textContent,o.parentElement.label])`);
  await cdp('Page.navigate',{url:base+'?film=weapons/05-ar-stalker-rifle'});
  await waitFor('window.theaterInspectorState?.film==="weapons/05-ar-stalker-rifle" && window.theaterInspectorState.view');
  const inspectorMenu=await menu('film');assert.equal(inspectorMenu.length,32);
  assert.deepEqual(inspectorMenu.find(r=>r[0]==='weapons/05-ar-stalker-rifle'),['weapons/05-ar-stalker-rifle','AR & Stalker Rifle','Weapon controls']);
  assert(inspectorMenu.some(r=>r[0]==='maps/02-aquarius'));
  await evaluate('document.querySelector("nav a:not([aria-current])").click()');
  await waitFor('window.theaterViewerState?.clip==="weapons/05-ar-stalker-rifle" && window.theaterViewerState.players.length===1');
  assert.deepEqual(await menu('clip'),inspectorMenu);
  assert.equal(await evaluate('document.getElementById("clip").value'),'weapons/05-ar-stalker-rifle');
  // Aquarius now has a recorded spawn and shares selection with the inspector.
  await evaluate('document.getElementById("clip").value="maps/02-aquarius";document.getElementById("clip").dispatchEvent(new Event("change"))');
  await waitFor('window.theaterViewerState?.clip==="maps/02-aquarius" && !window.theaterViewerState.loading');
  assert.equal(await evaluate('window.theaterViewerState.players.length'),1);
  assert(await evaluate('!document.querySelector(".viewport").hidden'));
  await snap('theater-shared-catalog-aquarius');
  await evaluate('document.querySelector("nav a:not([aria-current])").click()');
  await waitFor('window.theaterInspectorState?.film==="maps/02-aquarius" && window.theaterInspectorState.view');
  assert.equal(await evaluate('document.getElementById("film").value'),'maps/02-aquarius');
  assert.deepEqual(await menu('film'),inspectorMenu);
  await evaluate('document.querySelector("nav a:not([aria-current])").click()');
  await waitFor('window.theaterViewerState?.clip==="maps/02-aquarius" && !window.theaterViewerState.loading');
  // A catalog entry missing from the embedded build can load the current local export.
  await evaluate('CLIPS.splice(CLIPS.findIndex(c=>c.id==="weapons/05-ar-stalker-rifle"),1);document.getElementById("clip").value="weapons/05-ar-stalker-rifle";document.getElementById("clip").dispatchEvent(new Event("change"))');
  await waitFor('window.theaterViewerState?.clip==="weapons/05-ar-stalker-rifle" && window.theaterViewerState.players.length===1');
  assert(await evaluate('!document.querySelector(".viewport").hidden && document.getElementById("replay-message").hidden'));
  assert.equal(await evaluate('new URL(location.href).searchParams.get("clip")'),'weapons/05-ar-stalker-rifle');
  assert.deepEqual(await menu('clip'),inspectorMenu);
  await evaluate('document.getElementById("timeline").value=69;document.getElementById("timeline").dispatchEvent(new Event("input"))');
  assert.equal(await evaluate('window.theaterViewerState.players[0].weapon.name'),'Assault Rifle');
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:1100,deviceScaleFactor:1,mobile:true});
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  await snap('theater-shared-catalog-mobile');
  assert.deepEqual(errors,[]);
  console.log('PASS: identical 32-recording menus, canonical titles/categories/order, both navigation directions, Aquarius idle spawn, current local export loading and mobile layout');
 } finally {
  const stopped=new Promise(resolve=>chrome.once('exit',resolve));chrome.kill('SIGTERM');await stopped;
  for(const p of pending.values())clearTimeout(p.timer);
  fs.rmSync(profile,{recursive:true,force:true,maxRetries:3,retryDelay:100});
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
