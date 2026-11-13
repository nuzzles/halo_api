import json
from pathlib import Path

def b(w,v=0):return format(v,f'0{w}b') if w else ''
def handle(w,v=5,g=0):return b(w,v)+b(2,g)
def packet(k,body,baseline=None):return '1'+b(2,k)+('0' if baseline is None else '1'+b(7,baseline))+body
# Explicit test wire layouts, independently passed through the pinned upstream reader.
abs_short=b(32,0xfedcba98)+b(7,85)+'1'
abs_full=b(32,0x10203040)+b(7,12)+'0'+'0'+'0'+''.join(b(32,v) for v in [0,0x80000000,0x7fc12345,0x3f800000,0xffffffff]*3)+'1'+b(8,173)+b(4,9)+'1'+'1'+'1'+handle(8,212,0)
rel_short='1'+'1'+b(6,37)+'0'+'1'
rel_min='1'+'0'+'0'+'0'+'0'+'0'+'00000'+'0'+'0'+'0'
rel_full='0'+b(32,0x89abcdef)+'1'+b(7,63)+'0'+'1'+handle(13,4100,0)+'0'+b(10,1001)+'1'+handle(9,301,3)
rel_full+=''.join('1'+b(5,0)+b(32,0xdeadbeef)+b(5,31)+'1'+b(5,17)+b(15,0x4567) for _ in range(5))
rel_full+='0'+'1'+'1'+'0'
# All optional handles/gates in the full secondary prefix are specified explicitly.
sec_head=lambda state,unit: b(5,state)+b(2,3)+b(17,100001)+b(16,0xabcd)+unit
sec_base=sec_head(2,'1'+'1'+handle(9,300,0))+'1'+handle(13,4099,2)+b(2,2)+b(3,5)+b(3,2)
sec_base+='1'+handle(13,7777,3)+b(4,10)+b(4,4)+'0'+b(4,7)+b(4,9)
sec_base+=b(5,29)+b(3,6)+'0'+b(2,2)+'1'+b(6,47)+b(3,5)+b(3,3)+'0'+b(6,56)+'1'
sec_tail='0'+b(19,0x5abcd)+b(8,0xaa)+'0'+b(19,0x23456)+b(10,0x3ab)+'0'+b(19,0x12345)+b(8,0xda)+'1'+handle(13,8191,0)+b(7,101)
sec_tail_short='1'+b(8,27)+'1'+'1'+'0'+b(11,1799)+b(7,35)
cases=[]
def add(name,s,kinds,counts=None,secondary=None,index_bits=1,region_widths=None):
    s+='0'
    cases.append(dict(name=name,hex=int(s.ljust((len(s)+7)//8*8,'0'),2).to_bytes((len(s)+7)//8,'big').hex(),bit_len=len(s),kinds=kinds,command_counts=counts or [],secondary=secondary or [],index_bits=index_bits,region_widths=region_widths or {}))
add('empty','',[])
add('kind2-absent',packet(2,'0'),[2])
add('kind2-present',packet(2,'1'+b(13,0x1abc),127),[2])
add('commands-absent',packet(1,b(5,17)+'0'),[1],[None])
add('commands-zero-count',packet(1,b(5,17)+'1'+b(2,0)),[1],[0])
add('command-absolute-short',packet(1,b(5,17)+'1'+b(2,1)+abs_short),[1],[1])
add('command-absolute-full',packet(1,b(5,17)+'1'+b(2,1)+abs_full),[1],[1])
add('command-relative-minimal',packet(1,b(5,30)+'1'+b(2,1)+rel_min,0),[1],[1])
add('command-relative-full',packet(1,b(5,30)+'1'+b(2,1)+rel_full,127),[1],[1])
add('three-commands-baselines',packet(1,b(5,31)+'1'+b(2,3)+abs_short+rel_short+rel_full),[1],[3])
for name,unit in [('none','0'),('object','1'+'0'+handle(13,7000,0)),('unit','1'+'1'+handle(9,500,3))]:
    add('secondary-short-'+name,packet(0,b(5,21)+'0'+'1'+sec_head(0,unit)),[0],secondary=[True])
add('secondary-full-default',packet(0,b(5,21)+'0'+'1'+sec_base+'1'+b(22,1)+b(22,0x234567)+b(22,0x3fffff)+sec_tail),[0],secondary=[True])
add('secondary-full-world',packet(0,b(5,21)+'0'+'1'+sec_base+'0'+'0'+b(15,0x7123)+b(15,0x2345)+b(17,0x12345)+sec_tail_short),[0],secondary=[True])
add('secondary-full-region3',packet(0,b(5,21)+'0'+'1'+sec_base+'0'+'11'+b(19,0x71234)+b(19,0x23456)+b(16,0x1234)+sec_tail_short),[0],secondary=[True],index_bits=2,region_widths={'3':[19,19,16]})
add('mixed-wire-order',packet(2,'1'+b(13,543))+packet(1,b(5,17)+'1'+b(2,1)+abs_short)+packet(0,b(5,4)+'0'+'0'),[2,1,0],[1],[False])
Path(__file__).with_name('cases.json').write_text(json.dumps(cases,indent=2)+'\n')
