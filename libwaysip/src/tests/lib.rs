use crate::*;

#[test]
fn builder_sets_selection_type_and_style() {
    let sip = WaySip::new()
        .with_selection_type(SelectionType::Point)
        .with_background_color(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        })
        .with_foreground_color(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        })
        .with_border_text_color(Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        })
        .with_box_color(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        })
        .with_border_weight(2.5)
        .with_font_size(20)
        .with_font_name("Mono".to_string());

    assert!(matches!(sip.selection_type, SelectionType::Point));
    assert_eq!(sip.style.background_color.r, 1.0);
    assert_eq!(sip.style.foreground_color.g, 1.0);
    assert_eq!(sip.style.border_text_color.b, 1.0);
    assert_eq!(sip.style.box_color.r, 1.0);
    assert_eq!(sip.style.border_weight, 2.5);
    assert_eq!(sip.style.font_size, 20);
    assert_eq!(sip.style.font_name, "Mono");
}

#[test]
fn builder_sets_predefined_boxes_and_aspect_ratio() {
    let boxes = vec![state::BoxInfo {
        start_x: 0.0,
        start_y: 0.0,
        end_x: 10.0,
        end_y: 10.0,
    }];
    let sip = WaySip::new()
        .with_predefined_boxes(boxes)
        .with_aspect_ratio(16.0, 9.0);

    assert_eq!(sip.predefined_boxes.as_ref().map(|b| b.len()), Some(1));
    assert_eq!(sip.aspect_ratio, Some((16.0, 9.0)));
}

#[test]
fn builder_sets_edit_selection_and_confirm_key() {
    let sip = WaySip::new().with_edit_selection().with_confirm_key(15);
    assert!(sip.edit_selection);
    assert_eq!(sip.confirm_key, Some(15));
}

#[test]
fn builder_default_has_no_edit_selection() {
    let sip = WaySip::new();
    assert!(!sip.edit_selection);
    assert!(sip.confirm_key.is_none());
}

#[test]
fn builder_sets_background_provider() {
    let sip = WaySip::new().with_background_provider(|_output, _name| None);
    assert!(sip.background_provider.is_some());
}

#[test]
fn debug_impl_does_not_panic() {
    let sip = WaySip::new().with_edit_selection().with_confirm_key(5);
    let debug_str = format!("{sip:?}");
    assert!(debug_str.contains("WaySip"));
    assert!(debug_str.contains("edit_selection: true"));
}

#[cfg(feature = "benchmark")]
#[test]
fn builder_sets_bench_flag() {
    let sip = WaySip::new().with_bench();
    assert!(sip.bench);
}
