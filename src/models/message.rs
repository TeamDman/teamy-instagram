use super::MessageMedia;
use super::MessageReaction;
use super::MessageShare;
use facet::Facet;

/// Typed message data, retaining source fields without guessing its direction.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct Message {
    pub sender_name: String,
    pub timestamp_ms: u64,
    pub is_geoblocked_for_viewer: bool,
    pub content: Option<String>,
    pub is_unsent_image_by_messenger_kid_parent: Option<bool>,
    pub share: Option<MessageShare>,
    pub reactions: Option<Vec<MessageReaction>>,
    pub photos: Option<Vec<MessageMedia>>,
    pub videos: Option<Vec<MessageMedia>>,
    pub call_duration: Option<u64>,
}
