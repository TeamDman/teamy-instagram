use super::InvalidReelId;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// A syntactically validated Instagram reel shortcode.
///
/// This identifies a referenced reel; it does not establish that the reel exists
/// or that the exporter viewed it. Case is significant.
#[derive(Facet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[facet(json::proxy = String)]
pub struct ReelId(String);

impl ReelId {
    /// Validate a shortcode without accepting URL escapes or path separators.
    ///
    /// # Errors
    ///
    /// Returns a fixed error if the value is empty or contains anything except
    /// ASCII letters, ASCII digits, an underscore, or a hyphen, or exceeds the
    /// conservative 128-byte supported-shortcode limit.
    pub fn parse(value: &str) -> Result<Self, InvalidReelId> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return Err(InvalidReelId);
        }
        Ok(Self(value.to_owned()))
    }

    /// Return the validated, case-sensitive shortcode.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return a canonical URL without tracking parameters or fragments.
    #[must_use]
    pub fn canonical_url(&self) -> String {
        format!("https://www.instagram.com/reel/{}/", self.0)
    }
}

impl TryFrom<String> for ReelId {
    type Error = InvalidReelId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<&ReelId> for String {
    fn from(value: &ReelId) -> Self {
        value.0.clone()
    }
}

impl FromStr for ReelId {
    type Err = InvalidReelId;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for ReelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
