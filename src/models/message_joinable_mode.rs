use facet::Facet;

/// Optional joinability metadata on a message thread.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageJoinableMode {
    pub mode: u64,
    pub link: String,
}
