use super::SourceTimestampUnit;
use facet::Facet;

/// The original numeric timestamp, without guessing or rewriting its unit.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceTimestamp {
    pub value: u64,
    pub unit: SourceTimestampUnit,
}
