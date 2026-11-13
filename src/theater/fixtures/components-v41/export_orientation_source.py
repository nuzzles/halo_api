"""Run original pinned LevelUp orientation functions on the selected cases.

Only the three orientation decoders and output type are extracted unchanged.
The unselected normal absolute helper is a panic stub, so no replacement grammar
can accidentally validate a case. Source Git blob is checked before extraction.
"""
import hashlib,json,os,re,subprocess,sys,tempfile
from pathlib import Path
PIN='413971bcc485be53a2fa8cf229e170539a5146f8'
BLOB='a32dc647e20c3d32146358145ad93b979adbe83e'
raw=Path(sys.argv[1]).read_bytes();assert hashlib.sha1(f'blob {len(raw)}\0'.encode()+raw).hexdigest()==BLOB
source=raw.decode()
cases_path=Path(sys.argv[2]) if len(sys.argv)>2 else Path(__file__).with_name("orientation-source.json")
def extract(prefix):
    start=source.index(prefix);open_=source.index('{',start);depth=1;i=open_+1
    while depth:
        depth+=(source[i]=='{')-(source[i]=='}');i+=1
    return source[start:i]
selected='\n\n'.join(extract(p) for p in ['type FwdUpDynPrec struct','func decodeObjectForwardAndUpDynPrec(','func decodeFwdUpDynPrecDelta(','func decodeFwdUpDynPrecConfig('])
harness=r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt")
const fwdUpDynPrecMode2Bits=192
type Lecteur struct { data []byte; at int; trace [][2]int }
func(r *Lecteur) ReadBits(n uint)uint64 { r.trace=append(r.trace,[2]int{r.at,int(n)}); var v uint64; for i:=uint(0);i<n;i++{ if r.at>=len(r.data)*8{panic("truncated fixture")};v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++ };return v }
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func decodeObjectForwardAndUp(r *Lecteur)(uint32,bool,uint32){panic("unselected helper")}
func main(){var cases []struct{Name,Hex string;Version uint32;BitLen int `json:"bit_len"`;OrientationBit int `json:"orientation_bit"`;OrientationEnd int `json:"orientation_end"`};b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{d,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)};r:=&Lecteur{data:d};if c.OrientationEnd>0{r.at=c.OrientationBit;decodeFwdUpDynPrecConfig(r);if r.at!=c.OrientationEnd{panic(c.Name)}}else{_,ok:=decodeObjectForwardAndUpDynPrec(r,c.Version);if !ok||r.at!=c.BitLen{panic(c.Name)}};t,_:=json.Marshal(r.trace);fmt.Printf("%s\t%d\t%s\n",c.Name,r.at,t)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-orientation-oracle-') as tmp:
    p=Path(tmp);(p/'main.go').write_text(harness);(p/'orientation.go').write_text('package main\n'+selected)
    (p/'cases.json').write_bytes(cases_path.read_bytes())
    out=subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go','orientation.go','cases.json'],cwd=p,text=True)
rows=[]
for line in out.splitlines():
    name,end,trace=line.split('\t');rows.append(dict(name=name,end=int(end),trace=json.loads(trace)))
assert len(rows)==len(json.loads(cases_path.read_text()))
print(json.dumps(dict(commit=PIN,blob=BLOB,cases=rows),indent=2))
