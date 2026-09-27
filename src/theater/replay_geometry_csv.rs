//! Byte-preserving CSV behavior used by the native external geometry loader.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryCsvErrorKind {
    Eof,
    BareQuote,
    Quote,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryCsvStage {
    Header,
    Record,
}
/// Native CSV coordinates are one-based byte columns and physical line numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryCsvError {
    pub kind: GeometryCsvErrorKind,
    pub stage: GeometryCsvStage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}
impl std::fmt::Display for GeometryCsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "geometry CSV {:?} {:?}", self.stage, self.kind)?;
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(f, " at line {line}, byte column {column}")?;
        }
        Ok(())
    }
}
impl std::error::Error for GeometryCsvError {}
pub(super) struct GeometryCsv {
    pub columns: BTreeMap<Vec<u8>, usize>,
    pub rows: Vec<Vec<Vec<u8>>>,
}
impl GeometryCsv {
    pub fn field<'a>(&self, row: &'a [Vec<u8>], name: &[u8]) -> &'a [u8] {
        self.columns
            .get(name)
            .and_then(|&i| row.get(i))
            .map_or(&[], Vec::as_slice)
    }
}
pub(super) fn read_geometry_csv(input: &[u8]) -> Result<GeometryCsv, GeometryCsvError> {
    // Go normalizes CRLF inside quoted fields too, and drops a final bare CR.
    let lines: Vec<Vec<u8>> = input
        .split_inclusive(|&c| c == b'\n')
        .map(|line| {
            let mut line = line.to_vec();
            if line.ends_with(b"\r\n") {
                line.remove(line.len() - 2);
            } else if line.ends_with(b"\r") {
                line.pop();
            }
            line
        })
        .collect();
    let mut records = Vec::new();
    let mut line_index = 0;
    while line_index < lines.len() {
        if lines[line_index].is_empty() || lines[line_index] == b"\n" {
            line_index += 1;
            continue;
        }
        let start = line_index + 1;
        let stage = if records.is_empty() {
            GeometryCsvStage::Header
        } else {
            GeometryCsvStage::Record
        };
        let fail = |kind, line, column| GeometryCsvError {
            kind,
            stage,
            start_line: Some(start),
            line: Some(line),
            column: Some(column),
        };
        let mut row = Vec::new();
        let mut pos = 0;
        'record: loop {
            let line = &lines[line_index];
            if line.get(pos) != Some(&b'"') {
                let comma = line[pos..].iter().position(|&c| c == b',').map(|i| pos + i);
                let end = comma.unwrap_or(line.len() - usize::from(line.ends_with(b"\n")));
                let field = &line[pos..end];
                if let Some(i) = field.iter().position(|&c| c == b'"') {
                    return Err(fail(
                        GeometryCsvErrorKind::BareQuote,
                        line_index + 1,
                        pos + i + 1,
                    ));
                }
                row.push(field.to_vec());
                if let Some(comma) = comma {
                    pos = comma + 1;
                    continue;
                }
                break 'record;
            }
            pos += 1;
            let mut field = Vec::new();
            loop {
                let line = &lines[line_index];
                if let Some(i) = line[pos..].iter().position(|&c| c == b'"') {
                    let quote = pos + i;
                    field.extend_from_slice(&line[pos..quote]);
                    pos = quote + 1;
                    match line.get(pos) {
                        Some(b'"') => {
                            field.push(b'"');
                            pos += 1;
                        }
                        Some(b',') => {
                            row.push(field);
                            pos += 1;
                            continue 'record;
                        }
                        None => {
                            row.push(field);
                            break 'record;
                        }
                        Some(b'\n') if pos + 1 == line.len() => {
                            row.push(field);
                            break 'record;
                        }
                        _ => {
                            return Err(fail(
                                GeometryCsvErrorKind::Quote,
                                line_index + 1,
                                quote + 1,
                            ));
                        }
                    }
                } else {
                    field.extend_from_slice(&line[pos..]);
                    pos = line.len();
                    if line_index + 1 == lines.len() || lines[line_index + 1].is_empty() {
                        return Err(fail(GeometryCsvErrorKind::Quote, line_index + 1, pos + 1));
                    }
                    line_index += 1;
                    pos = 0;
                }
            }
        }
        records.push(row);
        line_index += 1;
    }
    if records.is_empty() {
        return Err(GeometryCsvError {
            kind: GeometryCsvErrorKind::Eof,
            stage: GeometryCsvStage::Header,
            start_line: None,
            line: None,
            column: None,
        });
    }
    let mut records = records.into_iter();
    let columns = records
        .next()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect();
    Ok(GeometryCsv {
        columns,
        rows: records.collect(),
    })
}
