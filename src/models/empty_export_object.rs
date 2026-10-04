use facet::Facet;

/// Placeholder for arrays observed only while empty.
///
/// The validation dispatcher rejects nonempty arrays of this type. Their item
/// schema is not claimed as supported until a real structural sample is known.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "Facet needs the JSON object shape for this observed-empty placeholder"
)]
pub struct EmptyExportObject {}
