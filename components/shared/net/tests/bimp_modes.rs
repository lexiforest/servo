use net_traits::request::Destination;
use net_traits::{
    BimpWebViewMode, bimp_mode_blocks_resource_destination, bimp_mode_disables_graphics_contexts,
    bimp_mode_disables_media, bimp_mode_disables_paint, bimp_mode_disables_webrtc,
};

#[test]
fn resource_matrix_matches_the_documented_mode_contract() {
    let always_allowed = [
        Destination::Document,
        Destination::Script,
        Destination::Xslt,
        Destination::None,
    ];
    for mode in [
        BimpWebViewMode::Nano,
        BimpWebViewMode::Flash,
        BimpWebViewMode::Full,
    ] {
        for destination in always_allowed {
            assert!(
                !bimp_mode_blocks_resource_destination(mode, destination),
                "{mode:?} must allow {destination:?}"
            );
        }
    }

    for destination in [
        Destination::Audio,
        Destination::Font,
        Destination::Image,
        Destination::Manifest,
        Destination::Style,
        Destination::Track,
        Destination::Video,
    ] {
        assert!(bimp_mode_blocks_resource_destination(
            BimpWebViewMode::Nano,
            destination
        ));
    }

    for destination in [
        Destination::Font,
        Destination::Audio,
        Destination::Track,
        Destination::Video,
    ] {
        assert!(bimp_mode_blocks_resource_destination(
            BimpWebViewMode::Flash,
            destination
        ));
    }
    for destination in [
        Destination::Image,
        Destination::Manifest,
        Destination::Style,
    ] {
        assert!(!bimp_mode_blocks_resource_destination(
            BimpWebViewMode::Flash,
            destination
        ));
    }

    for destination in [
        Destination::Audio,
        Destination::Font,
        Destination::Image,
        Destination::Manifest,
        Destination::Style,
        Destination::Track,
        Destination::Video,
    ] {
        assert!(!bimp_mode_blocks_resource_destination(
            BimpWebViewMode::Full,
            destination
        ));
    }
}

#[test]
fn rendering_media_and_graphics_policies_match_the_mode_contract() {
    assert!(bimp_mode_disables_paint(BimpWebViewMode::Nano));
    assert!(!bimp_mode_disables_paint(BimpWebViewMode::Flash));
    assert!(!bimp_mode_disables_paint(BimpWebViewMode::Full));

    for mode in [BimpWebViewMode::Nano, BimpWebViewMode::Flash] {
        assert!(bimp_mode_disables_media(mode));
        assert!(bimp_mode_disables_graphics_contexts(mode));
        assert!(bimp_mode_disables_webrtc(mode));
    }
    assert!(!bimp_mode_disables_media(BimpWebViewMode::Full));
    assert!(!bimp_mode_disables_graphics_contexts(BimpWebViewMode::Full));
    assert!(!bimp_mode_disables_webrtc(BimpWebViewMode::Full));
}
