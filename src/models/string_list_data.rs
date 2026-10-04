use facet::Facet;

/// Link and timestamp tuple in the older comment-like export format.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct StringListData {
    pub href: String,
    pub value: String,
    pub timestamp: u64,
}
