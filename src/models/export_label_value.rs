use super::EmptyExportObject;
use facet::Facet;

/// Recursive labels used by the current activity export format.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct ExportLabelValue {
    pub label: Option<String>,
    pub value: Option<String>,
    pub href: Option<String>,
    pub title: Option<String>,
    pub dict: Option<Vec<Self>>,
    /// Observed empty vector; its future element schema remains unsupported.
    pub vec: Option<Vec<EmptyExportObject>>,
}

impl ExportLabelValue {
    pub(crate) fn has_unsupported_vec(&self) -> bool {
        self.vec.as_ref().is_some_and(|values| !values.is_empty())
            || self
                .dict
                .as_ref()
                .is_some_and(|labels| labels.iter().any(Self::has_unsupported_vec))
    }
}
