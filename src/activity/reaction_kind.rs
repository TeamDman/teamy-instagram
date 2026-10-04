use facet::Facet;

/// A message reaction category, distinct from liking a reel.
/// Other reaction tokens are never copied into reports.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReactionKind {
    Heart,
    Other,
}
