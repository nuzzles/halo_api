//! Preserve JSON floating negative zero while admitting Go's integer spelling -0.
use super::callouts_json::Shape;
struct Scan<'a> {
    bytes: &'a [u8],
    at: usize,
    remove: Vec<usize>,
}
impl Scan<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.at += 1;
        }
    }
    fn string(&mut self) -> Option<(usize, usize)> {
        if self.bytes.get(self.at) != Some(&b'"') {
            return None;
        }
        let start = self.at;
        self.at += 1;
        loop {
            match self.bytes.get(self.at)? {
                b'"' => {
                    self.at += 1;
                    return Some((start, self.at));
                }
                b'\\' => self.at += 2,
                _ => self.at += 1,
            }
        }
    }
    fn value(&mut self, shape: Option<&Shape>, depth: usize) -> Option<()> {
        if depth > 128 {
            return None;
        }
        self.space();
        match self.bytes.get(self.at)? {
            b'"' => {
                self.string()?;
            }
            b'{' => {
                self.at += 1;
                self.space();
                if self.bytes.get(self.at) == Some(&b'}') {
                    self.at += 1;
                    return Some(());
                }
                loop {
                    self.space();
                    let (start, end) = self.string()?;
                    let child = match shape {
                        Some(Shape::Struct(fields)) => {
                            let key: String =
                                serde_json::from_slice(&self.bytes[start..end]).ok()?;
                            let key = super::map_catalog::field_name(&key);
                            fields
                                .iter()
                                .find(|(name, _)| name.eq_ignore_ascii_case(&key))
                                .map(|(_, s)| *s)
                        }
                        Some(Shape::Map(child)) => Some(*child),
                        _ => None,
                    };
                    self.space();
                    if self.bytes.get(self.at) != Some(&b':') {
                        return None;
                    }
                    self.at += 1;
                    self.value(child, depth + 1)?;
                    self.space();
                    match self.bytes.get(self.at)? {
                        b',' => self.at += 1,
                        b'}' => {
                            self.at += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
            }
            b'[' => {
                self.at += 1;
                self.space();
                if self.bytes.get(self.at) == Some(&b']') {
                    self.at += 1;
                    return Some(());
                }
                let child = match shape {
                    Some(Shape::Slice(child)) => Some(*child),
                    _ => None,
                };
                loop {
                    self.value(child, depth + 1)?;
                    self.space();
                    match self.bytes.get(self.at)? {
                        b',' => self.at += 1,
                        b']' => {
                            self.at += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
            }
            _ => {
                let start = self.at;
                while self.bytes.get(self.at).is_some_and(|b| {
                    !matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b',' | b']' | b'}')
                }) {
                    self.at += 1;
                }
                if self.at == start {
                    return None;
                }
                if matches!(shape, Some(Shape::Int)) && self.bytes[start..self.at] == *b"-0" {
                    self.remove.push(start);
                }
            }
        }
        Some(())
    }
}
pub(super) fn normalize(bytes: Vec<u8>, shape: &Shape) -> Vec<u8> {
    let mut scan = Scan {
        bytes: &bytes,
        at: 0,
        remove: vec![],
    };
    if scan.value(Some(shape), 0).is_none() {
        return bytes;
    }
    scan.space();
    if scan.at != bytes.len() || scan.remove.is_empty() {
        return bytes;
    }
    let remove = scan.remove;
    let mut removed = remove.into_iter().peekable();
    bytes
        .into_iter()
        .enumerate()
        .filter_map(|(i, b)| {
            if removed.peek() == Some(&i) {
                removed.next();
                None
            } else {
                Some(b)
            }
        })
        .collect()
}
