use super::EmptyExportObject;
use super::Message;
use super::MessageJoinableMode;
use super::MessageParticipant;
use facet::Facet;

/// Known message thread schema. Other export sections remain unsupported.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageThread {
    pub participants: Vec<MessageParticipant>,
    pub messages: Vec<Message>,
    pub title: String,
    pub is_still_participant: bool,
    pub thread_path: String,
    pub magic_words: Vec<EmptyExportObject>,
    pub joinable_mode: Option<MessageJoinableMode>,
    pub is_pending: Option<bool>,
}
