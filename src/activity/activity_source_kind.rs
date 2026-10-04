use facet::Facet;

/// The export section or message field that supplies a reel reference.
/// Viewed stories and watched-ad records are distinct evidence; neither
/// establishes a complete watch history or video completion.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActivitySourceKind {
    SavedPost,
    LikedPost,
    ViewedStory,
    AdWatchedVideo,
    MessageSharedLink,
    MessageContentLink,
    /// A URL occurring within a record caption, separate from its action target.
    RecordCaptionLink,
}
