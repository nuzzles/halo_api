(() => {
  'use strict';
  const $ = id => document.getElementById(id);
  const COLORS = {decoded:'#89ead0', structure:'#8db7f4', opaque:'#e3b56b', unparsed:'#8b758c'};
  const TITLES = {decoded:'Decoded', structure:'Checked structure', opaque:'Opaque', unparsed:'Unparsed'};
  const number = n => n.toLocaleString('en-US');
  const hex = n => '0x' + n.toString(16).toUpperCase().padStart(8, '0');
  const el = (tag, text, cls) => { const e=document.createElement(tag); if(text!==undefined)e.textContent=text; if(cls)e.className=cls; return e; };
  let film=null, chunk=null, view=null, selected=null, selectedByte=0, packetPage=0, revision=0;
  let recordingCoverage=null, coverageController=null, coverageRevision=0;
  const coverageCache=new Map();
  let lastSearch='';
  function notice(message, error=false) { $('notice').textContent=message; $('notice').classList.toggle('error',error); }
  async function api(path, args={}, signal) {
    const response=await fetch('/api/'+path+'?'+new URLSearchParams(args),{signal});
    const data=await response.json(); if(!response.ok)throw Error(data.error || 'Request failed'); return data;
  }
  function params(extra={}) { return {film:film.label, chunk:chunk.index, ...extra}; }
  function run(task) { const token=++revision; notice('Reading cached bytes and checking decoder evidence…'); return task(token).catch(e=>{if(token===revision)notice(e.message,true);}); }
  function tone(element,status) { element.style.setProperty('--tone',COLORS[status]); }
  function currentSpan(bit) { return view?.spans.find(s=>s.start<=bit && bit<s.end); }
  function reportState() { window.theaterInspectorState={film:film?.label,chunk:chunk?.index,view,selected,selectedByte,revision,recordingCoverage}; }
  function renderRecordingCoverage(message) {
    const result=recordingCoverage, complete=Boolean(result?.complete), total=result?.total_bits || 0;
    $('recording-coverage').setAttribute('aria-busy',String(Boolean(coverageController)));
    $('recording-coverage-status').textContent=message;
    $('export-coverage').disabled=!complete;
    const stats=[],sectors=[],labels=[];let angle=0;
    for(const status of Object.keys(COLORS)) {
      const count=complete?result.coverage[status]:0, percent=total?count/total*100:0;
      const percentage=count && percent<.01?'<0.01%':percent.toFixed(2)+'%';
      const cell=el('div'),title=el('dt'),dot=el('i',undefined,'dot');dot.style.background=COLORS[status];
      title.append(dot,el('span',TITLES[status]));
      const value=el('dd',complete?percentage:'—');
      value.append(el('small',complete?number(count)+' bits':'Not calculated'));
      cell.dataset.status=status;cell.append(title,value);stats.push(cell);
      sectors.push(`${COLORS[status]} ${angle}% ${angle+percent}%`);angle+=percent;
      labels.push(`${TITLES[status]}: ${percentage}, ${number(count)} bits`);
    }
    $('recording-coverage-stats').replaceChildren(...stats);
    $('recording-coverage-chart').style.background=complete&&total?`conic-gradient(${sectors.join(',')})`:'';
    $('recording-coverage-chart').setAttribute('aria-label',complete?labels.join('; '):'Coverage not calculated');
    reportState();
  }
  function resetRecordingCoverage() {
    coverageController?.abort();coverageController=null;coverageRevision++;recordingCoverage=null;
    renderRecordingCoverage('Loading recording…');
  }
  async function loadRecordingCoverage(recording) {
    const token=coverageRevision;
    if(coverageCache.has(recording.label)) {
      recordingCoverage=coverageCache.get(recording.label);finishRecordingCoverage();return;
    }
    const controller=new AbortController();coverageController=controller;
    const result={film:recording.label,match_id:recording.match_id,
      basis:'Verified inspector annotations across all decompressed chunk bytes, including headers and padding. Decoder fields without exact annotations remain unparsed.',
      total_bits:recording.chunks.reduce((n,c)=>n+c.size*8,0),scanned_bits:0,
      coverage:Object.fromEntries(Object.keys(COLORS).map(s=>[s,0])),chunks_scanned:0,
      chunks_total:recording.chunks.length,issue_count:0,issues:[],complete:false};
    recordingCoverage=result;
    renderRecordingCoverage(`Calculating · 0 / ${number(result.chunks_total)} chunks`);
    try {
      for(const c of recording.chunks) {
        const row=await api('coverage',{film:recording.label,chunk:c.index},controller.signal);
        if(token!==coverageRevision)return;
        if(row.total_bits!==c.size*8 || Object.values(row.coverage).reduce((n,v)=>n+v,0)!==row.total_bits)throw Error('Chunk sizes changed; reload the inspector to recalculate.');
        for(const status of Object.keys(COLORS))result.coverage[status]+=row.coverage[status];
        result.scanned_bits+=row.total_bits;result.chunks_scanned++;result.issue_count+=row.issue_count;
        result.issues.push(...row.issues.slice(0,Math.max(0,8-result.issues.length)).map(issue=>({...issue,chunk:c.index})));
        renderRecordingCoverage(`Calculating · ${number(result.chunks_scanned)} / ${number(result.chunks_total)} chunks · ${number(result.scanned_bits/8)} / ${number(result.total_bits/8)} bytes checked`);
      }
      result.complete=true;coverageController=null;coverageCache.set(recording.label,result);finishRecordingCoverage();
    } catch(error) {
      if(token!==coverageRevision)return;
      coverageController=null;
      renderRecordingCoverage('Coverage unavailable · '+error.message);
    }
  }
  function finishRecordingCoverage() {
    const result=recordingCoverage;
    renderRecordingCoverage(`${number(result.total_bits/8)} bytes · ${number(result.total_bits)} bits · ${number(result.chunks_total)} chunks${result.issue_count?' · '+number(result.issue_count)+' annotations withheld':''}`);
  }
  async function show(offset, token, selection=null) {
    const result=await api('view',params({offset})); if(token!==revision)return;
    view=result;
    selected=selection || {start:offset*8,end:offset*8+8}; selectedByte=Math.floor(selected.start/8);
    render();
    notice(view.issues.length ? view.issues.join(' · ') : 'Read only · Select a field or byte to inspect its exact bits. Unannotated regions stay unparsed.', view.issues.length>0);
  }
  async function loadChunk(index, offset, token, selection=null, preferEvidence=false) {
    const result=await api('chunk',{film:film.label,chunk:index}); if(token!==revision)return;
    chunk=result; $('chunk').value=index; packetPage=0; lastSearch='';
    if(preferEvidence && !chunk.packets.find(p=>p.offset===offset)?.evidence.length) {
      const candidates=chunk.packets.filter(p=>p.evidence.length);
      if(candidates.length)offset=candidates.reduce((a,b)=>Math.abs(a.offset-offset)<Math.abs(b.offset-offset)?a:b).offset;
    }
    const selectedIndex=filteredPackets().findIndex(p=>p.offset===offset);
    if(selectedIndex>=0)packetPage=Math.floor(selectedIndex/40);
    await show(offset,token,selection);
  }
  async function loadFilm(label, token) {
    resetRecordingCoverage();
    const result=await api('film',{film:label}); if(token!==revision)return;
    film=result; $('film').value=label;
    const query=new URLSearchParams({film:label});
    history.replaceState(null,'','?'+query);
    document.querySelector('nav a:not([aria-current])').href='/replay?'+new URLSearchParams({clip:label});
    document.querySelector('nav a[aria-current]').href='/?'+query;
    loadRecordingCoverage(result);
    $('film-meta').replaceChildren(el('div',`v${film.version} · ${number(film.chunks.length)} chunks · ${(film.duration/60).toFixed(2)} minutes · ${number(film.chunks.reduce((n,c)=>n+c.size,0))} bytes`),el('code',film.match_id));
    $('chunk').replaceChildren(...film.chunks.map(c=>{const o=el('option',`${String(c.index).padStart(3,'0')} · ${c.chunk_type===1?'Registry':c.chunk_type===2?'Replication':'Type '+c.chunk_type} · ${(c.size/1024).toFixed(0)} KiB`);o.value=c.index;return o;}));
    $('time').max=film.duration;
    const time=Math.min(30,film.duration); $('time').value=time;
    const target=await api('seek',{film:label,time}); if(token!==revision)return;
    await loadChunk(target.chunk,target.offset,token,null,true);
  }
  function filteredPackets() {
    if(!chunk)return [];
    const filter=$('packet-filter').value;
    return chunk.packets.filter(p=>filter==='all'||filter==='frames'&&p.kind===0||filter==='evidence'&&p.evidence.length);
  }
  function renderPackets() {
    const rows=filteredPackets(), pages=Math.max(1,Math.ceil(rows.length/40)); packetPage=Math.min(packetPage,pages-1);
    $('packet-count').textContent=number(chunk.packets.length);
    const buttons=rows.slice(packetPage*40,packetPage*40+40).map(p=>{
      const b=el('button',undefined,'packet'+(view.packet?.offset===p.offset?' selected':''));
      const title=el('strong');title.append(el('span',p.time==null?'No timestamp':p.time.toFixed(3)+' s'),el('span',p.kind===0?'FRAME':p.kind===10?'MARKER':'KIND '+p.kind));
      b.append(title,el('small',hex(p.offset)+' · '+number(p.size)+' B'),el('small',p.evidence.join(' · ')||'Packet header'));
      b.onclick=()=>run(t=>show(p.offset,t));return b;
    });
    $('packets').replaceChildren(...(buttons.length?buttons:[el('div',chunk.packets.length?'No packets match this filter. Choose Frames or All packets.':'This chunk has no replication packet stream. Browse bytes or search its text.', 'empty')]));
    $('packet-page').textContent=`${packetPage+1} / ${pages}`;
    $('packets-prev').disabled=packetPage===0;$('packets-next').disabled=packetPage+1>=pages;
  }
  function renderCoverage() {
    const total=view.scope[1]-view.scope[0];
    $('scope-title').textContent=view.packet?`${view.packet.kind===0?'Frame':'Packet '+view.packet.kind} · ${view.packet.time==null?'no timestamp':view.packet.time.toFixed(6)+' s'}`:'Current byte window';
    $('scope-size').textContent=number(total/8)+' bytes';
    const bars=[],legends=[];
    for(const [status,n] of Object.entries(view.coverage)) {
      const segment=el('span');segment.style.width=(n/total*100)+'%';segment.style.background=COLORS[status];segment.title=`${TITLES[status]}: ${number(n)} bits`;bars.push(segment);
      const item=el('span'),dot=el('i',undefined,'dot');dot.style.background=COLORS[status];item.append(dot,el('span',TITLES[status]),el('b',(n/total*100).toFixed(1)+'%'));item.title=number(n)+' bits';legends.push(item);
    }
    $('coverage-bar').replaceChildren(...bars);$('legend').replaceChildren(...legends);
    $('coverage-note').textContent=`${view.packet?'Selected packet, including its 16-byte header':'Visible chunk window'} · ${number(total)} bits. Opaque = checked length, unknown value. Coverage does not measure the entire recording.`;
  }
  function choose(start,end,bitByte=null) {
    selected={start,end}; selectedByte=bitByte ?? Math.floor(start/8);
    if(selectedByte<view.offset||selectedByte>=view.end) {
      return run(t=>show(selectedByte,t,selected));
    }
    renderBytes();renderFields();renderSelection();reportState();
  }
  function renderBytes() {
    $('offset').value=hex(view.offset);$('byte-range').textContent=`${hex(view.offset)}–${hex(view.end-1)}`;
    $('prev-page').disabled=view.offset===0;$('next-page').disabled=view.end>=view.chunk_size;
    const fragment=document.createDocumentFragment();
    for(let i=0;i<view.bytes.length;i+=16) {
      const row=el('div',undefined,'hex-row'),hexes=el('div',undefined,'hex-bytes'),ascii=el('div',undefined,'ascii-bytes');
      row.append(el('span',hex(view.offset+i).slice(2),'hex-offset'),hexes,ascii);
      for(let j=i;j<Math.min(i+16,view.bytes.length);j++) {
        const offset=view.offset+j,byte=view.bytes[j],statuses=Array.from({length:8},(_,b)=>currentSpan(offset*8+b)?.status||'unparsed');
        const mixed=new Set(statuses).size>1,active=selected&&offset*8<selected.end&&(offset+1)*8>selected.start;
        for(const [parent,text] of [[hexes,byte.toString(16).padStart(2,'0').toUpperCase()],[ascii,byte>=32&&byte<127?String.fromCharCode(byte):'·']]) {
          const b=el('button',text,'byte'+(mixed?' mixed':'')+(active?' active':''));tone(b,mixed?'structure':statuses[0]);
          b.dataset.byte=offset;b.title=`${hex(offset)} · ${byte} · ${mixed?'Mixed bit coverage':TITLES[statuses[0]]}`;
          b.setAttribute('aria-label',`Byte ${offset}: ${byte.toString(16).padStart(2,'0')}`);
          b.onclick=()=>choose(offset*8,offset*8+8,offset);parent.append(b);
        }
      }
      fragment.append(row);
    }
    $('hex').replaceChildren(fragment);
  }
  function renderFields() {
    const filter=$('field-filter').value,rows=view.spans.filter(s=>filter==='all'||s.status===filter);
    $('field-count').textContent=number(view.spans.length)+' regions';
    const fragment=document.createDocumentFragment();let lastRecord=null;
    for(const s of rows) {
      if(lastRecord!==s.record){fragment.append(el('div',s.record,'record-label'));lastRecord=s.record;}
      const active=selected&&s.start<selected.end&&s.end>selected.start;
      const b=el('button',undefined,'field'+(active?' selected':''));tone(b,s.status);b.dataset.start=s.start;b.dataset.status=s.status;
      const title=el('span',undefined,'field-title'),dot=el('i',undefined,'dot');dot.style.background=COLORS[s.status];title.append(dot,el('span',s.label));
      const value=s.value==null?(s.status==='unparsed'?'Not parsed':'Value unknown'):(Array.isArray(s.value)?s.value.join(', '):String(s.value));
      b.append(title,el('strong',value.length>84?value.slice(0,81)+'…':value),el('small',`${hex(Math.floor(s.start/8))} + bit ${s.start%8} · ${number(s.end-s.start)} bits`));
      b.title=`${TITLES[s.status]} · bits ${s.start}–${s.end-1}${s.note?' · '+s.note:''}`;
      b.onclick=()=>choose(s.start,s.end);fragment.append(b);
    }
    if(!rows.length)fragment.append(el('div','No regions in this category.','empty'));
    $('fields').replaceChildren(fragment);
  }
  function renderSelection() {
    const byte=view.bytes[selectedByte-view.offset]; if(byte===undefined)return;
    const hits=view.spans.filter(s=>s.start<selected.end&&s.end>selected.start),span=hits.length===1?hits[0]:null;
    $('selection-title').textContent=`Byte ${hex(selectedByte)}`;
    $('selection-summary').textContent=`${span?span.label:'Mixed fields'} · selected bits ${number(selected.start)}–${number(selected.end-1)} · ${number(selected.end-selected.start)} bit${selected.end-selected.start===1?'':'s'}`;
    const buttons=[];
    for(let i=0;i<8;i++) {
      const a=selectedByte*8+i,s=currentSpan(a),b=el('button',String((byte>>(7-i))&1),'bit'+(a>=selected.start&&a<selected.end?' active':''));tone(b,s?.status||'unparsed');b.append(el('small','bit '+i));
      b.title=`Chunk bit ${a} · ${s?.label||'Unparsed'} · ${TITLES[s?.status||'unparsed']}`;b.dataset.bit=a;
      b.onclick=()=>choose(a,a+1,selectedByte);buttons.push(b);
    }
    $('bits').replaceChildren(...buttons);
    $('value').replaceChildren();
    if(span&&span.value!=null)$('value').append(el('strong',Array.isArray(span.value)?span.value.join(', '):String(span.value)),el('div',TITLES[span.status]+' · '+number(span.field_end-span.field_start)+'-bit field'));
    else $('value').append(el('div',span?TITLES[span.status]:hits.map(s=>s.label).join(' · ')));
    $('value').append(el('div',`Byte: ${byte} decimal · ${byte.toString(16).padStart(2,'0').toUpperCase()} hex · ${byte.toString(2).padStart(8,'0')} binary`));
    const start=Math.max(view.offset,Math.floor(selected.start/8)),end=Math.min(view.end,Math.ceil(selected.end/8));
    const raw=new Uint8Array(view.bytes.slice(start-view.offset,end-view.offset));
    const text=new TextDecoder('utf-8',{fatal:false}).decode(raw).replace(/[\u0000-\u001f\u007f]/g,'·');
    $('value').append(el('div','UTF-8 preview: '+(text.slice(0,100)||'·')+(text.length>100?'…':'')));
    $('source').replaceChildren();
    const sources=[...new Set(hits.map(s=>s.source).filter(Boolean))];
    $('source').append(el('code',sources.join(' · ')||'No supported source annotation'));
    for(const note of [...new Set(hits.map(s=>s.note).filter(Boolean))])$('source').append(el('div',note));
    if(span)$('source').append(el('div',`${hex(Math.floor(span.field_start/8))} + bit ${span.field_start%8} → ${hex(Math.floor((span.field_end-1)/8))} + bit ${(span.field_end-1)%8}`));
    if(view.packet)$('source').append(el('div',`Packet-relative bit ${selected.start-view.packet.offset*8}; payload-relative bit ${selected.start-view.packet.payload*8}.`));
  }
  function render() {renderPackets();renderCoverage();renderBytes();renderFields();renderSelection();reportState();}
  $('film').onchange=()=>run(t=>loadFilm($('film').value,t));
  $('chunk').onchange=()=>run(t=>loadChunk(Number($('chunk').value),0,t));
  $('packet-filter').onchange=()=>{packetPage=0;renderPackets();};
  $('packets-prev').onclick=()=>{packetPage--;renderPackets();};$('packets-next').onclick=()=>{packetPage++;renderPackets();};
  $('field-filter').onchange=renderFields;
  $('prev-page').onclick=()=>run(t=>show(Math.max(0,view.offset-256),t));
  $('next-page').onclick=()=>run(t=>show(view.end,t));
  $('offset-form').onsubmit=e=>{e.preventDefault();const offset=Number($('offset').value);if(!Number.isSafeInteger(offset)||offset<0||offset>=view.chunk_size)return notice('Enter a chunk byte offset in decimal or 0x hexadecimal.',true);run(t=>show(offset,t));};
  $('time-form').onsubmit=e=>{e.preventDefault();run(async t=>{const target=await api('seek',{film:film.label,time:$('time').value});if(t!==revision)return;await loadChunk(target.chunk,target.offset,t);});};
  $('search-form').onsubmit=e=>{e.preventDefault();run(async t=>{
    const q=$('search').value,mode=$('search-mode').value,key=film.label+'/'+chunk.index+'/'+mode+'/'+q,start=key===lastSearch?Math.min(view.chunk_size,selectedByte+1):view.offset;
    const found=await api('search',params({q,mode,start}));if(t!==revision)return;lastSearch=key;
    if(found.offset<0){notice('No matching bytes in this chunk.');return;}
    await show(found.offset,t,{start:found.offset*8,end:(found.offset+found.length)*8});
    if(t===revision)notice('Found at '+hex(found.offset)+(found.wrapped?' · wrapped to the beginning of this chunk.':'.'));
  });};
  $('copy-hex').textContent='Copy page hex';$('export-json').textContent='Export page JSON';
  $('copy-hex').onclick=async()=>{try{await navigator.clipboard.writeText(view.bytes.map(b=>b.toString(16).padStart(2,'0')).join(' '));notice('Visible byte page copied as hex.');}catch{notice('Clipboard unavailable. Use Export page JSON to save the bytes.',true);}};
  $('export-json').onclick=()=>{
    const data={recording:film.match_id,chunk:chunk.index,offset:view.offset,hex:view.bytes.map(b=>b.toString(16).padStart(2,'0')).join(' '),selected,scope:view.scope,coverage:view.coverage,spans:view.spans,issues:view.issues};
    const url=URL.createObjectURL(new Blob([JSON.stringify(data,null,2)],{type:'application/json'}));const a=el('a');a.href=url;a.download=`theater-${chunk.index}-${view.offset}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
  };
  $('export-coverage').onclick=()=>{
    if(!recordingCoverage?.complete)return;
    const url=URL.createObjectURL(new Blob([JSON.stringify(recordingCoverage,null,2)],{type:'application/json'}));
    const a=el('a');a.href=url;a.download=`theater-${recordingCoverage.match_id}-coverage.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
  };
  run(async t=>{
    const catalog=await api('catalog');if(t!==revision)return;
    TheaterRecordings.populate($('film'),catalog);
    if(!catalog.length){notice('No cached recordings found. Download films using the experiment catalog.',true);return;}
    const requested=new URLSearchParams(location.search).get('film');await loadFilm(catalog.some(r=>r.label===requested)?requested:catalog[0].label,t);
  });
})();
