//! Complete native FilmStatborg JSON payload, including nil collections.
use super::facts_json_read::{Cell, FactsJsonDecode, FactsJsonState, Shape};
use super::facts_json_write::{FactsJsonValue, FactsJsonWriter};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsStatValue {
    #[serde(rename = "A")]
    pub a: i64,
    #[serde(rename = "B")]
    pub b: i64,
    #[serde(rename = "C")]
    pub c: i64,
    #[serde(rename = "D")]
    pub d: i64,
    #[serde(rename = "HasC")]
    pub has_c: bool,
    #[serde(rename = "HasD")]
    pub has_d: bool,
}
impl FactsJsonDecode for FactsStatValue {
    const SHAPE: Shape = Shape::Struct {
        name: "types.StatValue",
        fields: &[
            ("A", &Shape::Signed(64, "int64")),
            ("B", &Shape::Signed(64, "int64")),
            ("C", &Shape::Signed(64, "int64")),
            ("D", &Shape::Signed(64, "int64")),
            ("HasC", &<bool as FactsJsonDecode>::SHAPE),
            ("HasD", &<bool as FactsJsonDecode>::SHAPE),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            a: FactsJsonDecode::from_cell(cell.field("A")),
            b: FactsJsonDecode::from_cell(cell.field("B")),
            c: FactsJsonDecode::from_cell(cell.field("C")),
            d: FactsJsonDecode::from_cell(cell.field("D")),
            has_c: FactsJsonDecode::from_cell(cell.field("HasC")),
            has_d: FactsJsonDecode::from_cell(cell.field("HasD")),
        }
    }
}
impl FactsJsonValue for FactsStatValue {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        let mut first = true;
        w.field("A", &self.a, &mut first);
        w.field("B", &self.b, &mut first);
        w.field("C", &self.c, &mut first);
        w.field("D", &self.d, &mut first);
        w.field("HasC", &self.has_c, &mut first);
        w.field("HasD", &self.has_d, &mut first);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsStatRecord {
    #[serde(rename = "TimeMS")]
    pub time_ms: i64,
    #[serde(rename = "Slot")]
    pub slot: i64,
    #[serde(rename = "Round")]
    pub round: i64,
    #[serde(rename = "Comps")]
    pub comps: Option<BTreeMap<i64, FactsStatValue>>,
}
impl FactsJsonDecode for FactsStatRecord {
    const SHAPE: Shape = Shape::Struct {
        name: "types.StatRecord",
        fields: &[
            ("TimeMS", &<i64 as FactsJsonDecode>::SHAPE),
            ("Slot", &<i64 as FactsJsonDecode>::SHAPE),
            ("Round", &<i64 as FactsJsonDecode>::SHAPE),
            (
                "Comps",
                &<Option<BTreeMap<i64, FactsStatValue>> as FactsJsonDecode>::SHAPE,
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            time_ms: FactsJsonDecode::from_cell(cell.field("TimeMS")),
            slot: FactsJsonDecode::from_cell(cell.field("Slot")),
            round: FactsJsonDecode::from_cell(cell.field("Round")),
            comps: FactsJsonDecode::from_cell(cell.field("Comps")),
        }
    }
}
impl FactsJsonValue for FactsStatRecord {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        let mut first = true;
        w.field("TimeMS", &self.time_ms, &mut first);
        w.field("Slot", &self.slot, &mut first);
        w.field("Round", &self.round, &mut first);
        w.field("Comps", &self.comps, &mut first);
        w.raw(b"}");
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsFileStatborg {
    #[serde(rename = "Records")]
    pub records: Option<Vec<FactsStatRecord>>,
    #[serde(rename = "BurstMS")]
    pub burst_ms: Option<Vec<i64>>,
    #[serde(rename = "Truncated")]
    pub truncated: bool,
    #[serde(rename = "ChunkStartMS")]
    pub chunk_start_ms: Option<BTreeMap<i64, i64>>,
}
impl FactsJsonDecode for FactsFileStatborg {
    const SHAPE: Shape = Shape::Struct {
        name: "replay.FilmStatborg",
        fields: &[
            (
                "Records",
                &<Option<Vec<FactsStatRecord>> as FactsJsonDecode>::SHAPE,
            ),
            ("BurstMS", &<Option<Vec<i64>> as FactsJsonDecode>::SHAPE),
            ("Truncated", &<bool as FactsJsonDecode>::SHAPE),
            (
                "ChunkStartMS",
                &<Option<BTreeMap<i64, i64>> as FactsJsonDecode>::SHAPE,
            ),
        ],
    };
    fn from_cell(cell: &Cell) -> Self {
        Self {
            records: FactsJsonDecode::from_cell(cell.field("Records")),
            burst_ms: FactsJsonDecode::from_cell(cell.field("BurstMS")),
            truncated: FactsJsonDecode::from_cell(cell.field("Truncated")),
            chunk_start_ms: FactsJsonDecode::from_cell(cell.field("ChunkStartMS")),
        }
    }
}
impl FactsJsonValue for FactsFileStatborg {
    fn write_json(&self, w: &mut FactsJsonWriter) {
        w.raw(b"{");
        let mut first = true;
        w.field("Records", &self.records, &mut first);
        w.field("BurstMS", &self.burst_ms, &mut first);
        w.field("Truncated", &self.truncated, &mut first);
        w.field("ChunkStartMS", &self.chunk_start_ms, &mut first);
        w.raw(b"}");
    }
}
pub fn encode_facts_statborg_json(value: &FactsFileStatborg) -> Vec<u8> {
    let mut w = FactsJsonWriter::default();
    value.write_json(&mut w);
    w.bytes
}
pub fn decode_facts_statborg_json(bytes: &[u8]) -> Result<FactsFileStatborg, String> {
    FactsStatborgJsonReader::default().read(bytes)
}
/// Retains successful native section updates. Discard after an error.
#[derive(Default)]
pub struct FactsStatborgJsonReader {
    state: FactsJsonState<FactsFileStatborg>,
}
impl FactsStatborgJsonReader {
    pub fn read(&mut self, bytes: &[u8]) -> Result<FactsFileStatborg, String> {
        self.state.apply(bytes)?;
        self.state.value()
    }
}
