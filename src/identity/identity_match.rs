/// Relationship established by an explicit exporter label mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityMatch {
    SelfActor,
    OtherActor,
    Unresolved,
    Ambiguous,
}
