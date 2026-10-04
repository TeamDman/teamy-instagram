//! Reel identity and URL occurrence parsing.
//!
//! These functions normalize references, not activity or viewing claims. They
//! never make network requests or log rejected URLs. Callers retain each source
//! occurrence as evidence even when several occurrences have the same identity.

mod invalid_reel_id;
mod located_reference;
mod reel_id;
mod reel_reference;
mod reference_resolution;
mod unsupported_instagram_reference;

pub use invalid_reel_id::InvalidReelId;
pub use located_reference::LocatedReference;
pub use reel_id::ReelId;
pub use reel_reference::ReelReference;
pub use reference_resolution::ReferenceResolution;
pub use unsupported_instagram_reference::UnsupportedInstagramReference;
use url::Url;

/// Resolve a URL to a stable reel shortcode when its path establishes one.
///
/// Only HTTP(S) and the exact Instagram hosts instagram.com, www.instagram.com,
/// and m.instagram.com are accepted. Query strings and fragments do not affect
/// identity. Opaque sharing URLs and /p/ posts never become reel identities.
#[must_use]
pub fn parse_reference(value: &str) -> ReferenceResolution {
    let Some(authority) = authority(value) else {
        return ReferenceResolution::Unrelated;
    };
    let raw_host = authority.split(':').next().unwrap_or_default();
    if !is_instagram_host(raw_host) {
        return ReferenceResolution::Unrelated;
    }
    let Ok(parsed) = Url::parse(value) else {
        return unsupported(UnsupportedInstagramReference::InvalidUrl);
    };
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed
            .host_str()
            .is_none_or(|host| !is_instagram_host(host))
        || parsed.port().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b'\\')
    {
        return unsupported(UnsupportedInstagramReference::InvalidUrl);
    }
    let path = parsed.path().strip_suffix('/').unwrap_or(parsed.path());
    let mut components = path.split('/').skip(1);
    match components.next() {
        Some("reel" | "reels") => {
            let Some(shortcode) = components.next() else {
                return unsupported(UnsupportedInstagramReference::InvalidReel);
            };
            if components.next().is_some() {
                return unsupported(UnsupportedInstagramReference::InvalidReel);
            }
            let Ok(reel_id) = ReelId::parse(shortcode) else {
                return unsupported(UnsupportedInstagramReference::InvalidReel);
            };
            let canonical_url = reel_id.canonical_url();
            ReferenceResolution::Reel(ReelReference {
                reel_id,
                canonical_url,
            })
        }
        Some("share") if components.next() == Some("reel") => {
            unsupported(UnsupportedInstagramReference::ShareReel)
        }
        Some("p" | "tv" | "stories") => unsupported(UnsupportedInstagramReference::NonReel),
        _ => unsupported(UnsupportedInstagramReference::UnsupportedPath),
    }
}

/// Find HTTP(S) URL occurrences in plain text without discarding repeats.
///
/// Whitespace, quotes, angle brackets, and paired prose delimiters end a URL.
/// Trailing prose punctuation is removed. Byte offsets identify the resulting
/// URL substring in the original text; no copies of unrelated URLs are retained.
/// Bare domains are not treated as URLs.
#[must_use]
pub fn find_references(text: &str) -> Vec<LocatedReference> {
    let mut references = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        let suffix = &text[offset..];
        let Some(relative_start) = find_scheme(suffix) else {
            break;
        };
        let start = offset + relative_start;
        let remaining = &text[start..];
        let candidate_length = remaining
            .char_indices()
            .find_map(|(index, character)| {
                (is_url_delimiter(character)
                    || (matches!(character, ',' | ';') && has_http_scheme(&remaining[index + 1..])))
                .then_some(index)
            })
            .unwrap_or(remaining.len());
        let candidate_end = start + candidate_length;
        let candidate = text[start..candidate_end].trim_end_matches(['.', ',', ';', ':', '!', '?']);
        let end = start + candidate.len();
        references.push(LocatedReference {
            start,
            end,
            ordinal: references.len(),
            resolution: parse_reference(candidate),
        });
        offset = candidate_end;
    }
    references
}

fn authority(value: &str) -> Option<&str> {
    let (_, after_scheme) = value.split_once("://")?;
    Some(
        after_scheme
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default(),
    )
}

fn is_instagram_host(value: &str) -> bool {
    ["instagram.com", "www.instagram.com", "m.instagram.com"]
        .iter()
        .any(|host| value.eq_ignore_ascii_case(host))
}

fn unsupported(kind: UnsupportedInstagramReference) -> ReferenceResolution {
    ReferenceResolution::Unsupported(kind)
}

fn find_scheme(value: &str) -> Option<usize> {
    value.char_indices().find_map(|(index, _)| {
        let preceding = value[..index].chars().next_back();
        (has_http_scheme(&value[index..])
            && preceding.is_none_or(|character| !character.is_alphanumeric() && character != '_'))
        .then_some(index)
    })
}

fn has_http_scheme(value: &str) -> bool {
    value
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || value
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
}

fn is_url_delimiter(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '"' | '\''
                | '<'
                | '>'
                | '['
                | ']'
                | '{'
                | '}'
                | '('
                | ')'
                | '\u{00ab}'
                | '\u{00bb}'
                | '\u{2018}'
                | '\u{2019}'
                | '\u{201c}'
                | '\u{201d}'
        )
}
