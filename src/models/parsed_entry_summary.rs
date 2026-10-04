use facet::Facet;

/// Counts derived from a successful typed parse, safe for coverage reporting.
#[derive(Facet, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct ParsedEntrySummary {
    pub record_count: u64,
    pub message_count: u64,
    pub reaction_count: u64,
}
