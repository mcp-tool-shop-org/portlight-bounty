//! Filter ruling for in-chart text.
//!
//! This file is not `chart_canvas.rs`. The needles below are absent from the
//! source under test, so reverting the label filter to nearest fails the test.

#[test]
fn chart_text_is_linear_and_the_plates_stay_nearest() {
    let src = include_str!("../src/chart_canvas.rs");
    let (art, labels) = src.split_once("struct ChartLabels").expect("label item");
    assert!(
        art.contains("self.base_mut().set_texture_filter(TextureFilter::NEAREST)"),
        "chart art and plates stay nearest"
    );
    assert!(
        art.contains("labels.set_texture_filter(TextureFilter::LINEAR)"),
        "the label item is created with a linear filter"
    );
    assert!(
        labels.contains("self.base_mut().set_texture_filter(TextureFilter::LINEAR)"),
        "ChartLabels keeps a linear filter"
    );
    let draw = labels
        .split_once("fn draw_chart_text")
        .expect("text draw")
        .1
        .split_once("fn lane_caption")
        .expect("lane caption")
        .0;
    assert!(draw.contains("font_size(PORT_NAME_PX)"));
    assert!(draw.contains("font_size(BADGE_PX)"));
    assert!(draw.contains("font_size(LANE_PX)"));
    assert!(draw.contains("font_size(HOVER_PX)"));
    assert!(
        !draw.contains("zoom"),
        "font size stays in world pixels and is not divided by zoom"
    );
}
