use facet::Facet;

/// A source-backed action or incidental reference. Unresolved variants make no claim
/// about who sent a shared reel or performed a message reaction.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActivityKind {
    Seen,
    Liked,
    Bookmarked,
    Sent,
    Received,
    ReactedTo,
    SharedUnresolved,
    ReactionUnresolved,
    /// A caption URL does not establish the action applied to its target reel.
    IncidentalReference,
}
