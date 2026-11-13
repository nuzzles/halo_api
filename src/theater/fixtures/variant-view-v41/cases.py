"""Author Bond variant trees independently of halo_api; no expected Rust output."""
import json
from pathlib import Path
import struct as binary

ROOT = Path(__file__).parent
def vi(n):
    b=bytearray()
    while n>=128: b.append(n%128+128); n>>=7
    return bytes(b)+bytes([n])
def f(t,i,b):
    return (bytes([t|i<<5]) if i<6 else bytes([t|192,i]))+b
def st(fields):
    b=b''.join(fields)+b'\0'
    return vi(len(b))+b
def seq(t,values):
    n=len(values)
    return (bytes([t|(n+1)<<5]) if n<7 else bytes([t])+vi(n))+b''.join(values)
def integer(n): return 16,vi(((n<<1)^(n>>31))&0xffffffff)
def ident(n): return integer(n if n<0x80000000 else n-0x100000000)
def tree(d):
    return st([f(10,k,tree(v)) if isinstance(v,dict) else f(v[0],k,v[1]) for k,v in d.items()])
def put(d,path,value):
    for k in path[:-1]:d=d.setdefault(k,{})
    d[path[-1]]=value
def option(hash,raw):
    return tree({**({0:{0:ident(hash)}} if hash is not None else {}),**({1:{0:(11,seq(3,[bytes([b]) for b in raw]))}} if raw is not None else {})})

settings=[('RespawnSeconds',[1,2,3]),('SuicidePenaltySeconds',[1,2,5]),('BetrayalPenaltySeconds',[1,2,6]),
('ScoreToWinRound',[1,7,0]),('RoundTimeLimitSeconds',[1,7,3]),('OvertimeSeconds',[1,7,9]),
('LastChanceEnabledInOvertime',[1,7,10]),('LastChanceSeconds',[1,7,11]),('DroppedWeaponsSeconds',[1,0,1,6]),
('BotsTeam1',[1,0,4,2]),('BotsTeam2',[1,0,4,5]),('SpawnProtection',[1,2,22,0]),('EarlyVictoryRounds',[1,7,5]),
('RoundLimit',[1,7,17]),('SwapSides',[1,7,18]),('RandomWeapons',[1,10,0,11,11]),('RandomEquipment',[1,10,1,8,12]),
('RandomVehicles',[1,10,2,10,10]),('Server30Fps',[5,0]),('TickRate',[5,7]),('KillcamEnabled',[1,0,5,0]),
('KillcamSeconds',[1,0,5,1]),('PlayOfTheGameEnabled',[1,0,5,2]),('MachinimaEnabled',[1,0,5,3])]
cases=[]
def add(name,base,instance=1,root=None):
    encoded=st([f(11,instance,seq(10,[base]))]) if root is None else root
    cases.append(dict(name=name,base_hex=base.hex(),hex=encoded.hex()))

d={}
for i,(_,path) in enumerate(settings):put(d,path,integer(i+1))
put(d,[3,21,0],(11,seq(10,[])))
for path,values in [([1,2,0],[4,10,4]),([1,3,0],[3,10]),([1,4,0],[0,2,3,4,5,6,8,99])]:put(d,path,(11,seq(16,[integer(x)[1] for x in values])))
weapons=[tree({0:{0:ident(0xfedcba98)},2:(2,b'\1')}),tree({0:{0:ident(0)},2:(2,b'\0')})]
loadouts=[tree({0:{0:ident(0xffffffff)},1:(11,seq(10,weapons)),2:{0:{0:ident(0x80000000)}},4:integer(5)}),tree({1:(11,seq(10,[tree({2:(2,b'\1')})]))})]
put(d,[1,6,1],(11,seq(10,loadouts)))
raws=[n.to_bytes(4,'little')+bytes([3,0xaa,i,0xff]) for i,n in enumerate([0,0x80000000,0x3eaaaaab,0x7fc12345,0x7f800000,0xff800000,1])]
raws += [bytes([0,0x83,0,0,2,1,2,3]),bytes([255,0,0,0,2,3,2,1]),bytes(range(8))]
put(d,[3,9,0],(11,seq(10,[option(0x185f4770 if i<2 else i,raw) for i,raw in enumerate(raws)])))
for i in range(4):add(f'complete/{i}',tree(d),i)
for path in [[1,2,0],[1,3,0],[1,4,0]]:
    at=d
    for k in path[:-1]:at=at[k]
    at[path[-1]]=(12,at[path[-1]][1])
add('complete-set/1',tree(d))
add('empty',tree({}))
add('present-empty-lists',tree({1:{2:{0:(11,seq(16,[]))},6:{1:(11,seq(10,[]))}},3:{9:{0:(11,seq(10,[]))}}}))
add('option-row-boundaries',tree({3:{9:{0:(11,seq(10,[option(7,None),option(None,raws[0]),option(7,raws[1]),option(7,raws[2])]))}}}))
for t,b in [(2,b'\xff'),(3,b'\xff'),(14,b'\xff'),(4,vi(65535)),(5,vi(0xffffffff)),(6,vi(0xffffffffffffffff)),(15,vi(65535)),(16,vi(0xffffffff)),(17,vi(0xffffffffffffffff)),(7,binary.pack('<I',0x80000000)),(7,binary.pack('<I',0x7fc12345)),(8,binary.pack('<Q',0x8000000000000000))]:
    add(f'scalar/{t}/{b.hex()}',tree({1:{2:{3:(t,b)}}}))
add('duplicate-setting',tree({1:{2:(10,st([f(14,3,b'\1'),f(14,3,b'\2')]))}}))
add('base-setting',tree({1:{2:(10,st([f(14,3,b'\1'),b'\1',f(14,3,b'\2')]))}}))
add('base-only-setting',tree({1:{2:(10,st([f(14,3,b'\1'),b'\1']))}}))
add('wrong-parent',tree({1:{2:integer(12)}}))
add('wrong-option-list',tree({3:{9:{0:integer(1)}}}))
add('wrong-option-length',tree({3:{9:{0:(11,seq(10,[option(7,b'1234567')]))}}}))
badrow=st([f(10,0,tree({0:ident(7)})),f(10,0,tree({0:ident(8)})),f(10,1,tree({0:(11,seq(3,[bytes([b]) for b in raws[0]]))}))])
add('duplicate-option-hash',tree({3:{9:{0:(11,seq(10,[badrow]))}}}))
add('ambiguous-instances',tree({}),root=st([f(11,0,seq(10,[tree({})])),f(11,1,seq(10,[tree({})]))]))
add('duplicate-instance-field',tree({}),root=st([f(11,0,seq(10,[])),f(11,0,seq(10,[tree({})]))]))
add('multiple-list-instances',tree({}),root=st([f(11,0,seq(10,[tree({}),tree({})]))]))
add('no-instance',tree({}),root=tree({}))

def text(b): return 9,vi(len(b))+b
def unsigned(n): return 5,vi(n)
def stat(i,name=b'Score',extra=None):
    return tree({**({0:unsigned(i)} if i is not None else {}),15:text(name),**(extra or {})})
def stat_variant(defs,columns,spawn=None):
    d={}
    if defs is not None:put(d,[3,21,0],(11,seq(10,defs)))
    if columns is not None:put(d,[2,30],(11,seq(10,columns)))
    if spawn is not None:put(d,[1,2,18,0],ident(spawn))
    return tree(d)
def column(i): return tree({0:unsigned(i)}) if i is not None else tree({})
add('stats/complete',stat_variant([
    stat(0,b'Score\0Raw',{1:integer(2),3:integer(7),12:integer(-1),13:integer(2),17:integer(3)}),
    stat(0xffffffff,'Précision'.encode(),{1:integer(0),3:integer(0),12:integer(1),13:integer(0),17:integer(0)}),
], [column(0xffffffff),column(0),column(0xffffffff)],0xfedcba98))
add('stats/missing-id',stat_variant([stat(None),stat(0)],[column(None),column(0)]))
add('stats/duplicates',stat_variant([stat(7),stat(7),stat(9)],[column(7),column(9),column(42),column(7)]))
add('stats/invalid-id',stat_variant([tree({0:text(b'bad')}),stat(7)],[column(7)]))
add('stats/invalid-text',stat_variant([stat(7,b'\xff\0\xed\xa0\x80')],[column(7)]))
add('stats/text-presence',stat_variant([tree({0:unsigned(7)}),stat(9,b'')],[column(7),column(9)]))
add('stats/absent-definitions',stat_variant(None,[column(7)]))
add('stats/empty-definitions',stat_variant([],[column(7)]))
add('stats/empty-columns',stat_variant([stat(7)],[]))
add('stats/absent-columns',stat_variant([stat(7)],None))
add('stats/wrong-definitions',tree({3:{21:{0:integer(9)}},2:{30:(11,seq(10,[column(7)]))}}))
add('stats/duplicate-fields',stat_variant([st([f(5,0,vi(7)),f(5,0,vi(9))])],[column(7)]))
add('stats/base-only-id',stat_variant([st([f(5,0,vi(7)),b'\1'])],[column(7)]))
add('stats/base-derived-id',stat_variant([st([f(5,0,vi(9)),b'\1',f(5,0,vi(7))])],[column(7)]))
add('stats/wide-integer',stat_variant([stat(7,b'wide',{17:(17,vi(0xffffffffffffffff))})],[column(7)]))
add('stats/unsigned-numbers',stat_variant([stat(7,b'unsigned',{1:(3,b'\xff'),3:unsigned(0xffffffff),12:(14,b'\xff'),13:(3,b'\2'),17:(4,vi(65535))})],[column(7)]))


def blob(b): return 11,seq(3,[bytes([x]) for x in b])
def string_row(offsets,flag=True):
    return tree({**({0:(2,bytes([flag]))} if flag is not None else {}),
                 **({1:(11,seq(16,[integer(x)[1] for x in offsets]))} if offsets is not None else {})})
def strings(data,rows):
    return tree({3:{0:{**({0:{0:blob(data)}} if data is not None else {}),
                      **({1:{0:(11,seq(10,rows))}} if rows is not None else {})}}})
add('strings/shared',strings(b'Name\0Description\0\xffx\0tail\0',[
    string_row([0,1,0]),string_row([5],False),string_row([17,16,4,19],None),string_row([])]))
add('strings/invalid-offsets',strings(b'a\0tail',[string_row([-1,6,2147483647,-2147483648,2,3,5])]))
add('strings/missing-blob',strings(None,[string_row([0])]))
add('strings/missing-entries',strings(b'a\0',None))
add('strings/missing-offsets',strings(b'a\0',[string_row(None)]))
add('strings/empty',strings(b'',[]))
add('strings/empty-blob',strings(b'',[string_row([0])]))
add('strings/wrong-blob',tree({3:{0:{0:{0:text(b'a\0')},1:{0:(11,seq(10,[string_row([0])]))}}}}))
add('strings/wrong-entries',tree({3:{0:{0:{0:blob(b'a\0')},1:{0:integer(0)}}}}))
add('strings/wrong-offsets',strings(b'a\0',[tree({1:(11,seq(5,[vi(0)]))})]))
add('strings/wrong-flag',strings(b'a\0',[tree({0:integer(1),1:(11,seq(16,[vi(0)]))})]))
add('strings/duplicate-offsets',strings(b'a\0',[st([f(11,1,seq(16,[vi(0)])),f(11,1,seq(16,[vi(0)]))])]))
add('strings/base-only-offsets',strings(b'a\0',[st([f(11,1,seq(16,[vi(0)])),b'\1'])]))
add('strings/base-derived-offsets',strings(b'a\0',[st([f(11,1,seq(16,[integer(-1)[1]])),b'\1',f(11,1,seq(16,[vi(0)]))])]))
# An ordinary reference-convention table for the unchanged upstream text API.
data=bytearray();rows=[]
for word in [b'Fixture Name',b'Fixture Description',b'Score',b'Rounds Won',b'Kills']:
    offsets=[]
    for lang in range(18):
        offsets.append(len(data));data.extend(word+b'\0')
    rows.append(string_row(offsets))
add('strings/reference',strings(data,rows))

(ROOT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
