"""Authored orientation words; independent of halo_api decoding."""
import json,struct

def bits(n,v): return format(v,f'0{n}b')
cases=[]
def case(name,wire,kind):
    padded=wire+'0'*(-len(wire)%8)
    cases.append(dict(name=name,hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=len(wire),kind=kind,version=2))
def config(name,roll,default=True,prefix='001'):
    case(name,prefix+('1' if default else '0'+bits(30,0x1234567))+bits(30,roll),'config')
    cases[-1]['roll']=roll
config('config-quarter',1<<28)
def raw(name,f,u):
    b=struct.pack('<6f',*f,*u)
    case(name,'10'+''.join(bits(8,v) for v in b),'raw')
raw('raw-east',[1.,0.,0.],[0.,0.,1.])
case('delta-keep-absent','01010','delta')
for name,roll in [('zero',0),('half',1<<29),('last',(1<<30)-1)]:config('config-'+name,roll)
config('config-up',17,False)
config('raw-overridden-default',33,True,'11')
config('raw-overridden-up',37,False,'11')
config('delta-overridden',41,True,'011')
case('nominal-default','0001'+bits(8,100),'nominal')
case('nominal-up','0000'+bits(19,123)+bits(8,200),'nominal')
for keep,delta in [(1,0),(0,0),(0,1)]:
    for roll in [0,1]:
        wire='010'+bits(1,keep)
        if not keep:wire+=bits(1,delta)+(bits(8,0xe3) if delta else bits(19,0x54321))
        wire+=bits(1,roll)+(bits(4,13) if roll else '')
        case(f'delta-{keep}-{delta}-{roll}',wire,'delta')
raw('raw-north',[0.,1.,0.],[0.,0.,1.])
raw('raw-west',[-1.,0.,0.],[0.,0.,1.])
raw('raw-nan',[float('nan'),0.,0.],[0.,0.,1.])
raw('raw-parallel',[1.,0.,0.],[1.,0.,0.])
raw('raw-vertical',[0.,0.,1.],[1.,0.,0.])
print(json.dumps(cases,indent=2))
