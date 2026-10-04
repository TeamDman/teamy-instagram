use std::fmt;

/// A fixed, privacy-safe shortcode validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidReelId;

impl fmt::Display for InvalidReelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid reel shortcode")
    }
}

impl std::error::Error for InvalidReelId {}
