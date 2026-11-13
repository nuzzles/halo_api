//! Recorded event-table admission for the established v41/27 writer.
//! Native column: reference analysis 413971b vue_a_versions.go, DAT_14474cd90.
use crate::theater::parser::v41::chunks::registry::source::FilmRegistryRead;
use crate::theater::parser::v41::chunks::replication::source::EventListStop;

const NATIVE: &[u8; 123] = b"111111111111111111211111111111311114311121111121211111112111261111111211111111111515111112331211141111111113111111111111111";

/// Internal configured readers explicitly select Native. Public Film/replay
/// replace that selection with the result of checking the entire recorded table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum EventVersionContext {
    #[default]
    Native,
    Missing {
        event_code: u8,
    },
    Unsupported {
        event_code: u8,
        recorded: u32,
        supported: u32,
    },
}

impl EventVersionContext {
    pub(crate) fn native_version(code: usize) -> Option<u32> {
        NATIVE.get(code).map(|v| u32::from(v - b'0'))
    }

    pub(crate) fn recorded(registry: &FilmRegistryRead) -> Self {
        let header = registry
            .bootstrap_header
            .as_ref()
            .filter(|_| registry.header == Some([41, 27]));
        for event_code in 0..123 {
            let Some(recorded) = header.and_then(|h| h.event_version(event_code)) else {
                return Self::Missing { event_code };
            };
            let supported = Self::native_version(usize::from(event_code)).unwrap();
            if recorded != supported {
                return Self::Unsupported {
                    event_code,
                    recorded,
                    supported,
                };
            }
        }
        Self::Native
    }

    pub(crate) fn is_native(self) -> bool {
        self == Self::Native
    }

    pub(crate) fn stop(self) -> Option<EventListStop> {
        match self {
            Self::Native => None,
            Self::Missing { event_code } => Some(EventListStop::MissingEventVersion { event_code }),
            Self::Unsupported {
                event_code,
                recorded,
                supported,
            } => Some(EventListStop::UnsupportedEventVersion {
                event_code,
                recorded,
                supported,
            }),
        }
    }
}

#[cfg(test)]
mod tests;
