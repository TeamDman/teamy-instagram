use arbitrary::Arbitrary;
use facet::Facet;

/// Selecting "latest" always requires one explicit, independent date source.
#[derive(Facet, Arbitrary, Debug, Clone, Copy, PartialEq, Eq)]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum DateBasis {
    ExportFilename,
    FileModified,
}
