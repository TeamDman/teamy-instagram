use super::EmptyExportObject;
use super::ExportLabelValue;
use facet::Facet;

/// Current saved, liked, and viewed-story activity record.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct ExportRecord {
    pub fbid: String,
    pub timestamp: u64,
    pub media: Vec<EmptyExportObject>,
    pub label_values: Vec<ExportLabelValue>,
}
