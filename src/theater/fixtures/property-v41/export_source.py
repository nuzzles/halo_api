"""Execute pinned original Go variant/default readers and export original film vectors."""
import hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).parent
PIN='413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS={'components_managed_property.go':'8f7093171e8b48f7feb8c9f85258dd0cce24dee7','default_state_arch.go':'53f0146423b00f404c41791e5ca00f4eb00a6ea9','ti13_vecteurs_test.go':'926944ccbe1dca6abdcc14843a774d4fa8e51e70'}
sources={}
for name,blob in BLOBS.items():
    path=Path(sys.argv[1])/name if name!='ti13_vecteurs_test.go' else Path(sys.argv[2])
    raw=path.read_bytes();assert hashlib.sha1(f'blob {len(raw)}\0'.encode()+raw).hexdigest()==blob
    sources[name]=raw.decode()
def extract(s,prefix):
    start=s.index(prefix);i=s.index('{',start);depth=1;j=i+1
    while depth:
        depth+=(s[j]=='{')-(s[j]=='}');j+=1
    return s[start:j]
p=sources['components_managed_property.go'];d=sources['default_state_arch.go'];v=sources['ti13_vecteurs_test.go']
start=p.index('const (\n\tManagedPropertyTagVide');end=p.index('\n)',start)+2
selected=p[start:end]+'\n'+'\n'.join(extract(p,f) for f in ['func managedPropertyPayloadBits(','func lireVariantDePropriete('])+'\n'+'\n'.join(extract(d,f) for f in ['func consumeVersionPrefix(','func consumeDefaultStateTI13('])+'\n'+'\n'.join(extract(v,f) for f in ['type ti13VecteurFige struct','var ti13VecteursModeA =','var ti13VecteursModeB ='])
harness=r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt")
type Lecteur struct {data []byte; at int; trace [][2]int}
func(r *Lecteur) ReadBits(n uint)uint64 {r.trace=append(r.trace,[2]int{r.at,int(n)});var v uint64;for i:=uint(0);i<n;i++ {if r.at>=len(r.data)*8{panic("truncated fixture")};v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++};return v}
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func main(){out:=map[string]any{};for _,kind:=range []string{"components","defaults"}{var cases []struct{Name,Hex,Component string;BitLen int `json:"bit_len"`};b,e:=os.ReadFile(kind+".json");if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)};rows:=[]map[string]any{};for _,c:=range cases{b,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)};r:=&Lecteur{data:b};if kind=="defaults"{consumeDefaultStateTI13(r)}else{lireVariantDePropriete(r,c.Component=="managed-object-property-component")};if r.at!=c.BitLen{panic(c.Name)};rows=append(rows,map[string]any{"name":c.Name,"end":r.at,"trace":r.trace})};out[kind]=rows}
 rows:=[]map[string]any{};for _,group:=range [][]ti13VecteurFige{ti13VecteursModeA,ti13VecteursModeB}{for _,v:=range group{b,e:=hex.DecodeString(fmt.Sprintf("%016x",v.raw));if e!=nil{panic(e)};r:=&Lecteur{data:b};tag,payload,width:=lireVariantDePropriete(r,v.modeA);if int(tag)!=v.tag || payload!=v.payload || width!=v.bits || r.at!=4+v.bits{panic("captured vector differs from original reader")};rows=append(rows,map[string]any{"film":v.film,"chunk":v.chunk,"packet":v.pkt,"bit":v.bit,"slot":v.slot,"row":v.i,"masked":!v.modeA,"hex":fmt.Sprintf("%016x",v.raw),"tag":v.tag,"width":v.bits,"payload":v.payload,"chained":v.chaine,"comment":v.commentaire})}};out["captured"]=rows;if e:=json.NewEncoder(os.Stdout).Encode(out);e!=nil{panic(e)}}
'''
with tempfile.TemporaryDirectory(prefix='halo-property-oracle-') as tmp:
    root=Path(tmp);(root/'main.go').write_text(harness);(root/'source.go').write_text('package main\n'+selected)
    for name in ['components','defaults']:(root/(name+'.json')).write_bytes((ROOT/(name+'.json')).read_bytes())
    env=dict(os.environ);env.setdefault('GOCACHE',str(root/'cache'))
    result=json.loads(subprocess.check_output([env.get('GO_BIN','go'),'run','main.go','source.go'],cwd=root,env=env,text=True))
print(json.dumps(dict(commit=PIN,blobs=BLOBS,**result),indent=2))
