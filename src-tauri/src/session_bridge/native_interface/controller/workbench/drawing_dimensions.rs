//! Paper-space linear dimension presentation, matching drawing/annotations.ts.
use super::{Label, Segment};
use nbcad_sketch::{DrawingSheetStyleDto, DrawingStandard};

fn offset(point: [f64; 2], direction: [f64; 2], distance: f64) -> [f64; 2] {
    [
        point[0] + direction[0] * distance,
        point[1] + direction[1] * distance,
    ]
}

fn stroke(start: [f64; 2], end: [f64; 2], width: f64, arrow: bool) -> Segment {
    Segment {
        x1: start[0] as f32,
        y1: start[1] as f32,
        x2: end[0] as f32,
        y2: end[1] as f32,
        hidden: false,
        width_mm: width as f32,
        arrow,
        ink: super::Ink::Drawing,
    }
}

fn extension(anchor: [f64; 2], end: [f64; 2], width: f64) -> Segment {
    let delta = [end[0] - anchor[0], end[1] - anchor[1]];
    let length = delta[0].hypot(delta[1]);
    if length < 1e-7 {
        return stroke(anchor, end, width, false);
    }
    let direction = [delta[0] / length, delta[1] / length];
    stroke(
        offset(anchor, direction, 1_f64.min(length * 0.2)),
        offset(end, direction, 1.2),
        width,
        false,
    )
}

pub(super) fn layout(
    first: [f64; 2],
    second: [f64; 2],
    start: [f64; 2],
    end: [f64; 2],
    text: String,
    style: &DrawingSheetStyleDto,
    standard: DrawingStandard,
) -> (Vec<Segment>, Label) {
    let span = (end[0] - start[0]).hypot(end[1] - start[1]);
    let direction = [(end[0] - start[0]) / span, (end[1] - start[1]) / span];
    let advance: f64 = text
        .chars()
        .map(|c| {
            if c == ' ' {
                0.34
            } else if "1ilI.,:;'|".contains(c) {
                0.32
            } else if "MW@%".contains(c) {
                0.86
            } else {
                0.58
            }
        })
        .sum();
    let text_width = (style.text_height_mm * 1.8).max(advance * style.text_height_mm + 2.2);
    let arrow = style.arrow_size_mm;
    let clearance = 0.8_f64.max(arrow * 0.4);
    let outside = span < text_width + 2. * arrow + 2. * clearance;
    let text_outside = span < text_width + 2. * clearance;
    let mut line_start = start;
    let mut line_end = end;
    let mut text_point = [(start[0] + end[0]) / 2., (start[1] + end[1]) / 2.];
    if outside {
        line_start = offset(start, direction, -arrow - clearance);
        line_end = offset(end, direction, arrow + clearance);
        if text_outside {
            let text_offset = arrow + 1_f64.max(arrow * 0.55) + text_width / 2.;
            line_end = offset(end, direction, text_offset + text_width / 2. + clearance);
            text_point = offset(end, direction, text_offset);
        }
    }
    let mask = standard == DrawingStandard::Ansi && !text_outside;
    let mut angle = direction[1].atan2(direction[0]);
    if angle > std::f64::consts::FRAC_PI_2 || angle < -std::f64::consts::FRAC_PI_2 {
        angle += std::f64::consts::PI;
    }
    let baseline = if mask {
        0.8
    } else {
        1.2_f64.max(style.text_height_mm * 0.22 + style.dimension.width_mm * 2.)
    };
    // Bevy positions a text box by its center, SVG by its baseline. Rotate
    // the same baseline offset and half the glyph height about the text anchor.
    let lift = baseline + style.text_height_mm * 0.4;
    let label = Label {
        align: Default::default(),
        text,
        x: (text_point[0] + angle.sin() * lift) as f32,
        y: (text_point[1] - angle.cos() * lift) as f32,
        angle: angle as f32,
        width_mm: text_width as f32,
        height_mm: (style.text_height_mm * 1.18 + 1.5) as f32,
        text_height_mm: style.text_height_mm as f32,
        mask,
        ink: super::Ink::Drawing,
    };
    let sign = if outside { -1. } else { 1. };
    (
        vec![
            extension(first, start, style.extension.width_mm),
            extension(second, end, style.extension.width_mm),
            stroke(line_start, line_end, style.dimension.width_mm, false),
            stroke(start, offset(start, direction, arrow * sign), 0., true),
            stroke(end, offset(end, direction, -arrow * sign), 0., true),
        ],
        label,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminals_use_paper_size_and_extensions_leave_geometry_clear() {
        let style = DrawingSheetStyleDto::default();
        let (lines, label) = layout(
            [0., 0.],
            [40., 0.],
            [0., 12.],
            [40., 12.],
            "40.00".into(),
            &style,
            DrawingStandard::Iso,
        );
        assert_eq!((lines[0].y1, lines[0].y2), (1., 13.2));
        assert_eq!(lines.iter().filter(|line| line.arrow).count(), 2);
        assert_eq!((lines[3].x1, lines[3].x2), (0., 2.5));
        assert_eq!((lines[4].x1, lines[4].x2), (40., 37.5));
        assert!(!label.mask && label.y < 12.);
        assert_eq!(lines[2].width_mm, style.dimension.width_mm as f32);
        let mut style = style;
        style.arrow_size_mm = 4.;
        let (lines, label) = layout(
            [0., 0.],
            [40., 0.],
            [0., 12.],
            [40., 12.],
            "40.00".into(),
            &style,
            DrawingStandard::Ansi,
        );
        assert_eq!(lines[3].x2, 4.);
        assert!(
            label.mask,
            "ANSI inside values interrupt the dimension line"
        );
    }
    #[test]
    fn narrow_vertical_span_moves_terminals_and_value_outside() {
        let style = DrawingSheetStyleDto::default();
        let (lines, label) = layout(
            [0., 0.],
            [0., 6.],
            [8., 0.],
            [8., 6.],
            "6.00 mm".into(),
            &style,
            DrawingStandard::Ansi,
        );
        assert_eq!((lines[3].y1, lines[3].y2), (0., -2.5));
        assert_eq!((lines[4].y1, lines[4].y2), (6., 8.5));
        assert!(lines[2].y1 < 0. && lines[2].y2 > 8.5);
        assert!(label.y > 8.5 && !label.mask);
        assert!((label.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }
}
