use facet::Facet;

/// Media metadata inside a message; validation never opens the referenced URI.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageMedia {
    pub uri: String,
    pub creation_timestamp: Option<u64>,
}
