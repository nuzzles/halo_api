//! Ordered JSON syntax tree for native facts. Flat storage avoids recursion at
//! Go's 10,000-container depth limit, including when dropping malformed inputs.
#[derive(Debug)]
pub(super) enum JsonNode {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<usize>),
    Object(Vec<(String, usize)>),
}
pub(super) struct JsonTree {
    pub nodes: Vec<JsonNode>,
    pub root: usize,
}
struct Frame {
    node: usize,
    state: u8,
    key: Option<String>,
    object: bool,
}
struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    nodes: Vec<JsonNode>,
    stack: Vec<Frame>,
    root: Option<usize>,
}
fn quoted(c: u8) -> String {
    match c {
        b'\'' => "'\\''".into(),
        b'"' => "'\"'".into(),
        b'\\' => "'\\\\'".into(),
        7 => "'\\a'".into(),
        8 => "'\\b'".into(),
        12 => "'\\f'".into(),
        b'\n' => "'\\n'".into(),
        b'\r' => "'\\r'".into(),
        b'\t' => "'\\t'".into(),
        11 => "'\\v'".into(),
        0..=31 | 127 => format!("'\\x{c:02x}'"),
        128..=160 | 173 => format!("'\\u{c:04x}'"),
        _ => format!("'{}'", char::from(c)),
    }
}
fn invalid(c: u8, context: &str) -> String {
    format!("invalid character {} {context}", quoted(c))
}
impl Parser<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|c| matches!(c, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.at += 1;
        }
    }
    fn bad(&self, context: &str) -> String {
        invalid(self.bytes.get(self.at).copied().unwrap_or(b' '), context)
    }
    fn string(&mut self) -> Result<String, String> {
        let start = self.at;
        self.at += 1;
        loop {
            let c = *self
                .bytes
                .get(self.at)
                .ok_or("unexpected end of JSON input")?;
            self.at += 1;
            match c {
                b'"' => {
                    let normalized =
                        super::map_catalog::native_catalog_json(&self.bytes[start..self.at]);
                    return serde_json::from_slice(&normalized).map_err(|e| e.to_string());
                }
                b'\\' => {
                    let c = self.bytes.get(self.at).copied().unwrap_or(b' ');
                    if c == b'u' {
                        self.at += 1;
                        for _ in 0..4 {
                            if !self.bytes.get(self.at).is_some_and(u8::is_ascii_hexdigit) {
                                return Err(self.bad("in \\u hexadecimal character escape"));
                            }
                            self.at += 1;
                        }
                    } else if matches!(c, b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') {
                        self.at += 1;
                    } else {
                        return Err(self.bad("in string escape code"));
                    }
                }
                0..=31 => return Err(invalid(c, "in string literal")),
                _ => {}
            }
        }
    }
    fn number(&mut self) -> Result<String, String> {
        let start = self.at;
        if self.bytes.get(self.at) == Some(&b'-') {
            self.at += 1;
        }
        match self.bytes.get(self.at) {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => {
                while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                    self.at += 1;
                }
            }
            _ => return Err(self.bad("in numeric literal")),
        }
        if self.bytes.get(self.at) == Some(&b'.') {
            self.at += 1;
            if !self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                return Err(self.bad("after decimal point in numeric literal"));
            }
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                self.at += 1;
            }
        }
        if matches!(self.bytes.get(self.at), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.bytes.get(self.at), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if !self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                return Err(self.bad("in exponent of numeric literal"));
            }
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                self.at += 1;
            }
        }
        Ok(std::str::from_utf8(&self.bytes[start..self.at])
            .unwrap()
            .to_owned())
    }
    fn value(&mut self) -> Result<(), String> {
        let c = *self
            .bytes
            .get(self.at)
            .ok_or("unexpected end of JSON input")?;
        let node = match c {
            b'{' | b'[' => {
                if self.stack.len() == 10000 {
                    return Err(invalid(c, "exceeded max depth"));
                }
                self.at += 1;
                if c == b'{' {
                    JsonNode::Object(Vec::new())
                } else {
                    JsonNode::Array(Vec::new())
                }
            }
            b'"' => JsonNode::String(self.string()?),
            b'-' | b'0'..=b'9' => JsonNode::Number(self.number()?),
            b'n' | b't' | b'f' => {
                let literal = match c {
                    b'n' => "null",
                    b't' => "true",
                    _ => "false",
                };
                self.at += 1;
                for &expected in &literal.as_bytes()[1..] {
                    if self.bytes.get(self.at) != Some(&expected) {
                        return Err(self.bad(&format!(
                            "in literal {literal} (expecting '{}')",
                            char::from(expected)
                        )));
                    }
                    self.at += 1;
                }
                match c {
                    b'n' => JsonNode::Null,
                    b't' => JsonNode::Bool(true),
                    _ => JsonNode::Bool(false),
                }
            }
            _ => return Err(invalid(c, "looking for beginning of value")),
        };
        let id = self.nodes.len();
        self.nodes.push(node);
        if let Some(frame) = self.stack.last_mut() {
            match &mut self.nodes[frame.node] {
                JsonNode::Object(pairs) => {
                    pairs.push((frame.key.take().unwrap(), id));
                    frame.state = 4;
                }
                JsonNode::Array(values) => {
                    values.push(id);
                    frame.state = 2;
                }
                _ => unreachable!(),
            }
        } else {
            self.root = Some(id);
        }
        if matches!(c, b'{' | b'[') {
            self.stack.push(Frame {
                node: id,
                state: 0,
                key: None,
                object: c == b'{',
            });
        }
        Ok(())
    }
    fn parse(mut self) -> Result<JsonTree, String> {
        loop {
            self.space();
            let Some(frame) = self.stack.last() else {
                if self.root.is_none() {
                    self.value()?;
                    continue;
                }
                if self.at != self.bytes.len() {
                    return Err(self.bad("after top-level value"));
                }
                return Ok(JsonTree {
                    nodes: self.nodes,
                    root: self.root.unwrap(),
                });
            };
            let c = *self
                .bytes
                .get(self.at)
                .ok_or("unexpected end of JSON input")?;
            match (frame.object, frame.state) {
                (false, 0) if c == b']' => {
                    self.at += 1;
                    self.stack.pop();
                }
                (false, 0 | 1) => self.value()?,
                (false, 2) => match c {
                    b',' => {
                        self.at += 1;
                        self.stack.last_mut().unwrap().state = 1;
                    }
                    b']' => {
                        self.at += 1;
                        self.stack.pop();
                    }
                    _ => return Err(self.bad("after array element")),
                },
                (true, 0) if c == b'}' => {
                    self.at += 1;
                    self.stack.pop();
                }
                (true, 0 | 1) => {
                    if c != b'"' {
                        return Err(self.bad("looking for beginning of object key string"));
                    }
                    let key = self.string()?;
                    let f = self.stack.last_mut().unwrap();
                    f.key = Some(key);
                    f.state = 2;
                }
                (true, 2) => {
                    if c != b':' {
                        return Err(self.bad("after object key"));
                    }
                    self.at += 1;
                    self.stack.last_mut().unwrap().state = 3;
                }
                (true, 3) => self.value()?,
                (true, 4) => match c {
                    b',' => {
                        self.at += 1;
                        self.stack.last_mut().unwrap().state = 1;
                    }
                    b'}' => {
                        self.at += 1;
                        self.stack.pop();
                    }
                    _ => return Err(self.bad("after object key:value pair")),
                },
                _ => unreachable!(),
            }
        }
    }
}
pub(super) fn parse(bytes: &[u8]) -> Result<JsonTree, String> {
    Parser {
        bytes,
        at: 0,
        nodes: Vec::new(),
        stack: Vec::new(),
        root: None,
    }
    .parse()
}
