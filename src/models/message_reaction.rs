use facet::Facet;

/// A reaction to a message, separate from activity likes.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageReaction {
    pub reaction: String,
    pub actor: String,
    pub timestamp: Option<u64>,
}
