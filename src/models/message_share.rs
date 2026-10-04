use super::EmptyExportObject;
use facet::Facet;

/// Observed variants of a shared link or profile in a message.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageShare {
    pub link: Option<String>,
    pub share_text: Option<String>,
    pub original_content_owner: Option<String>,
    pub media: Option<Vec<EmptyExportObject>>,
    pub profile_share_name: Option<String>,
    pub profile_share_username: Option<String>,
}
