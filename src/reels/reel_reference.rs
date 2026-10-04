use super::ReelId;
use facet::Facet;

/// A recognized reel URL normalized by its shortcode.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ReelReference {
    pub reel_id: ReelId,
    pub canonical_url: String,
}
