use super::LikedComment;
use facet::Facet;

/// Wrapper used by the comment-like export section.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct LikedComments {
    pub likes_comment_likes: Vec<LikedComment>,
}
