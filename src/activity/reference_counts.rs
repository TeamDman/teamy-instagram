use facet::Facet;

/// Counts cover URL occurrences scanned in the selected fixed source fields.
/// Reactions derived from a link do not inflate the link occurrence count.
#[derive(Facet, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReferenceCounts {
    pub recognized_reel_occurrences: usize,
    pub unsupported_instagram_occurrences: usize,
    /// Opaque shares or malformed reel-like links cannot identify a reel.
    pub ambiguous_reel_occurrences: usize,
    pub unrelated_url_occurrences: usize,
}
