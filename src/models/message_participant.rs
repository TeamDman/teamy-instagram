use facet::Facet;

/// Source-defined participant identity, without exporter-direction inference.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(deny_unknown_fields)]
pub struct MessageParticipant {
    pub name: String,
}
