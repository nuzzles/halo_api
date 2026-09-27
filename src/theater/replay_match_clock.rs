//! Shared conversion between match-event time and the published film frame grid.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayMatchClock {
    pub origin_us: u64,
    pub step_us: u64,
    pub frames: i64,
    pub death_offset_ms: i64,
}
impl ReplayMatchClock {
    pub fn frame_of_match_ms(self, time_ms: i64) -> i64 {
        if self.step_us == 0 {
            return -1;
        }
        let film_us = time_ms
            .wrapping_add(self.death_offset_ms)
            .wrapping_mul(1000);
        if film_us < (self.origin_us as i64) {
            return -1;
        }
        ((film_us as u64).wrapping_sub(self.origin_us) / self.step_us) as i64
    }
    pub fn match_ms_of_frame(self, frame: i64) -> i64 {
        if frame < 0 {
            return 0;
        }
        ((self
            .origin_us
            .wrapping_add((frame as u64).wrapping_mul(self.step_us)) as i64)
            / 1000)
            .wrapping_sub(self.death_offset_ms)
    }
    pub fn slack_frames(self, ms: i64) -> i64 {
        if self.step_us == 0 {
            return 0;
        }
        ((ms as u64)
            .wrapping_mul(1000)
            .wrapping_add(self.step_us)
            .wrapping_sub(1)
            / self.step_us) as i64
    }
}
