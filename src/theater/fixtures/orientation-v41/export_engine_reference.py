"""Execute unchanged pinned LevelUp roll projection; this is an f32 oracle."""
import hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
PIN='413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS={'orientation_frame.go':'fe2f382c1bd333ebea227d205e7196cf7662537b','components_managed_object.go':'8e0e8742f54a0f81fb222ebd544b3f8e97ae51ad'}
sources={}
for name,expected in BLOBS.items():
    b=(Path(sys.argv[1])/name).read_bytes();assert hashlib.sha1(f'blob {len(b)}\0'.encode()+b).hexdigest()==expected
    sources[name]=b.decode()
def extract(s,prefix):
    a=s.index(prefix);p=s.index('{',a)+1;depth=1
    while depth:depth+=(s[p]=='{')-(s[p]=='}');p+=1
    return s[a:p]
code=extract(sources['orientation_frame.go'],'func RollAngleFromRaw(')+'\n'+extract(sources['components_managed_object.go'],'func dequantMidpoint(')
main=r'''
package main
import("math";"encoding/json";"os";"fmt")
func main(){var cases []struct{Name string;Roll *uint32};b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)};for _,c:=range cases{if c.Roll!=nil{fmt.Printf("%s\t%d\n",c.Name,math.Float32bits(RollAngleFromRaw(*c.Roll,30)))}}}
'''
with tempfile.TemporaryDirectory(prefix='halo-engine-angle-') as tmp:
    p=Path(tmp);(p/'main.go').write_text(main+code)
    out=subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go',str(Path(__file__).with_name('cases.json').resolve())],cwd=p,text=True,env={**os.environ,"GOCACHE":os.environ.get("GOCACHE",str(p/"cache"))})
rows=[]
for line in out.splitlines():
    name,value=line.split('\t');rows.append(dict(name=name,roll_bits=int(value)))
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=rows),indent=2))
