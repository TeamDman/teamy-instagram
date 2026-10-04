use facet::Facet;

/// A fixed reason an Instagram URL cannot establish a reel identity.
///
/// No original URL, token, or rejected value is retained in this type.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UnsupportedInstagramReference {
    /// An opaque sharing token needs external resolution, which is not attempted.
    ShareReel,
    /// A post, story, or other explicitly non-reel path.
    NonReel,
    /// A reel path has a missing or syntactically invalid shortcode.
    InvalidReel,
    /// Another Instagram path whose content type is not established.
    UnsupportedPath,
    /// Invalid syntax, credentials, a nonstandard port, or an unsupported scheme.
    InvalidUrl,
}
