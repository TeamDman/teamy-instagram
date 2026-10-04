/// Sanitized failures when constructing an explicit identity mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    EmptyLabel,
    DuplicateLabel,
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyLabel => "identity_empty_label",
            Self::DuplicateLabel => "identity_duplicate_label",
        })
    }
}

impl std::error::Error for IdentityError {}
