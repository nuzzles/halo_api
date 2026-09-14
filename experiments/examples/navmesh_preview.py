"""Standalone orbit preview for decoded navigation polygons; no dependencies."""
import json


def height_groups(mesh):
    """Offer a camera filter across a large empty Z interval, not an object label."""
    spans = [(min(mesh['vertices'][i][2] for i in p), max(mesh['vertices'][i][2] for i in p), n)
             for n, p in enumerate(mesh['polygons'])]
    bands = []
    for lo, hi, _ in sorted(spans):
        if bands and lo <= bands[-1][1]:
            bands[-1][1] = max(bands[-1][1], hi)
        else:
            bands.append([lo, hi])
    if len(bands) < 2:
        return []
    gap, split = max((b[0] - a[1], (a[1] + b[0]) / 2) for a, b in zip(bands, bands[1:]))
    if gap <= (bands[-1][1] - bands[0][0]) / 4:
        return []
    result = []
    for key, title, ids in [('lower', 'Lower surfaces', [n for lo, hi, n in spans if hi < split]),
                            ('upper', 'Upper surfaces', [n for lo, hi, n in spans if lo > split])]:
        lo = min(spans[i][0] for i in ids)
        hi = max(spans[i][1] for i in ids)
        noun = 'polygon' if len(ids) == 1 else 'polygons'
        result.append(dict(key=key, label=f'{title} ({len(ids)} {noun}, Z {lo:.2f}–{hi:.2f})', polygons=ids))
    return result


def html_text(report):
    mesh = report['navmesh']
    value = dict(source=report['source'], sha256=report['source_sha256'],
                 vertices=mesh['vertices'], polygons=mesh['polygons'], groups=height_groups(mesh))
    if 'spatial_tree' in report:
        tree = report['spatial_tree']
        # Compact rows keep the standalone preview usable with 49k nodes.
        value['tree'] = dict(
            nodes=[[n['parent'], *(n['children'] or [None, None]), n['cell_index'], n['depth'],
                    *n['bounds_min'], *n['bounds_max']] for n in tree['nodes']],
            cells=[[*c['bounds_min'], *c['bounds_max'], c['sample_count'],
                    list(c.get('face_sample_counts', {}))] for c in report['spatial_cells']['cells']],
            leaf_by_cell=tree['cell_links']['leaf_node_by_cell'])
    data = json.dumps(value, allow_nan=False, separators=(',', ':')).replace('<', '\\u003c')
    return TEMPLATE.replace('/*NAVMESH_DATA*/', data)


TEMPLATE = '''<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Navigation surface viewer</title>
<style>
*{box-sizing:border-box}body{margin:0;background:#101722;color:#e8edf5;font:15px system-ui}
header{padding:18px 22px;background:#182333}h1{font-size:20px;margin:0 0 7px}
p{margin:5px 0;color:#b8c6d9}select,button,input{font:inherit;background:#22354a;color:#fff;
border:1px solid #58718e;border-radius:5px;padding:5px 9px;margin-left:6px}
input[type=number]{width:105px}button:disabled{opacity:.4}#tree-controls[hidden]{display:none}
nav{display:flex;flex-wrap:wrap;gap:12px;align-items:center;margin-top:14px}
canvas{display:block;width:100%;height:65vh;min-height:300px;touch-action:none}
footer{padding:14px 22px;overflow-wrap:anywhere;font-size:13px}#details{color:#8cded5}
</style>
<header><h1>Navigation surfaces</h1>
<p>Decoded from the map's companion asset. These polygons do not describe every wall or collision surface.</p>
<nav><label>Show <select id="face"><option value="all">All polygons</option></select></label>
<button id="frame">Frame selection</button><button id="reset">Reset view</button>
<span>Drag to orbit · Shift-drag or right-drag to pan · Scroll to zoom at cursor</span></nav>
<p id="group-help" hidden>All polygons includes separated height ranges. Choose Upper or Lower surfaces to frame a group. World coordinates, Z up.</p>
<nav id="tree-controls" hidden><label><input type="checkbox" id="tree-enabled"> Spatial tree</label>
<label>Node <input type="number" id="node" value="0" min="0" step="1"></label>
<button id="root">Root</button><button id="parent">Parent</button>
<button id="left">Left child</button><button id="right">Right child</button>
<label>Go to cell <input type="number" id="cell" value="0" min="0" step="1"></label><button id="go-cell">Go</button></nav>
</header><canvas id="view" aria-label="Interactive 3D navigation polygons"></canvas>
<footer><p id="details"></p><p id="tree-details"></p><p id="source"></p><p id="hash"></p>
<p>Original vertex coordinates; no film-to-map transform or inferred geometry.</p></footer>
<script>
'use strict';
const data=/*NAVMESH_DATA*/;
const canvas=document.querySelector('#view'),ctx=canvas.getContext('2d'),select=document.querySelector('#face');
const vertices=data.vertices,polygons=data.polygons;
const treeEnabled=document.querySelector('#tree-enabled'),nodeInput=document.querySelector('#node');
let nodeId=0;
function currentNode(){return data.tree&&treeEnabled.checked?data.tree.nodes[nodeId]:null}
function boxPoints(lo,hi){return Array.from({length:8},(_,i)=>[0,1,2].map(a=>i&(1<<a)?hi[a]:lo[a]))}
function moveNode(id){if(!Number.isInteger(id)||id<0||id>=data.tree.nodes.length)return;nodeId=id;nodeInput.value=id;treeEnabled.checked=true;fit()}
if(data.tree){
 document.querySelector('#tree-controls').hidden=false;nodeInput.max=data.tree.nodes.length-1;
 document.querySelector('#cell').max=data.tree.cells.length-1;
 nodeInput.addEventListener('change',()=>{const id=nodeInput.valueAsNumber;moveNode(id);nodeInput.value=nodeId});
 treeEnabled.addEventListener('change',fit);document.querySelector('#root').addEventListener('click',()=>moveNode(0));
 for(const [name,slot]of [['parent',0],['left',1],['right',2]])document.querySelector('#'+name).addEventListener('click',()=>{const id=data.tree.nodes[nodeId][slot];if(id!==null)moveNode(id)});
 document.querySelector('#go-cell').addEventListener('click',()=>{const id=document.querySelector('#cell').valueAsNumber;if(Number.isInteger(id)&&id>=0&&id<data.tree.cells.length)moveNode(data.tree.leaf_by_cell[id])});
}
for(const group of data.groups){const o=document.createElement('option');o.value=group.key;o.textContent=group.label;select.append(o)}
document.querySelector('#group-help').hidden=!data.groups.length;
polygons.forEach((p,i)=>{const o=document.createElement('option');o.value=i;o.textContent=`Polygon ${i} (${p.length} vertices)`;select.append(o)});
document.querySelector('#source').textContent=data.source;
document.querySelector('#hash').textContent=`Source SHA-256: ${data.sha256}`;
let yaw=-.65,pitch=-.9,zoom=1,center=[0,0,0],radius=1,width=1,height=1,drag=null;
function selected(){const n=currentNode();if(n&&n[3]!==null)return data.tree.cells[n[3]][7].map(Number);const group=data.groups.find(g=>g.key===select.value);return group?group.polygons.slice():select.value==='all'?polygons.map((p,i)=>i):[Number(select.value)]}
function fit(){
 const n=currentNode(),ids=new Set(selected().flatMap(i=>polygons[i])),points=n?boxPoints(n.slice(5,8),n.slice(8,11)):[...ids].map(i=>vertices[i]);
 const lo=[0,1,2].map(i=>Math.min(...points.map(v=>v[i]))),hi=[0,1,2].map(i=>Math.max(...points.map(v=>v[i])));
 center=lo.map((v,i)=>(v+hi[i])/2);radius=Math.max(1e-6,...points.map(v=>Math.hypot(...center.map((c,i)=>v[i]-c))));zoom=1;
 document.querySelector('#details').textContent=`${selected().length} polygons · ${ids.size} vertices · XYZ min [${lo.map(v=>v.toFixed(4)).join(', ')}] · max [${hi.map(v=>v.toFixed(4)).join(', ')}]`;
 document.querySelector('#tree-details').textContent=n?`Node ${nodeId} · Depth ${n[4]} · ${n[3]===null?'Branch':`Cell ${n[3]} · ${data.tree.cells[n[3]][6]} samples`} · Cyan: encoded tree bounds · Amber: cell bounds · Green/purple: children`:'';
 for(const [name,slot]of [['parent',0],['left',1],['right',2]])document.querySelector('#'+name).disabled=!n||n[slot]===null;
 draw();
}
function rotate(v){
 const x=Math.cos(yaw)*v[0]-Math.sin(yaw)*v[1],y=Math.sin(yaw)*v[0]+Math.cos(yaw)*v[1];
 return [x,Math.cos(pitch)*y-Math.sin(pitch)*v[2],Math.sin(pitch)*y+Math.cos(pitch)*v[2]];
}
function scale(){return .41*Math.min(width,height)*zoom/radius}
function project(v){const p=rotate(v.map((a,i)=>a-center[i])),s=scale();return [width/2+p[0]*s,height/2-p[1]*s,p[2]]}
function moveCenterView(x,y){
 center[0]+=Math.cos(yaw)*x+Math.cos(pitch)*Math.sin(yaw)*y;
 center[1]+=-Math.sin(yaw)*x+Math.cos(pitch)*Math.cos(yaw)*y;
 center[2]+=-Math.sin(pitch)*y;
}
function panPixels(dx,dy){const s=scale();moveCenterView(-dx/s,dy/s)}
function draw(){
 ctx.clearRect(0,0,width,height);ctx.lineWidth=1.4;
 const projected=vertices.map(project);
 const order=selected().sort((a,b)=>polygons[a].reduce((s,i)=>s+projected[i][2],0)/polygons[a].length-polygons[b].reduce((s,i)=>s+projected[i][2],0)/polygons[b].length);
 for(const i of order){
  const p=polygons[i].map(j=>projected[j]);ctx.beginPath();p.forEach((v,j)=>j?ctx.lineTo(v[0],v[1]):ctx.moveTo(v[0],v[1]));ctx.closePath();
  ctx.fillStyle=`hsla(${(i*47+165)%360},65%,55%,.2)`;ctx.strokeStyle=`hsl(${(i*47+165)%360},70%,70%)`;ctx.fill();ctx.stroke();
  for(const v of p){ctx.fillStyle='#ebf5ff';ctx.fillRect(v[0]-2,v[1]-2,4,4)}
  const x=p.reduce((s,v)=>s+v[0],0)/p.length,y=p.reduce((s,v)=>s+v[1],0)/p.length;
  ctx.font='13px system-ui';ctx.fillStyle='#fff';ctx.fillText(String(i),x+5,y-5);
 }
 function drawBox(lo,hi,color,lineWidth){const p=boxPoints(lo,hi).map(project);ctx.strokeStyle=color;ctx.lineWidth=lineWidth;ctx.beginPath();for(let i=0;i<8;i++)for(let a=0;a<3;a++)if(!(i&(1<<a))){ctx.moveTo(p[i][0],p[i][1]);ctx.lineTo(p[i|(1<<a)][0],p[i|(1<<a)][1])}ctx.stroke()}
 const n=currentNode();
 if(n){
  drawBox(n.slice(5,8),n.slice(8,11),'#6be7f2',2);
  for(const [slot,color]of [[1,'#8befa4'],[2,'#d89cff']])if(n[slot]!==null){const c=data.tree.nodes[n[slot]];drawBox(c.slice(5,8),c.slice(8,11),color,1)}
  if(n[3]!==null){const c=data.tree.cells[n[3]];drawBox(c.slice(0,3),c.slice(3,6),'#ffc771',2)}
 }
 ctx.lineWidth=1.4;
 for(const [i,color] of ['#ff8c8c','#8befa4','#8ebeff'].entries()){
  const v=rotate([0,1,2].map(j=>j===i?1:0));ctx.beginPath();ctx.moveTo(50,height-50);ctx.lineTo(50+v[0]*32,height-50-v[1]*32);ctx.strokeStyle=color;ctx.stroke();ctx.fillStyle=color;ctx.fillText('XYZ'[i],50+v[0]*43,height-50-v[1]*43);
 }
}
function resize(){const r=canvas.getBoundingClientRect(),dpr=devicePixelRatio||1;width=r.width;height=r.height;canvas.width=Math.round(width*dpr);canvas.height=Math.round(height*dpr);ctx.setTransform(dpr,0,0,dpr,0,0);draw()}
canvas.addEventListener('pointerdown',e=>{e.preventDefault();drag={x:e.clientX,y:e.clientY,pan:e.shiftKey||e.button===1||e.button===2};canvas.setPointerCapture(e.pointerId)});
canvas.addEventListener('pointermove',e=>{if(!drag)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(drag.pan)panPixels(dx,dy);else{yaw+=dx*.008;pitch=Math.max(-1.5,Math.min(1.5,pitch+dy*.008))}drag.x=e.clientX;drag.y=e.clientY;draw()});
canvas.addEventListener('contextmenu',e=>e.preventDefault());
for(const event of ['pointerup','pointercancel','lostpointercapture'])canvas.addEventListener(event,()=>{drag=null});
canvas.addEventListener('wheel',e=>{e.preventDefault();const before=scale(),r=canvas.getBoundingClientRect();zoom=Math.max(.02,Math.min(200,zoom*Math.exp(-e.deltaY*.001)));const delta=1/before-1/scale();moveCenterView((e.clientX-r.left-width/2)*delta,(height/2-e.clientY+r.top)*delta);draw()},{passive:false});
select.addEventListener('change',()=>{treeEnabled.checked=false;fit()});
document.querySelector('#frame').addEventListener('click',fit);document.querySelector('#reset').addEventListener('click',()=>{yaw=-.65;pitch=-.9;fit()});
window.addEventListener('resize',resize);resize();fit();
</script></html>
'''
