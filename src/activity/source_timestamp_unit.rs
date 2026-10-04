use facet::Facet;

/// Timestamp units come from a verified field schema. `Unspecified` preserves
/// a reaction timestamp whose source field has no verified unit convention.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SourceTimestampUnit {
    Seconds,
    Milliseconds,
    Unspecified,
}
