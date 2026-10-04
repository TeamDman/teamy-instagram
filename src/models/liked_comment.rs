use super::StringListData;
use facet::Facet;

/// A comment-like record; this is distinct from liking a reel.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct LikedComment {
    pub title: String,
    pub string_list_data: Vec<StringListData>,
}
