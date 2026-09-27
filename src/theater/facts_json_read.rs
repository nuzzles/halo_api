//! Schema-directed Go JSON updates for facts. Syntax is checked first, before
//! any type error; slices retain backing cells across repeated struct fields.
use super::facts_json_parse::{JsonNode, JsonTree};
use std::collections::BTreeMap;
pub(super) enum MapKey {
    String,
    Uint32,
    Int,
}
impl MapKey {
    fn name(&self) -> &str {
        match self {
            Self::String => "string",
            Self::Uint32 => "uint32",
            Self::Int => "int",
        }
    }
}
pub(super) enum Shape {
    Bool,
    Signed(u8, &'static str),
    Unsigned(u8, &'static str),
    Float32,
    String(&'static str),
    Pointer(&'static Shape),
    Struct {
        name: &'static str,
        fields: &'static [(&'static str, &'static Shape)],
    },
    Slice(&'static Shape),
    Array(&'static Shape, usize),
    NamedArray(&'static Shape, usize, &'static str),
    Map {
        key: MapKey,
        value: &'static Shape,
    },
}
impl Shape {
    fn name(&self) -> String {
        match self {
            Self::Bool => "bool".into(),
            Self::Signed(_, name) | Self::Unsigned(_, name) => name.to_string(),
            Self::Float32 => "float32".into(),
            Self::String(name) => name.to_string(),
            Self::Pointer(child) => format!("*{}", child.name()),
            Self::Struct { name, .. } => name.to_string(),
            Self::Slice(t) => format!("[]{}", t.name()),
            Self::NamedArray(_, _, name) => name.to_string(),
            Self::Array(t, n) => format!("[{n}]{}", t.name()),
            Self::Map { key, value } => format!("map[{}]{}", key.name(), value.name()),
        }
    }
}
#[derive(Debug, Clone)]
pub(super) enum Cell {
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Float(f32),
    String(String),
    Pointer(Option<Box<Cell>>),
    Struct(BTreeMap<&'static str, Cell>),
    Map(Option<BTreeMap<String, Cell>>),
    Sequence {
        backing: Vec<Cell>,
        len: usize,
        present: bool,
    },
}
impl Cell {
    fn zero(shape: &Shape) -> Self {
        match shape {
            Shape::Bool => Self::Bool(false),
            Shape::Signed(..) => Self::Signed(0),
            Shape::Unsigned(..) => Self::Unsigned(0),
            Shape::Float32 => Self::Float(0.),
            Shape::String(_) => Self::String(String::new()),
            Shape::Pointer(_) => Self::Pointer(None),
            Shape::Struct { fields, .. } => Self::Struct(
                fields
                    .iter()
                    .map(|(name, s)| (*name, Self::zero(s)))
                    .collect(),
            ),
            Shape::Map { .. } => Self::Map(None),
            Shape::Slice(_) => Self::Sequence {
                backing: Vec::new(),
                len: 0,
                present: false,
            },
            Shape::Array(t, n) | Shape::NamedArray(t, n, _) => Self::Sequence {
                backing: (0..*n).map(|_| Self::zero(t)).collect(),
                len: *n,
                present: true,
            },
        }
    }
    pub fn field(&self, name: &str) -> &Self {
        let Self::Struct(fields) = self else {
            unreachable!()
        };
        &fields[name]
    }
    fn nil(&self) -> bool {
        matches!(
            self,
            Self::Map(None) | Self::Sequence { present: false, .. }
        )
    }
}
fn error(actual: &str, shape: &Shape, owner: Option<&str>, path: &str) -> String {
    let location = if let Some(owner) = owner {
        format!(
            "Go struct field {}.{path}",
            owner.rsplit('.').next().unwrap()
        )
    } else {
        "Go value".into()
    };
    format!(
        "json: cannot unmarshal {actual} into {location} of type {}",
        shape.name()
    )
}
fn kind(n: &JsonNode) -> &'static str {
    match n {
        JsonNode::Null => "null",
        JsonNode::Bool(_) => "bool",
        JsonNode::Number(_) => "number",
        JsonNode::String(_) => "string",
        JsonNode::Array(_) => "array",
        JsonNode::Object(_) => "object",
    }
}
fn update(
    shape: &'static Shape,
    cell: &mut Cell,
    tree: &JsonTree,
    node: usize,
    owner: Option<&str>,
    path: &str,
) -> Result<(), String> {
    let value = &tree.nodes[node];
    if matches!(value, JsonNode::Null) {
        if matches!(
            shape,
            Shape::Map { .. } | Shape::Slice(_) | Shape::Pointer(_)
        ) {
            *cell = Cell::zero(shape);
        }
        return Ok(());
    }
    let bad = || error(kind(value), shape, owner, path);
    match (shape, cell, value) {
        (Shape::Pointer(child), Cell::Pointer(out), _) => {
            let target = out.get_or_insert_with(|| Box::new(Cell::zero(child)));
            update(child, target, tree, node, owner, path)?;
        }
        (Shape::String(_), Cell::String(out), JsonNode::String(v)) => *out = v.clone(),
        (Shape::Bool, Cell::Bool(out), JsonNode::Bool(v)) => *out = *v,
        (Shape::Signed(bits, _), Cell::Signed(out), JsonNode::Number(v)) => {
            let parsed = v.parse::<i64>().ok().filter(|n| {
                *bits == 64 || (*n >= -(1i64 << (*bits - 1)) && *n < (1i64 << (*bits - 1)))
            });
            *out = parsed.ok_or_else(|| error(&format!("number {v}"), shape, owner, path))?;
        }
        (Shape::Unsigned(bits, _), Cell::Unsigned(out), JsonNode::Number(v)) => {
            let parsed = (!v.starts_with(['-', '+']))
                .then(|| v.parse::<u64>().ok())
                .flatten()
                .filter(|n| *bits == 64 || *n < (1u64 << *bits));
            *out = parsed.ok_or_else(|| error(&format!("number {v}"), shape, owner, path))?;
        }
        (Shape::Float32, Cell::Float(out), JsonNode::Number(v)) => {
            *out = v
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| error(&format!("number {v}"), shape, owner, path))?
        }
        (Shape::Struct { name, fields }, Cell::Struct(out), JsonNode::Object(pairs)) => {
            for (key, id) in pairs {
                let field = fields.iter().find(|(name, _)| *name == key).or_else(|| {
                    let folded = super::map_catalog::field_name(key);
                    fields
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(&folded))
                });
                if let Some((field, child)) = field {
                    let path = if path.is_empty() {
                        field.to_string()
                    } else {
                        format!("{path}.{field}")
                    };
                    update(
                        child,
                        out.get_mut(field).unwrap(),
                        tree,
                        *id,
                        Some(name),
                        &path,
                    )?;
                }
            }
        }
        (
            Shape::Map {
                key: key_type,
                value: child,
            },
            Cell::Map(out),
            JsonNode::Object(pairs),
        ) => {
            let map = out.get_or_insert_with(BTreeMap::new);
            for (key, id) in pairs {
                let mut value = Cell::zero(child);
                update(child, &mut value, tree, *id, owner, path)?;
                let key = if matches!(key_type, MapKey::Uint32) {
                    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(error(
                            &format!("number {key}"),
                            &Shape::Unsigned(32, "uint32"),
                            owner,
                            path,
                        ));
                    }
                    key.parse::<u32>()
                        .map_err(|_| {
                            error(
                                &format!("number {key}"),
                                &Shape::Unsigned(32, "uint32"),
                                owner,
                                path,
                            )
                        })?
                        .to_string()
                } else if matches!(key_type, MapKey::Int) {
                    key.parse::<i64>()
                        .map_err(|_| {
                            error(
                                &format!("number {key}"),
                                &Shape::Signed(64, "int"),
                                owner,
                                path,
                            )
                        })?
                        .to_string()
                } else {
                    key.clone()
                };
                map.insert(key, value);
            }
        }
        (
            Shape::Slice(child) | Shape::Array(child, _) | Shape::NamedArray(child, _, _),
            Cell::Sequence {
                backing,
                len,
                present,
            },
            JsonNode::Array(values),
        ) => {
            *present = true;
            let count = match shape {
                Shape::Array(_, n) | Shape::NamedArray(_, n, _) => values.len().min(*n),
                _ => values.len(),
            };
            while backing.len() < count {
                backing.push(Cell::zero(child));
            }
            for (i, id) in values.iter().take(count).enumerate() {
                update(child, &mut backing[i], tree, *id, owner, path)?;
            }
            if let Shape::Array(_, n) | Shape::NamedArray(_, n, _) = shape {
                for cell in &mut backing[count..] {
                    *cell = Cell::zero(child);
                }
                *len = *n;
            } else {
                *len = count;
                if count == 0 {
                    backing.clear();
                }
            }
        }
        _ => return Err(bad()),
    }
    Ok(())
}
pub(super) trait FactsJsonDecode: Sized {
    const SHAPE: Shape;
    fn from_cell(cell: &Cell) -> Self;
}

/// An actual Go pointer, distinct from Option<Vec/Map> used for nil collections.
pub(super) struct JsonPointer<T>(pub Option<T>);
impl<T: FactsJsonDecode> FactsJsonDecode for JsonPointer<T> {
    const SHAPE: Shape = Shape::Pointer(&T::SHAPE);
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Pointer(value) = cell else {
            unreachable!()
        };
        Self(value.as_deref().map(T::from_cell))
    }
}

/// Preserve native backing cells between successful repeated JSON sections.
/// As with full-file decoding, an error invalidates this state: discard it.
pub(super) struct FactsJsonState<T> {
    cell: Cell,
    error: Option<String>,
    marker: std::marker::PhantomData<T>,
}
impl<T: FactsJsonDecode> Default for FactsJsonState<T> {
    fn default() -> Self {
        Self {
            cell: Cell::zero(&T::SHAPE),
            error: None,
            marker: std::marker::PhantomData,
        }
    }
}
impl<T: FactsJsonDecode> FactsJsonState<T> {
    pub fn apply(&mut self, bytes: &[u8]) -> Result<(), String> {
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        let result = super::facts_json_parse::parse(bytes)
            .and_then(|tree| update(&T::SHAPE, &mut self.cell, &tree, tree.root, None, ""));
        if let Err(e) = &result {
            self.error = Some(e.clone());
        }
        result
    }
    pub fn value(&self) -> Result<T, String> {
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        Ok(T::from_cell(&self.cell))
    }
}
macro_rules! signed {($($t:ty=>$bits:literal,$name:literal);*$(;)?)=>{$(impl FactsJsonDecode for $t{const SHAPE:Shape=Shape::Signed($bits,$name);fn from_cell(cell:&Cell)->Self{let Cell::Signed(v)=cell else{unreachable!()};*v as Self}})*};}
signed!(i64=>64,"int";i32=>32,"int32";i8=>8,"int8");
macro_rules! unsigned {($($t:ty=>$bits:literal,$name:literal);*$(;)?)=>{$(impl FactsJsonDecode for $t{const SHAPE:Shape=Shape::Unsigned($bits,$name);fn from_cell(cell:&Cell)->Self{let Cell::Unsigned(v)=cell else{unreachable!()};*v as Self}})*};}
unsigned!(u64=>64,"uint64";u32=>32,"uint32";u8=>8,"uint8");
impl FactsJsonDecode for bool {
    const SHAPE: Shape = Shape::Bool;
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Bool(v) = cell else { unreachable!() };
        *v
    }
}
impl FactsJsonDecode for f32 {
    const SHAPE: Shape = Shape::Float32;
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Float(v) = cell else { unreachable!() };
        *v
    }
}
impl<T: FactsJsonDecode> FactsJsonDecode for Option<T> {
    const SHAPE: Shape = T::SHAPE;
    fn from_cell(cell: &Cell) -> Self {
        (!cell.nil()).then(|| T::from_cell(cell))
    }
}
impl<T: FactsJsonDecode> FactsJsonDecode for Vec<T> {
    const SHAPE: Shape = Shape::Slice(&T::SHAPE);
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Sequence { backing, len, .. } = cell else {
            unreachable!()
        };
        backing[..*len].iter().map(T::from_cell).collect()
    }
}
impl<T: FactsJsonDecode, const N: usize> FactsJsonDecode for [T; N] {
    const SHAPE: Shape = Shape::Array(&T::SHAPE, N);
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Sequence { backing, .. } = cell else {
            unreachable!()
        };
        std::array::from_fn(|i| T::from_cell(&backing[i]))
    }
}
impl FactsJsonDecode for BTreeMap<u32, i64> {
    const SHAPE: Shape = Shape::Map {
        key: MapKey::Uint32,
        value: &i64::SHAPE,
    };
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Map(Some(values)) = cell else {
            unreachable!()
        };
        values
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), i64::from_cell(v)))
            .collect()
    }
}
impl FactsJsonDecode for BTreeMap<Vec<u8>, i64> {
    const SHAPE: Shape = Shape::Map {
        key: MapKey::String,
        value: &i64::SHAPE,
    };
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Map(Some(values)) = cell else {
            unreachable!()
        };
        values
            .iter()
            .map(|(k, v)| (k.as_bytes().to_vec(), i64::from_cell(v)))
            .collect()
    }
}
pub(super) fn decode<T: FactsJsonDecode>(bytes: &[u8]) -> Result<T, String> {
    let tree = super::facts_json_parse::parse(bytes)?;
    let mut cell = Cell::zero(&T::SHAPE);
    update(&T::SHAPE, &mut cell, &tree, tree.root, None, "")?;
    Ok(T::from_cell(&cell))
}

impl<T: FactsJsonDecode> FactsJsonDecode for BTreeMap<i64, T> {
    const SHAPE: Shape = Shape::Map {
        key: MapKey::Int,
        value: &T::SHAPE,
    };
    fn from_cell(cell: &Cell) -> Self {
        let Cell::Map(Some(values)) = cell else {
            unreachable!()
        };
        values
            .iter()
            .map(|(k, v)| (k.parse().unwrap(), T::from_cell(v)))
            .collect()
    }
}
