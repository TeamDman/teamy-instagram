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
}
