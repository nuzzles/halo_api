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
  await cdp('Page.navigate',{url:base+'?film=natural-end/03-shoot'});
  async function waitFor(expression){for(let n=0;n<1200;n++){const value=await evaluate(expression);if(value)return value;await delay(100);}throw Error('Timed out waiting for '+expression);}
  async function complete(label){return waitFor(`window.theaterInspectorState?.film===${JSON.stringify(label)} && window.theaterInspectorState?.recordingCoverage?.complete && window.theaterInspectorState`);}
  async function choose(label){return evaluate(`document.getElementById('film').value=${JSON.stringify(label)};document.getElementById('film').dispatchEvent(new Event('change'))`);}
  let state=await complete('natural-end/03-shoot');
  assert(state.recordingCoverage.total_bits>state.view.scope[1]-state.view.scope[0]);
  const initial=state.recordingCoverage;
  assert.equal(initial.chunks_scanned,initial.chunks_total);
  assert.equal(Object.values(initial.coverage).reduce((a,b)=>a+b),initial.total_bits);
  assert.equal(initial.scanned_bits,initial.total_bits);
  assert.equal(await evaluate('document.querySelectorAll("#recording-coverage-stats > div").length'),4);
  assert.match(await evaluate('getComputedStyle(document.getElementById("recording-coverage-chart")).backgroundImage'),/conic-gradient/);
  const chartLabel=await evaluate('document.getElementById("recording-coverage-chart").getAttribute("aria-label")');
  for(const name of ['Decoded','Checked structure','Opaque','Unparsed'])assert(chartLabel.includes(name));
  await evaluate('document.getElementById("next-page").click()');
  await waitFor(`window.theaterInspectorState.view.offset!==${state.view.offset}`);
  assert.deepEqual(await evaluate('window.theaterInspectorState.recordingCoverage'),initial);
  await cdp('Browser.setDownloadBehavior',{behavior:'allow',downloadPath:profile});
  await evaluate('document.getElementById("export-coverage").click()');
  const download=path.join(profile,`theater-${initial.match_id}-coverage.json`);
  for(let n=0;n<100&&!fs.existsSync(download);n++)await delay(50);
  assert.deepEqual(JSON.parse(fs.readFileSync(download,'utf8')),initial);
  // Changing films during a scan clears percentages and cannot mix old results.
  await choose('ranked-arena/02-oddball');
  await waitFor('window.theaterInspectorState?.recordingCoverage?.film==="ranked-arena/02-oddball" && !window.theaterInspectorState.recordingCoverage.complete');
  assert(await evaluate('document.getElementById("export-coverage").disabled'));
  assert(!await evaluate('document.getElementById("recording-coverage-stats").textContent.includes("%")'));
  await choose('natural-end/03-shoot');await complete('natural-end/03-shoot');await delay(500);
  assert.deepEqual(await evaluate('window.theaterInspectorState.recordingCoverage'),initial);
  await choose('ranked-arena/02-oddball');state=await complete('ranked-arena/02-oddball');
  const totals=state.recordingCoverage;
  const metadata=await evaluate('(async()=>await(await fetch("/api/film?film=ranked-arena%2F02-oddball")).json())()');
  assert.equal(totals.total_bits,metadata.chunks.reduce((n,c)=>n+c.size*8,0));
  assert.equal(Object.values(totals.coverage).reduce((a,b)=>a+b),totals.total_bits);
  assert(Object.values(totals.coverage).every(n=>n>0));
  assert.equal(await evaluate('document.getElementById("recording-coverage").getAttribute("aria-busy")'),'false');
  await snap('theater-inspector-recording-coverage');
  // Current native summary packets contribute to coverage and link medal names to exact bits.
  await evaluate('document.getElementById("match-events").click()');
  state=await waitFor('window.theaterInspectorState.view.packet?.kind===9 && window.theaterInspectorState');
  const summary=await evaluate(`(async()=>await(await fetch('/api/chunk?'+new URLSearchParams({film:'ranked-arena/02-oddball',chunk:${state.chunk}}))).json())()`);
  assert.deepEqual(summary.packets.map(p=>p.kind),[9,7]);
  assert.equal(summary.events.length,734);assert.equal(summary.events.filter(e=>e.medal).length,132);
  assert.equal(await evaluate('document.querySelectorAll("#summary-event option").length'),734);
  assert.match(await evaluate('document.getElementById("notice").textContent'),/132 medals/);
  const medalIndex=summary.events.findIndex(e=>e.medal);
  async function selectMedal(){
    await evaluate(`document.getElementById('summary-event').value='${medalIndex}';document.getElementById('summary-event').dispatchEvent(new Event('change'))`);
    return waitFor('window.theaterInspectorState.view.spans.some(s=>s.label==="Medal") && window.theaterInspectorState');
  }
  state=await selectMedal();
  assert(state.view.spans.length<40,'Summary fields must be paged, including in long films');
  assert(state.view.coverage.opaque>state.view.coverage.decoded);
  const medal=state.view.spans.find(s=>s.label==='Medal');
  assert.equal(medal.end-medal.start,8);assert.match(medal.note,/NameId/);
  await evaluate(`document.querySelector('.field[data-start="${medal.start}"]').click()`);
  state=await waitFor(`window.theaterInspectorState.selected.start===${medal.start} && window.theaterInspectorState`);
  assert.equal(state.selected.end-state.selected.start,8);
  await snap('theater-inspector-medals');
  await evaluate('document.getElementById("refresh-decoding").click()');
  await complete('ranked-arena/02-oddball');
  await waitFor('window.theaterInspectorState.view.packet?.kind===9');
  assert.deepEqual(await evaluate('window.theaterInspectorState.recordingCoverage.coverage'),totals.coverage);
  await selectMedal();
  await cdp('Emulation.setDeviceMetricsOverride',{width:390,height:1100,deviceScaleFactor:1,mobile:true});
  await snap('theater-inspector-recording-coverage-mobile');
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
  assert(await evaluate('[...document.querySelectorAll("#recording-coverage-chart, #recording-coverage-stats")].every(e=>{const r=e.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth;})'));
  assert.deepEqual(errors,[]);
  console.log('PASS: whole-recording totals, pie/stats, JSON export, cache refresh, current native summary packets, all Oddball medals, exact byte linking, paged fields and mobile layout');
  console.log(JSON.stringify(totals));
 }finally{
  const stopped=new Promise(resolve=>chrome.once('exit',resolve));chrome.kill('SIGTERM');await stopped;
  for(const p of pending.values())clearTimeout(p.timer);
  fs.rmSync(profile,{recursive:true,force:true,maxRetries:3,retryDelay:100});
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
