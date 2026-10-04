use teamy_instagram::identity::ExporterIdentity;
use teamy_instagram::identity::IdentityError;
use teamy_instagram::identity::IdentityMatch;
use teamy_instagram::models::MessageParticipant;

fn participants(labels: &[&str]) -> Vec<MessageParticipant> {
    labels
        .iter()
        .map(|name| MessageParticipant {
            name: (*name).to_owned(),
        })
        .collect()
}

#[test]
fn only_explicit_exact_labels_establish_self() -> Result<(), Box<dyn std::error::Error>> {
    let thread = participants(&["Synthetic Self", "Synthetic Friend"]);
    assert_eq!(
        ExporterIdentity::default().resolve_label("Synthetic Self", &thread),
        IdentityMatch::Unresolved
    );
    let identity = ExporterIdentity::new(vec!["Synthetic Self".to_owned()])?;
    assert_eq!(
        identity.resolve_label("Synthetic Self", &thread),
        IdentityMatch::SelfActor
    );
    assert_eq!(
        identity.resolve_label("Synthetic Friend", &thread),
        IdentityMatch::OtherActor
    );
    let wrong = ExporterIdentity::new(vec!["synthetic self".to_owned()])?;
    assert_eq!(
        wrong.resolve_label("Synthetic Friend", &thread),
        IdentityMatch::Unresolved
    );
    assert!(!format!("{identity:?}").contains("Synthetic Self"));
    Ok(())
}

#[test]
fn duplicate_display_labels_remain_ambiguous() -> Result<(), Box<dyn std::error::Error>> {
    let identity = ExporterIdentity::new(vec!["Synthetic Self".to_owned()])?;
    assert_eq!(
        identity.resolve_label(
            "Synthetic Self",
            &participants(&["Synthetic Self", "Synthetic Self"])
        ),
        IdentityMatch::Ambiguous
    );
    assert_eq!(
        identity.resolve_label("Synthetic Friend", &participants(&["Synthetic Friend"])),
        IdentityMatch::Unresolved
    );
    Ok(())
}

#[test]
fn invalid_mapping_errors_contain_no_labels() {
    assert_eq!(
        ExporterIdentity::new(vec![String::new()]).err(),
        Some(IdentityError::EmptyLabel)
    );
    assert_eq!(
        ExporterIdentity::new(vec!["Synthetic Self".to_owned(); 2]).err(),
        Some(IdentityError::DuplicateLabel)
    );
}

#[test]
fn multiple_mapped_participants_do_not_establish_one_actor()
-> Result<(), Box<dyn std::error::Error>> {
    let identity = ExporterIdentity::new(vec![
        "Synthetic Self".to_owned(),
        "Synthetic Alias".to_owned(),
    ])?;
    assert_eq!(
        identity.resolve_label(
            "Synthetic Self",
            &participants(&["Synthetic Self", "Synthetic Alias"])
        ),
        IdentityMatch::Ambiguous
    );
    Ok(())
}
#[test]
fn unknown_actor_label_is_not_inferred_to_be_someone_else() -> Result<(), Box<dyn std::error::Error>>
{
    let identity = ExporterIdentity::new(vec!["Synthetic Self".to_owned()])?;
    assert_eq!(
        identity.resolve_label(
            "Synthetic Unknown",
            &participants(&["Synthetic Self", "Synthetic Friend"])
        ),
        IdentityMatch::Unresolved
    );
    Ok(())
}
