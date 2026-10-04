use super::ReelReference;
use super::UnsupportedInstagramReference;
use facet::Facet;

/// Whether a URL establishes a reel reference, is unsupported, or is unrelated.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ReferenceResolution {
    Reel(ReelReference),
    Unsupported(UnsupportedInstagramReference),
    Unrelated,
}
