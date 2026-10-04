use crate::identity::IdentityError;
use crate::identity::IdentityMatch;
use crate::models::MessageParticipant;

/// Explicit exporter display labels, kept out of serializable reports.
///
/// Labels are exact, case-sensitive assertions made by the caller. No account
/// username, profile display name, or message frequency is used to guess them.
#[derive(Clone, Default)]
pub struct ExporterIdentity {
    self_labels: Vec<String>,
}

impl std::fmt::Debug for ExporterIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExporterIdentity")
            .field("label_count", &self.self_labels.len())
            .finish()
    }
}

impl ExporterIdentity {
    /// Construct an exact mapping. An empty list leaves identity unresolved.
    ///
    /// # Errors
    /// Returns a fixed code for an empty or duplicate label, without its value.
    pub fn new(self_labels: Vec<String>) -> Result<Self, IdentityError> {
        for (index, label) in self_labels.iter().enumerate() {
            if label.trim().is_empty() {
                return Err(IdentityError::EmptyLabel);
            }
            if self_labels[..index].contains(label) {
                return Err(IdentityError::DuplicateLabel);
            }
        }
        Ok(Self { self_labels })
    }

    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.self_labels.is_empty()
    }

    /// Resolve a label only when the thread contains an unambiguous self label.
    ///
    /// Duplicate participant labels cannot establish which participant is self.
    /// Missing mapped labels leave that thread unresolved, including reactions.
    #[must_use]
    pub fn resolve_label(&self, label: &str, participants: &[MessageParticipant]) -> IdentityMatch {
        if !self.is_configured() {
            return IdentityMatch::Unresolved;
        }
        let occurrences = self
            .self_labels
            .iter()
            .map(|mapped| {
                participants
                    .iter()
                    .filter(|participant| participant.name == *mapped)
                    .count()
            })
            .collect::<Vec<_>>();
        if occurrences.iter().sum::<usize>() > 1 {
            return IdentityMatch::Ambiguous;
        }
        if !occurrences.contains(&1) {
            return IdentityMatch::Unresolved;
        }
        if let Some(index) = self.self_labels.iter().position(|mapped| mapped == label) {
            if occurrences[index] == 1 {
                IdentityMatch::SelfActor
            } else {
                IdentityMatch::Unresolved
            }
        } else if participants
            .iter()
            .any(|participant| participant.name == label)
        {
            IdentityMatch::OtherActor
        } else {
            IdentityMatch::Unresolved
        }
    }
}
