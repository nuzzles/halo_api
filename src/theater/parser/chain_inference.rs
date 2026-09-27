//! Optional native chain inference. Confirmed alignment is evidence for skipping
//! a transient, not proof of its archetype when several types share that alignment.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChainInferenceOutcome {
    Immediate,
    Deep,
    NoConfirmation,
    BudgetExhausted,
    Ambiguous,
}
