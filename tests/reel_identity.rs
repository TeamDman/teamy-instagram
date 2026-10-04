use teamy_instagram::reels::ReelId;
use teamy_instagram::reels::ReferenceResolution;
use teamy_instagram::reels::UnsupportedInstagramReference;
use teamy_instagram::reels::find_references;
use teamy_instagram::reels::parse_reference;

// All shortcodes, links, and surrounding text are synthetic.

fn resolved_id(value: &str) -> ReelId {
    match parse_reference(value) {
        ReferenceResolution::Reel(reference) => {
            assert_eq!(reference.canonical_url, reference.reel_id.canonical_url());
            reference.reel_id
        }
        other => panic!("expected a synthetic reel reference, got {other:?}"),
    }
}

#[test]
fn exact_hosts_schemes_and_plural_path_normalize() {
    for value in [
        "https://www.instagram.com/reel/Synthetic_A-1/",
        "http://instagram.com/reel/Synthetic_A-1",
        "https://m.instagram.com/reels/Synthetic_A-1/",
        "HTTPS://WWW.INSTAGRAM.COM/reel/Synthetic_A-1/",
        "https://instagram.com:443/reel/Synthetic_A-1/",
        "http://instagram.com:80/reel/Synthetic_A-1/",
    ] {
        let id = resolved_id(value);
        assert_eq!(id.as_str(), "Synthetic_A-1");
        assert_eq!(
            id.canonical_url(),
            "https://www.instagram.com/reel/Synthetic_A-1/"
        );
    }
}

#[test]
fn tracking_and_fragments_do_not_change_identity() {
    let first = resolved_id("https://instagram.com/reel/SyntheticOne/?igsh=synthetic#first");
    let second = resolved_id("https://m.instagram.com/reels/SyntheticOne?other=synthetic#second");
    assert_eq!(first, second);
}

#[test]
fn shortcode_case_is_significant() {
    assert_ne!(
        resolved_id("https://instagram.com/reel/SyntheticOne/"),
        resolved_id("https://instagram.com/reel/syntheticOne/")
    );
}

#[test]
fn ordinary_posts_stories_and_video_paths_are_not_inferred_to_be_reels() {
    for value in [
        "https://instagram.com/p/SyntheticOne/",
        "https://instagram.com/tv/SyntheticOne/",
        "https://instagram.com/stories/synthetic_profile/123/",
    ] {
        assert_eq!(
            parse_reference(value),
            ReferenceResolution::Unsupported(UnsupportedInstagramReference::NonReel)
        );
    }
}

#[test]
fn opaque_reel_sharing_tokens_remain_unresolved() {
    assert_eq!(
        parse_reference("https://www.instagram.com/share/reel/SyntheticToken/?igsh=synthetic"),
        ReferenceResolution::Unsupported(UnsupportedInstagramReference::ShareReel)
    );
}

#[test]
fn profile_and_unknown_paths_remain_unsupported() {
    for value in [
        "https://instagram.com/synthetic_profile/",
        "https://instagram.com/",
        "https://instagram.com/explore/",
    ] {
        assert_eq!(
            parse_reference(value),
            ReferenceResolution::Unsupported(UnsupportedInstagramReference::UnsupportedPath)
        );
    }
}

#[test]
fn deceptive_hosts_and_other_sites_are_unrelated() {
    for value in [
        "https://instagram.com.example.invalid/reel/SyntheticOne/",
        "https://evilinstagram.com/reel/SyntheticOne/",
        "https://subdomain.instagram.com/reel/SyntheticOne/",
        "https://instagram.com@example.invalid/reel/SyntheticOne/",
        "https://example.invalid/reel/SyntheticOne/",
        "https://instagram.com./reel/SyntheticOne/",
        "https://ｉｎｓｔａｇｒａｍ.com/reel/SyntheticOne/",
    ] {
        assert_eq!(parse_reference(value), ReferenceResolution::Unrelated);
    }
}

#[test]
fn malformed_reel_shortcodes_and_paths_are_not_identities() {
    for value in [
        "https://instagram.com/reel/",
        "https://instagram.com/reel",
        "https://instagram.com/reel/Synthetic%20One/",
        "https://instagram.com/reel/Synthetic%2FOne/",
        "https://instagram.com/reel/Synthetic%zz/",
        "https://instagram.com/reel/Synthetic.One/",
        "https://instagram.com/reel/SyntheticOne/extra/",
        "https://instagram.com/reel/SyntheticOne//",
        "https://instagram.com/reel/例/",
    ] {
        assert_eq!(
            parse_reference(value),
            ReferenceResolution::Unsupported(UnsupportedInstagramReference::InvalidReel)
        );
    }
}

#[test]
fn invalid_ports_schemes_and_backslashes_are_rejected() {
    for value in [
        "ftp://instagram.com/reel/SyntheticOne/",
        "https://instagram.com:444/reel/SyntheticOne/",
        "https://instagram.com:notaport/reel/SyntheticOne/",
        "https://instagram.com:secret@instagram.com/reel/SyntheticOne/",
        "https://instagram.com/reel\\SyntheticOne/",
        "https://instagram.com/reel/Synthetic\nOne/",
    ] {
        assert_eq!(
            parse_reference(value),
            ReferenceResolution::Unsupported(UnsupportedInstagramReference::InvalidUrl)
        );
    }
}

#[test]
fn occurrences_retain_repeated_links_tracking_and_byte_offsets() {
    let text = "Synthetic ☀ (https://instagram.com/reel/SyntheticOne/?a=1), and https://m.instagram.com/reels/SyntheticOne/?a=2. https://example.invalid/";
    let found = find_references(text);
    assert_eq!(found.len(), 3);
    assert_eq!(found[0].ordinal, 0);
    assert_eq!(found[1].ordinal, 1);
    assert_eq!(found[2].ordinal, 2);
    assert_eq!(
        &text[found[0].start..found[0].end],
        "https://instagram.com/reel/SyntheticOne/?a=1"
    );
    assert_eq!(
        &text[found[1].start..found[1].end],
        "https://m.instagram.com/reels/SyntheticOne/?a=2"
    );
    assert_eq!(found[0].resolution, found[1].resolution);
    assert_eq!(found[2].resolution, ReferenceResolution::Unrelated);
}

#[test]
fn scanner_handles_case_quotes_and_adjacent_prose_delimiters() {
    let text = r#"<HTTPS://INSTAGRAM.COM/reel/SyntheticOne/> [https://instagram.com/reel/SyntheticTwo/] "https://instagram.com/reel/SyntheticThree/""#;
    let found = find_references(text);
    assert_eq!(found.len(), 3);
    for (index, reference) in found.iter().enumerate() {
        assert_eq!(reference.ordinal, index);
        assert!(matches!(reference.resolution, ReferenceResolution::Reel(_)));
    }
}

#[test]
fn nested_url_in_unrelated_query_does_not_become_separate_evidence() {
    let found = find_references(
        "https://example.invalid/?redirect=https://instagram.com/reel/SyntheticOne/",
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].resolution, ReferenceResolution::Unrelated);
}

#[test]
fn bare_domains_and_plain_text_are_not_urls() {
    assert_eq!(
        find_references("Synthetic plain text instagram.com/reel/SyntheticOne/"),
        Vec::<teamy_instagram::reels::LocatedReference>::new()
    );
}

#[test]
fn reel_id_constructor_and_json_proxy_preserve_validation() {
    let id = ReelId::parse("Synthetic_A-1").expect("synthetic shortcode is valid");
    let json = facet_json::to_string(&id).expect("synthetic shortcode serializes");
    assert_eq!(json, r#""Synthetic_A-1""#);
    let decoded: ReelId = facet_json::from_str(&json).expect("synthetic shortcode parses");
    assert_eq!(id, decoded);
    for invalid in ["", "Synthetic/One", "Synthetic One", "例", "../Synthetic"] {
        assert_eq!(
            ReelId::parse(invalid)
                .expect_err("invalid synthetic shortcode rejected")
                .to_string(),
            "invalid reel shortcode"
        );
        let invalid_json = facet_json::to_string(&invalid.to_owned()).expect("string serializes");
        facet_json::from_str::<ReelId>(&invalid_json).expect_err("unsupported shortcode JSON");
    }
}

#[test]
fn unsupported_references_retain_only_fixed_reason() {
    let result = parse_reference("https://instagram.com/share/reel/SyntheticPrivateToken/");
    let json = facet_json::to_string(&result).expect("fixed unsupported reason serializes");
    assert!(!json.contains("SyntheticPrivateToken"));
    assert!(!json.contains("instagram.com"));
}

#[test]
fn supported_shortcode_limit_bounds_identity_size() {
    let supported = "A".repeat(128);
    let oversized = "A".repeat(129);
    assert_eq!(
        ReelId::parse(&supported)
            .expect("supported synthetic boundary shortcode parses")
            .as_str()
            .len(),
        128
    );
    ReelId::parse(&oversized).expect_err("unsupported shortcode length");
    assert_eq!(
        parse_reference(&format!("https://instagram.com/reel/{oversized}/")),
        ReferenceResolution::Unsupported(UnsupportedInstagramReference::InvalidReel)
    );
}

#[test]
fn identical_adjacent_links_still_have_distinct_occurrences() {
    let url = "https://instagram.com/reel/SyntheticRepeated/";
    let found = find_references(&format!("{url},{url};{url}"));
    assert_eq!(found.len(), 3);
    assert_eq!(found[0].resolution, found[1].resolution);
    assert_eq!(found[1].resolution, found[2].resolution);
    assert_eq!(found[2].ordinal, 2);
    assert_ne!(found[0].start, found[1].start);
}

#[test]
fn unicode_prose_quotes_end_links_and_word_prefixes_are_ignored() {
    let found = find_references(
        "synthetichttps://instagram.com/reel/SyntheticPrefix/ \u{201c}https://instagram.com/reel/SyntheticQuote/\u{201d}",
    );
    assert_eq!(found.len(), 1);
    let ReferenceResolution::Reel(reference) = &found[0].resolution else {
        panic!("synthetic quoted reel reference should resolve");
    };
    assert_eq!(reference.reel_id.as_str(), "SyntheticQuote");
}
