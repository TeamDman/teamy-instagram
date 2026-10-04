use facet::Facet;

/// Fixed schema or content categories; archive names never become report fields.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchiveEntryCategory {
    SavedPosts,
    LikedPosts,
    LikedComments,
    MessageThread,
    StoriesViewed,
    AdWatchedVideos,
    UnsupportedJson,
    Media,
    Other,
    Directory,
    Unreadable,
}
