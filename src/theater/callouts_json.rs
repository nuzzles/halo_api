//! Schema-directed Go JSON updates for external catalogs. Slice backing storage
//! survives repeated fields until decoding ends, as in encoding/json.Unmarshal.
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

pub(super) enum Shape {
    Int,
    Float,
    NullableFloat,
    Timestamp,
    Text,
    Bool,
    Struct(&'static [(&'static str, &'static Shape)]),
    Map(&'static Shape),
    Slice(&'static Shape),
    Pair,
}
static INT: Shape = Shape::Int;
static FLOAT: Shape = Shape::Float;
static TEXT: Shape = Shape::Text;
static BOOL: Shape = Shape::Bool;
static PAIR: Shape = Shape::Pair;
static POLYGON: Shape = Shape::Slice(&PAIR);
static PARTS: Shape = Shape::Slice(&POLYGON);
static ZONE: Shape = Shape::Struct(&[
    ("volume_index", &INT),
    ("name", &TEXT),
    ("en", &TEXT),
    ("fr", &TEXT),
    ("x", &FLOAT),
    ("y", &FLOAT),
    ("z", &FLOAT),
    ("z_bottom", &FLOAT),
    ("z_top", &FLOAT),
    ("big", &BOOL),
    ("polygon", &POLYGON),
    ("parts", &PARTS),
    ("holes", &PARTS),
]);
static ZONES: Shape = Shape::Slice(&ZONE);
static ENTRY: Shape = Shape::Struct(&[("module", &TEXT), ("provenance", &TEXT), ("zones", &ZONES)]);
static MAPS: Shape = Shape::Map(&ENTRY);
static RAW: Shape = Shape::Struct(&[("volume_index", &INT), ("polygon", &POLYGON)]);
static RAW_ZONES: Shape = Shape::Slice(&RAW);
static RAW_MAPS: Shape = Shape::Map(&RAW_ZONES);
static CATALOG: Shape = Shape::Struct(&[
    ("schema_version", &INT),
    ("title_slug", &TEXT),
    ("source", &TEXT),
    ("maps", &MAPS),
    ("brut", &RAW_MAPS),
    ("maps_by_id", &MAPS),
]);

enum Cell {
    Scalar(Value),
    Object(BTreeMap<String, Cell>),
    Map(Option<BTreeMap<String, Cell>>),
    Sequence {
        present: bool,
        backing: Vec<Cell>,
        len: usize,
    },
}
impl Cell {
    fn zero(shape: &Shape) -> Self {
        match shape {
            Shape::Int => Self::Scalar(0.into()),
            Shape::Float => Self::Scalar(0.0.into()),
            Shape::NullableFloat => Self::Scalar(Value::Null),
            Shape::Timestamp => Self::Scalar(super::MapBackgroundTime::default().storage_value()),
            Shape::Text => Self::Scalar("".into()),
            Shape::Bool => Self::Scalar(false.into()),
            Shape::Struct(fields) => Self::Object(
                fields
                    .iter()
                    .map(|(key, shape)| (key.to_string(), Self::zero(shape)))
                    .collect(),
            ),
            Shape::Map(_) => Self::Map(None),
            Shape::Slice(_) => Self::Sequence {
                present: false,
                backing: vec![],
                len: 0,
            },
            Shape::Pair => Self::Sequence {
                present: true,
                backing: vec![Self::zero(&FLOAT), Self::zero(&FLOAT)],
                len: 2,
            },
        }
    }
    fn finish(self) -> Value {
        match self {
            Self::Scalar(value) => value,
            Self::Object(values) => {
                Value::Object(values.into_iter().map(|(k, v)| (k, v.finish())).collect())
            }
            Self::Map(None) => Value::Null,
            Self::Map(Some(values)) => {
                Value::Object(values.into_iter().map(|(k, v)| (k, v.finish())).collect())
            }
            Self::Sequence { present: false, .. } => Value::Null,
            Self::Sequence { backing, len, .. } => {
                Value::Array(backing.into_iter().take(len).map(Cell::finish).collect())
            }
        }
    }
}
struct Seed<'a> {
    shape: &'static Shape,
    cell: &'a mut Cell,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        let scalar = match self.shape {
            Shape::Int => Option::<i64>::deserialize(d)?.map(Value::from),
            Shape::Float => Option::<f64>::deserialize(d)?.map(Value::from),
            Shape::NullableFloat => {
                Some(Option::<f64>::deserialize(d)?.map_or(Value::Null, Value::from))
            }
            Shape::Timestamp => {
                // time.Time.UnmarshalJSON parses the raw quoted bytes, without
                // unescaping. Borrowed str rejects escaped JSON date strings.
                Option::<&str>::deserialize(d)?
                    .map(|s| super::MapBackgroundTime::parse(s).map(|t| t.storage_value()))
                    .transpose()
                    .map_err(D::Error::custom)?
            }
            Shape::Text => Option::<String>::deserialize(d)?.map(Value::from),
            Shape::Bool => Option::<bool>::deserialize(d)?.map(Value::from),
            _ => return d.deserialize_any(self),
        };
        if let Some(value) = scalar {
            *self.cell = Cell::Scalar(value);
        }
        Ok(())
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the native callout field type or null")
    }
    fn visit_unit<E: Error>(self) -> Result<(), E> {
        if matches!(self.shape, Shape::Map(_) | Shape::Slice(_)) {
            *self.cell = Cell::zero(self.shape);
        }
        Ok(())
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        match (self.shape, self.cell) {
            (Shape::Struct(fields), Cell::Object(cells)) => {
                while let Some(key) = map.next_key::<String>()? {
                    let folded = super::map_catalog::field_name(&key);
                    if let Some((name, shape)) = fields
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(&folded))
                    {
                        map.next_value_seed(Seed {
                            shape,
                            cell: cells.get_mut(*name).unwrap(),
                        })?;
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            (Shape::Map(shape), Cell::Map(values)) => {
                let values = values.get_or_insert_with(BTreeMap::new);
                while let Some(key) = map.next_key::<String>()? {
                    let mut value = Cell::zero(shape);
                    map.next_value_seed(Seed {
                        shape,
                        cell: &mut value,
                    })?;
                    values.insert(key, value);
                }
            }
            _ => {
                return Err(M::Error::custom(
                    "object supplied for non-object callout field",
                ));
            }
        }
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let child = match self.shape {
            Shape::Slice(child) => child,
            Shape::Pair => &FLOAT,
            _ => {
                return Err(A::Error::custom(
                    "array supplied for non-array callout field",
                ));
            }
        };
        let Cell::Sequence {
            present,
            backing,
            len,
        } = self.cell
        else {
            unreachable!()
        };
        *present = true;
        let pair = matches!(self.shape, Shape::Pair);
        let mut i = 0;
        loop {
            if pair && i == 2 {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                *len = 2;
                return Ok(());
            }
            let added = i == backing.len();
            if added {
                backing.push(Cell::zero(child));
            }
            if seq
                .next_element_seed(Seed {
                    shape: child,
                    cell: &mut backing[i],
                })?
                .is_none()
            {
                if added {
                    backing.pop();
                }
                if pair {
                    for value in &mut backing[i..] {
                        *value = Cell::zero(child);
                    }
                    *len = 2;
                } else {
                    *len = i;
                    if i == 0 {
                        backing.clear();
                    }
                }
                return Ok(());
            }
            i += 1;
        }
    }
}
pub(super) fn decode(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    decode_with(bytes, &CATALOG)
}
pub(super) fn decode_with(bytes: &[u8], shape: &'static Shape) -> Result<Value, serde_json::Error> {
    let bytes = super::catalog_integer_zero::normalize(
        super::map_catalog::native_catalog_json(bytes),
        shape,
    );
    let mut decoder = serde_json::Deserializer::from_slice(&bytes);
    let mut catalog = Cell::zero(shape);
    Seed {
        shape,
        cell: &mut catalog,
    }
    .deserialize(&mut decoder)?;
    decoder.end()?;
    Ok(catalog.finish())
}
