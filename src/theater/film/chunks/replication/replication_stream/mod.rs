//! Ordered replication stream and packet envelope models.

use super::ReplicationStreamChunk;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub chunks: Vec<ReplicationStreamChunk>,
}
