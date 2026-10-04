use facet::Facet;

/// Canonical directories selected by the user. These values are private output.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct RootsConfiguration {
    pub version: u64,
    pub roots: Vec<String>,
}

impl Default for RootsConfiguration {
    fn default() -> Self {
        Self {
            version: 1,
            roots: Vec::new(),
        }
    }
}
