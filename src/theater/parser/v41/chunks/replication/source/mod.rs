//! Structural replication reads retained only while parsing.
use crate::theater::parser::packet::*;
use crate::theater::parser::v41::chunks::replication::source::components::field::*;
use serde::{Deserialize, Serialize};

pub(crate) mod components;
pub(crate) mod replication_stream;
#[cfg(test)]
pub(crate) use components::RawBits;
pub(crate) use replication_stream::*;
