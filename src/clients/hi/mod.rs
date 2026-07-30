mod client;
pub(crate) mod endpoints;
mod error;
pub mod film;
pub mod models;
mod pager;
mod player;
mod rate_limit;

pub use client::HaloInfiniteClient;
pub use error::InfiniteClientError;
pub use pager::{MAX_PAGE_SIZE, MatchHistoryPager};
pub use player::Player;
