"""Trace original pinned libfilmshell CBv2 value boundaries on valid fixtures."""
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
PIN = '18067987ebc3628ad499c819c6aee814692df43b'

def main():
    cases = json.loads(Path(__file__).with_name('cases.json').read_text())
    archive = subprocess.check_output(['git', '-C', sys.argv[1], 'archive', PIN])
    with tempfile.TemporaryDirectory(prefix='filmshell-bond-') as tmp:
        root = Path(tmp)
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(root, filter='data')
        path = root / 'src/variant.rs'
        source = path.read_text()
        signature = 'fn value(b: &[u8], p: &mut usize, t: u8, path: &mut Vec<u32>, f: &mut impl FnMut(&[u32], i64)) -> Option<()> {'
        assert source.count(signature) == 1
        wrapper = signature + '''
    let start = *p;
    let result = original_value(b, p, t, path, f);
    #[cfg(test)] oracle::trace(t, start, *p);
    result
}
'''
        source = source.replace(signature, wrapper + signature.replace('fn value(', 'fn original_value('))
        source += r'''
#[cfg(test)]
mod oracle {
    use super::*;
    use std::cell::RefCell;
    thread_local! { static TRACE: RefCell<Vec<(u8,usize,usize)>> = const { RefCell::new(Vec::new()) }; }
    pub fn trace(t:u8, start:usize, end:usize) { TRACE.with(|v|v.borrow_mut().push((t,start,end))); }
    #[test]
    fn export_bond() {
        for line in include_str!("bond.tsv").lines() {
            let words:Vec<_> = line.split('\t').collect();
            let b:Vec<_> = words[1].as_bytes().chunks(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect();
            let mut p = 0;
            TRACE.with(|v|v.borrow_mut().clear());
            let result = value(&b, &mut p, 10, &mut Vec::new(), &mut |_,_|{});
            let trace = TRACE.with(|v|v.borrow().iter().map(|(t,s,e)|format!("[{t},{s},{e}]")).collect::<Vec<_>>().join(","));
            println!("ORACLE\t{}\t{}\t{}\t[{}]", words[0], p, result.is_some(), trace);
        }
    }
}
'''
        path.write_text(source)
        (root / 'src/bond.tsv').write_text('\n'.join(f"{c['name']}\t{c['hex']}" for c in cases))
        output = subprocess.check_output(['cargo', '+stable', 'test', '--offline', '--lib', 'export_bond', '--', '--nocapture'],cwd=root,text=True)
        rows=[]
        for line in output.splitlines():
            if line.startswith('ORACLE\t'):
                _,name,end,complete,trace=line.split('\t')
                case=cases[len(rows)]
                assert name==case['name'] and int(end)*8==case['bit_len'] and complete=='true'
                rows.append(dict(name=name,end_bit=int(end)*8,trace=[[t,s*8,e*8] for t,s,e in json.loads(trace)]))
        assert len(rows)==len(cases)
        print(json.dumps(dict(commit=PIN,cases=rows),indent=2))
if __name__=='__main__': main()
