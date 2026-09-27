//! Retained presentation of the existing drawing annotations. The document is
//! authoritative; these paper strokes and labels are disposable display data.
use super::{dimension_span, dimensions, paper_point, Fill, Ink, Label, LabelAlign, Segment};
use nbcad_core::UnitSystem;
use nbcad_occt::DrawingProjectionDto;
use nbcad_sketch::*;
use std::collections::BTreeMap;
#[path = "drawing_annotations/geometry.rs"]
mod geometry;
#[path = "drawing_annotations/text.rs"]
mod text;
use geometry::*;
#[path = "drawing_annotations/budget.rs"]
mod budget;
#[path = "drawing_annotations/frame.rs"]
mod frame;

pub(in super::super) fn valid_line_dimension(first: [P; 2], second: Option<[P; 2]>,
    mode: DrawingLineDimensionMode, position: P, scale: f64) -> bool {
    geometry::line_dimension(first, second, mode, position, scale).is_some()
}
pub(in super::super) fn valid_point_line(point: P, line: [P; 2], position: P, scale: f64) -> bool {
    geometry::point_line(point, line, position, scale).is_some()
}

pub(super) fn linear_points(
    view: &DrawingViewDto,
    projection: &DrawingProjectionDto,
    first: &DrawingTopologyAnchorRefDto,
    second: &DrawingTopologyAnchorRefDto,
) -> Option<[[f64; 2]; 2]> {
    let resolver = Resolver { view, projection };
    Some([resolver.anchor(first)?, resolver.anchor(second)?])
}

pub(super) fn anchor_points(
    view: &DrawingViewDto,
    projection: &DrawingProjectionDto,
    references: [&DrawingTopologyAnchorRefDto; 3],
) -> Option<[P; 3]> {
    let r = Resolver { view, projection };
    Some([
        r.anchor(references[0])?,
        r.anchor(references[1])?,
        r.anchor(references[2])?,
    ])
}
pub(super) fn radial_drag_geometry(
    view: &DrawingViewDto,
    projection: &DrawingProjectionDto,
    feature: &DrawingCircularRefDto,
    angle: f64,
    offset: f64,
) -> Option<super::RadialDrag> {
    let c = (Resolver { view, projection }).circle(feature)?;
    let angle = angle.to_radians();
    let shoulder = add(
        c.center,
        scale([angle.cos(), angle.sin()], c.radius + offset),
    );
    shoulder
        .iter()
        .chain(&c.center)
        .all(|n| n.is_finite())
        .then_some(super::RadialDrag {
            center: c.center,
            paper_radius: c.radius,
            shoulder,
        })
}
pub(super) fn angular_drag_geometry(
    view: &DrawingViewDto,
    projection: &DrawingProjectionDto,
    vertex: &DrawingTopologyAnchorRefDto,
    first: &DrawingTopologyAnchorRefDto,
    second: &DrawingTopologyAnchorRefDto,
    radius: f64,
) -> Option<super::AngularDrag> {
    let [vertex, first, second] = anchor_points(view, projection, [vertex, first, second])?;
    if !radius.is_finite() || radius <= 0. {
        return None;
    }
    let g = angular(vertex, first, second, radius)?;
    Some(super::AngularDrag {
        vertex: g.vertex,
        text: g.text,
    })
}

#[derive(Default)]
pub(super) struct Art {
    pub segments: Vec<Segment>,
    pub labels: Vec<Label>,
    pub fills: Vec<Fill>,
    pub marks: Vec<super::AnnotationMark>,
}
#[derive(Default)]
pub(super) struct CheckedArt {
    art: Art,
    budget: budget::Budget,
}
impl std::ops::Deref for CheckedArt {
    type Target = Art;
    fn deref(&self) -> &Art {
        &self.art
    }
}
impl CheckedArt {
    fn segment(&mut self, value: Segment) {
        if !budget::finite_segment(&value) {
            self.budget
                .reject("Drawing annotations exceed finite render coordinates");
            return;
        }
        if self
            .budget
            .reserve(&mut self.art.segments, 1, self.budget.limits.segments)
        {
            self.art.segments.push(value);
        }
    }
    pub(super) fn fill(&mut self, value: Fill) {
        if !budget::finite_fill(&value) {
            self.budget
                .reject("Drawing annotations exceed finite render coordinates");
            return;
        }
        if self
            .budget
            .reserve(&mut self.art.fills, 1, self.budget.limits.fills)
        {
            self.art.fills.push(value);
        }
    }
    pub(super) fn push_label(&mut self, value: Label) {
        if !budget::finite_label(&value) {
            self.budget
                .reject("Drawing annotation text exceeds finite render coordinates");
            return;
        }
        if self.budget.text(value.text.capacity())
            && self
                .budget
                .reserve(&mut self.art.labels, 1, self.budget.limits.labels)
        {
            self.art.labels.push(value);
        }
    }
    fn label_text(&mut self, mut value: Label, text: &str) {
        if !budget::finite_label(&value) {
            self.budget
                .reject("Drawing annotation text exceeds finite render coordinates");
            return;
        }
        if self.budget.text(text.len())
            && self
                .budget
                .reserve(&mut self.art.labels, 1, self.budget.limits.labels)
        {
            value.text = text.into();
            self.art.labels.push(value);
        }
    }
    fn checkpoint(&self) -> [usize; 3] {
        [self.segments.len(), self.labels.len(), self.fills.len()]
    }
    fn rollback(&mut self, mark: [usize; 3]) {
        self.art.segments.truncate(mark[0]);
        self.art.labels.truncate(mark[1]);
        self.art.fills.truncate(mark[2]);
    }
    fn mark(
        &mut self,
        annotation: &DrawingAnnotationDto,
        projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
        label_index: usize,
    ) {
        use DrawingAnnotationDto::*;
        if !matches!(
            annotation,
            Note { .. }
                | LinearDimension { .. }
                | RadialDimension { .. }
                | AngularDimension { .. }
                | ChainDimension { .. }
                | OrdinateDimension { .. }
                | LineDimension { .. }
                | PointLineDimension { .. }
        ) {
            return;
        }
        let mut mark = super::AnnotationMark {
            id: annotation.id(),
            part: 0,
            center: [0.; 2],
            size: [0.; 2],
            angle: 0.,
            linear_points: None,
            radial: None,
            angular: None,
            ordinate_points: None,
            position_resolved: false,
        };
        if let Some((view, projection)) = view_id(annotation).and_then(|id| projections.get(&id)) {
            match annotation {
                LinearDimension { first, second, .. } => {
                    mark.linear_points = linear_points(view, projection, first, second)
                }
                ChainDimension { anchors, .. } => {
                    // A broken later association must not become a draggable
                    // partial series just because the first two still resolve.
                    let resolver = Resolver { view, projection };
                    let mut points = anchors.iter().map(|a| resolver.anchor(a));
                    let first = points.next().flatten();
                    let second = points.next().flatten();
                    if points.all(|p| p.is_some()) {
                        mark.linear_points = first.zip(second).map(|(a, b)| [a, b]);
                    }
                }
                OrdinateDimension { origin, target, .. } => {
                    mark.ordinate_points = linear_points(view, projection, origin, target);
                }
                LineDimension { first, second, mode, position, .. } => {
                    let r = Resolver { view, projection };
                    mark.position_resolved = r.line(first).is_some_and(|a| {
                        let b = match second {
                            Some(reference) => match r.line(reference) { Some(b) => Some(b), None => return false },
                            None => None,
                        };
                        valid_line_dimension(a, b, *mode, *position, view.scale)
                    });
                }
                PointLineDimension { point, line, position, .. } => {
                    let r = Resolver { view, projection };
                    mark.position_resolved = r.anchor(point).zip(r.line(line))
                        .is_some_and(|(p,l)| valid_point_line(p,l,*position,view.scale));
                }
                RadialDimension {
                    feature,
                    leader_angle_deg,
                    offset,
                    ..
                } => {
                    mark.radial =
                        radial_drag_geometry(view, projection, feature, *leader_angle_deg, *offset)
                }
                AngularDimension {
                    vertex,
                    first,
                    second,
                    radius,
                    ..
                } => {
                    mark.angular =
                        angular_drag_geometry(view, projection, vertex, first, second, *radius)
                }
                _ => {}
            }
        }
        // Each series span owns its painted label only. Empty paper between
        // labels stays available to the view and paper navigation tools.
        let label_end = if matches!(annotation, ChainDimension { .. }) {
            self.labels.len()
        } else {
            label_index + 1
        };
        for (part, index) in (label_index..label_end).enumerate() {
            let Some(label) = self.labels.get(index) else {
                return;
            };
            let mut mark = mark.clone();
            mark.part = part;
            mark.center = [label.x as f64, label.y as f64];
            mark.size = [label.width_mm as f64, label.height_mm as f64];
            mark.angle = label.angle;
            if self
                .budget
                .reserve(&mut self.art.marks, 1, self.budget.limits.labels)
            {
                self.art.marks.push(mark);
            }
        }
    }

    fn disc(&mut self, center: P, radius: f64, stroke_width: f64) {
        let radius = (radius - stroke_width * 0.5).max(0.);
        self.fill(Fill {
            x: (center[0] - radius) as f32,
            y: (center[1] - radius) as f32,
            width: (radius * 2.) as f32,
            height: (radius * 2.) as f32,
            round: true,
        });
    }
    fn stroke(&mut self, a: P, b: P, width: f64, ink: Ink) {
        if length(sub(b, a)) < 1e-8 {
            return;
        }
        self.segment(Segment {
            x1: a[0] as f32,
            y1: a[1] as f32,
            x2: b[0] as f32,
            y2: b[1] as f32,
            width_mm: width as f32,
            hidden: false,
            arrow: false,
            ink,
        });
    }
    fn line(&mut self, a: P, b: P, style: &DrawingLineStyleDto, ink: Ink) {
        self.styled_path(&[a, b], style, ink);
    }
    fn polyline(&mut self, points: &[P], style: &DrawingLineStyleDto, ink: Ink) {
        self.styled_path(points, style, ink);
    }
    fn circle(&mut self, center: P, radius: f64, style: &DrawingLineStyleDto, ink: Ink) {
        self.polyline(&arc(center, radius, 0., std::f64::consts::TAU), style, ink);
    }
    fn arrow(&mut self, tip: P, toward: P, size: f64, ink: Ink) {
        let Some(direction) = unit(sub(toward, tip)) else {
            return;
        };
        let end = add(tip, scale(direction, size));
        self.segment(Segment {
            x1: tip[0] as f32,
            y1: tip[1] as f32,
            x2: end[0] as f32,
            y2: end[1] as f32,
            width_mm: 0.,
            hidden: false,
            arrow: true,
            ink,
        });
    }
    fn label(&mut self, baseline: P, value: String, size: f64, align: f64, mask: bool, ink: Ink) {
        if !self.budget.work(value.len() as u64) || value.len() > self.budget.limits.text {
            self.budget
                .reject("Drawing annotations exceed the text limit");
            return;
        }
        for (row, text) in value.lines().enumerate() {
            if !self.budget.ready() {
                return;
            }
            let advance: f64 = text
                .chars()
                .map(|c| {
                    if c == '\u{fe0e}' {
                        0.
                    } else if c == ' ' {
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
            let width = (size * 1.8).max(advance * size + 2.2);
            self.label_text(
                Label {
                    text: String::new(),
                    x: (baseline[0] + align * width * 0.5) as f32,
                    y: (baseline[1] - size * 0.4 + row as f64 * size * 1.25) as f32,
                    angle: 0.,
                    width_mm: width as f32,
                    height_mm: (size * 1.18 + 1.5) as f32,
                    text_height_mm: size as f32,
                    mask,
                    ink,
                    align: if align > 0. {
                        LabelAlign::Start
                    } else if align < 0. {
                        LabelAlign::End
                    } else {
                        LabelAlign::Center
                    },
                },
                text,
            );
        }
    }
    fn leader(&mut self, attachment: P, position: P, style: &DrawingSheetStyleDto) {
        self.line(attachment, position, &style.leader, Ink::Drawing);
        self.arrow(attachment, position, style.arrow_size_mm, Ink::Drawing);
    }
    fn rect(&mut self, left: f64, top: f64, width: f64, height: f64, style: &DrawingLineStyleDto) {
        let inset = style.width_mm * 0.5;
        self.fill(Fill {
            x: (left + inset) as f32,
            y: (top + inset) as f32,
            width: (width - 2. * inset).max(0.) as f32,
            height: (height - 2. * inset).max(0.) as f32,
            round: false,
        });
        self.polyline(
            &[
                [left, top],
                [left + width, top],
                [left + width, top + height],
                [left, top + height],
                [left, top],
            ],
            style,
            Ink::Drawing,
        );
    }
    fn linear(&mut self, g: Linear, value: String, sheet: &DrawingSheetDto) {
        let (strokes, label) = dimensions::layout(
            g.first,
            g.second,
            g.start,
            g.end,
            value,
            &sheet.style,
            sheet.standard,
        );
        for stroke in strokes {
            self.segment(stroke);
        }
        self.push_label(label);
    }
    fn angular(&mut self, g: Angular, value: String, style: &DrawingSheetStyleDto) {
        self.line(g.vertex, g.first, &style.extension, Ink::Drawing);
        self.line(g.vertex, g.second, &style.extension, Ink::Drawing);
        self.polyline(&g.points, &style.dimension, Ink::Drawing);
        if let (Some(first), Some(last)) = (g.points.first(), g.points.last()) {
            self.arrow(*first, g.points[1], style.arrow_size_mm, Ink::Drawing);
            self.arrow(
                *last,
                g.points[g.points.len() - 2],
                style.arrow_size_mm,
                Ink::Drawing,
            );
        }
        self.label(g.text, value, style.text_height_mm, 0., true, Ink::Drawing);
    }
    fn center_mark(&mut self, circle: Circle, extension: f64, style: &DrawingSheetStyleDto) {
        let extent = circle.radius + extension.max(0.);
        self.line(
            add(circle.center, [-extent, 0.]),
            add(circle.center, [extent, 0.]),
            &style.center,
            Ink::Center,
        );
        self.line(
            add(circle.center, [0., -extent]),
            add(circle.center, [0., extent]),
            &style.center,
            Ink::Center,
        );
        self.circle(
            circle.center,
            0.48,
            &DrawingLineStyleDto {
                width_mm: 0.36,
                dash_mm: vec![],
            },
            Ink::Center,
        );
        self.disc(circle.center, 0.48, 0.36);
    }
}

pub(super) fn try_render(
    sheet: &DrawingSheetDto,
    projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
    units: UnitSystem,
) -> Result<Art, String> {
    render_checked(sheet, projections, units, budget::Limits::default()).map(|b| b.art)
}
fn render_checked(
    sheet: &DrawingSheetDto,
    projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
    units: UnitSystem,
    limits: budget::Limits,
) -> Result<CheckedArt, String> {
    let mut art = CheckedArt {
        art: Art::default(),
        budget: budget::Budget::new(limits),
    };
    for annotation in &sheet.annotations {
        art.budget.work(1);
        art.budget.input(
            annotation,
            view_id(annotation).and_then(|id| projections.get(&id).map(|(_, p)| p)),
        );
        art.budget.check()?;
        if let DrawingAnnotationDto::ItemBalloon { bom_item_id, .. } = annotation {
            if let Some(item) = sheet.bom.iter().find(|b| b.id == *bom_item_id) {
                if item.item_number.len() > art.budget.limits.text {
                    return Err("Drawing balloon text exceeds the text limit".into());
                }
                art.budget.work(item.item_number.len() as u64);
                art.budget.check()?;
            }
        }
        let mark = art.checkpoint();
        if let DrawingAnnotationDto::Note { text, position, .. } = annotation {
            art.label(
                *position,
                text.clone(),
                sheet.style.text_height_mm,
                1.,
                false,
                Ink::Drawing,
            );
        } else if let DrawingAnnotationDto::RevisionCloud {
            revision, points, ..
        } = annotation
        {
            let style = DrawingLineStyleDto {
                width_mm: 0.45,
                dash_mm: vec![],
            };
            if points.len() > 1 {
                let scallops = (0..points.len())
                    .map(|i| length(sub(points[(i + 1) % points.len()], points[i])) / 5.)
                    .map(f64::ceil)
                    .sum::<f64>();
                if !art.budget.steps(scallops * 513.) {
                    return Err(art.budget.error.take().unwrap());
                }
                for index in 0..points.len() {
                    let start = points[index];
                    let end = points[(index + 1) % points.len()];
                    let Some(dir) = unit(sub(end, start)) else {
                        continue;
                    };
                    let distance = length(sub(end, start));
                    let count = (distance / 5.).ceil().max(1.) as usize;
                    let step = distance / count as f64;
                    let radius = (step * 0.58).max(1.4);
                    for i in 0..count {
                        if !art.budget.work(1) {
                            return Err(art.budget.error.take().unwrap());
                        }
                        let a = add(start, scale(dir, i as f64 * step));
                        let b = add(start, scale(dir, (i + 1) as f64 * step));
                        let center = add(
                            midpoint(a, b),
                            scale(normal(dir), (radius * radius - step * step * 0.25).sqrt()),
                        );
                        let from = sub(a, center);
                        art.polyline(
                            &arc(
                                center,
                                radius,
                                from[1].atan2(from[0]),
                                2. * (step / (2. * radius)).asin(),
                            ),
                            &style,
                            Ink::Revision,
                        );
                    }
                }
                let min = points
                    .iter()
                    .fold([f64::INFINITY; 2], |v, p| [v[0].min(p[0]), v[1].min(p[1])]);
                art.label(
                    add(min, [0., -2.]),
                    format!("REV {revision}"),
                    3.2,
                    1.,
                    false,
                    Ink::Revision,
                );
            }
        } else {
            let id = view_id(annotation).expect("view-bound annotation");
            let result = projections.get(&id).and_then(|(view, projection)| {
                render_view(
                    &mut art,
                    annotation,
                    sheet,
                    &Resolver { view, projection },
                    units,
                )
            });
            if result.is_none() {
                // A stale association is a visible diagnostic, never guessed
                // fallback geometry or silent disappearance of saved intent.
                art.budget.check()?;
                art.rollback(mark);
                let position = sheet
                    .views
                    .iter()
                    .find(|v| v.id == id)
                    .map_or([20., 20.], |v| add(v.position, [0., -8.]));
                art.circle(
                    position,
                    3.1,
                    &DrawingLineStyleDto {
                        width_mm: 0.45,
                        dash_mm: vec![],
                    },
                    Ink::Revision,
                );
                art.disc(position, 3.1, 0.45);
                art.label(
                    add(position, [0., 1.2]),
                    "!".into(),
                    3.5,
                    0.,
                    true,
                    Ink::Revision,
                );
            }
        }
        art.budget
            .check()
            .map_err(|e| format!("Annotation {}: {e}", annotation.id()))?;
        art.mark(annotation, projections, mark[1]);
        art.budget.check()?;
    }
    Ok(art)
}
/// Add view/source captions under the same final storage and label limits.
pub(super) fn try_render_decorated(
    sheet: &DrawingSheetDto,
    projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
    units: UnitSystem,
    source_labels: &[Label],
) -> Result<Art, String> {
    let mut b = render_checked(sheet, projections, units, budget::Limits::default())?;
    for (view, projection) in projections.values() {
        if view.name.len() > b.budget.limits.text {
            return Err("Drawing view captions exceed the text limit".into());
        }
        b.budget.work(view.name.len() as u64);
        b.budget.check()?;
        b.push_label(super::view_name_label(
            view,
            projection,
            sheet.style.small_text_height_mm,
        ));
        b.budget.check()?;
    }
    for label in source_labels {
        // Check before cloning a potentially long cached source label.
        if label.text.len() > b.budget.limits.text {
            return Err("Drawing source captions exceed the text limit".into());
        }
        b.budget.work(label.text.len() as u64);
        b.budget.check()?;
        b.label_text(
            Label {
                text: String::new(),
                x: label.x,
                y: label.y,
                angle: label.angle,
                width_mm: label.width_mm,
                height_mm: label.height_mm,
                text_height_mm: label.text_height_mm,
                mask: label.mask,
                ink: label.ink,
                align: label.align,
            },
            &label.text,
        );
        b.budget.check()?;
    }
    Ok(b.art)
}
#[cfg(test)]
fn render_with_limits(
    sheet: &DrawingSheetDto,
    projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
    units: UnitSystem,
    limits: budget::Limits,
) -> Result<Art, String> {
    render_checked(sheet, projections, units, limits).map(|b| b.art)
}
#[cfg(test)]
fn render(
    sheet: &DrawingSheetDto,
    projections: &BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>,
    units: UnitSystem,
) -> Art {
    try_render(sheet, projections, units).unwrap()
}
fn view_id(annotation: &DrawingAnnotationDto) -> Option<u64> {
    use DrawingAnnotationDto::*;
    match annotation {
        Note { .. } | RevisionCloud { .. } => None,
        LinearDimension { view_id, .. }
        | LineDimension { view_id, .. }
        | PointLineDimension { view_id, .. }
        | RadialDimension { view_id, .. }
        | AngularDimension { view_id, .. }
        | HoleNote { view_id, .. }
        | ChamferNote { view_id, .. }
        | CenterMark { view_id, .. }
        | CenterLine { view_id, .. }
        | CenterLineBetweenEdges { view_id, .. }
        | AutomaticSymmetryAxis { view_id, .. }
        | BoltCircleCenterLine { view_id, .. }
        | ChainDimension { view_id, .. }
        | OrdinateDimension { view_id, .. }
        | ArcLengthDimension { view_id, .. }
        | JoggedRadiusDimension { view_id, .. }
        | DatumFeature { view_id, .. }
        | GdtFrame { view_id, .. }
        | SurfaceTexture { view_id, .. }
        | EdgeRequirement { view_id, .. }
        | WeldSymbol { view_id, .. }
        | ItemBalloon { view_id, .. } => Some(*view_id),
    }
}

fn render_view(
    art: &mut CheckedArt,
    annotation: &DrawingAnnotationDto,
    sheet: &DrawingSheetDto,
    r: &Resolver<'_>,
    units: UnitSystem,
) -> Option<()> {
    use DrawingAnnotationDto::*;
    let style = &sheet.style;
    match annotation {
        Note { .. } | RevisionCloud { .. } => unreachable!(),
        LinearDimension {
            first,
            second,
            mode,
            offset,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let first = r.anchor(first)?;
            let second = r.anchor(second)?;
            let (value, _, _, start, end) =
                dimension_span(*mode, first, second, *offset, r.view.scale)?;
            art.linear(
                Linear {
                    first,
                    second,
                    start,
                    end,
                    value,
                },
                text::dimension(value, *precision, prefix, suffix, units, presentation),
                sheet,
            );
        }
        LineDimension {
            first,
            second,
            mode,
            position,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let first = r.line(first)?;
            let second = match second {
                Some(second) => Some(r.line(second)?),
                None => None,
            };
            match line_dimension(first, second, *mode, *position, r.view.scale)? {
                geometry::LineDimension::Linear(g) => {
                    let text =
                        text::dimension(g.value, *precision, prefix, suffix, units, presentation);
                    art.linear(g, text, sheet);
                }
                geometry::LineDimension::Angular(g) => {
                    let text = text::angular(g.value, *precision, prefix, suffix, presentation);
                    art.angular(g, text, style);
                }
            }
        }
        PointLineDimension {
            point,
            line,
            position,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let g = point_line(r.anchor(point)?, r.line(line)?, *position, r.view.scale)?;
            let text = text::dimension(g.value, *precision, prefix, suffix, units, presentation);
            art.linear(g, text, sheet);
        }
        RadialDimension {
            feature,
            mode,
            leader_angle_deg,
            offset,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let c = r.circle(feature)?;
            let angle = leader_angle_deg.to_radians();
            let dir = [angle.cos(), angle.sin()];
            let feature = add(c.center, scale(dir, c.radius));
            let shoulder = add(c.center, scale(dir, c.radius + offset));
            art.polyline(
                &[c.center, feature, shoulder],
                &style.dimension,
                Ink::Drawing,
            );
            art.arrow(feature, c.center, style.arrow_size_mm, Ink::Drawing);
            let (value, symbol) = if *mode == DrawingRadialDimensionMode::Diameter {
                (c.model_radius * 2., "⌀")
            } else {
                (c.model_radius, "R")
            };
            art.label(
                add(shoulder, [if dir[0] >= 0. { 2. } else { -2. }, -1.]),
                text::dimension(
                    value,
                    *precision,
                    &format!("{prefix}{symbol}"),
                    suffix,
                    units,
                    presentation,
                ),
                style.text_height_mm,
                if dir[0] >= 0. { 1. } else { -1. },
                true,
                Ink::Drawing,
            );
        }
        AngularDimension {
            vertex,
            first,
            second,
            radius,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let g = angular(
                r.anchor(vertex)?,
                r.anchor(first)?,
                r.anchor(second)?,
                *radius,
            )?;
            let text = text::angular(g.value, *precision, prefix, suffix, presentation);
            art.angular(g, text, style);
        }
        HoleNote {
            feature, position, ..
        } => {
            let c = r.circle(feature)?;
            let direction = unit(sub(*position, c.center))?;
            let edge = add(c.center, scale(direction, c.radius));
            art.leader(edge, *position, style);
            art.label(
                add(*position, [1.2, -0.8]),
                text::hole(annotation, units, sheet.standard),
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        ChamferNote {
            first,
            second,
            position,
            length,
            angle_deg,
            prefix,
            ..
        } => {
            art.leader(
                midpoint(r.anchor(first)?, r.anchor(second)?),
                *position,
                style,
            );
            art.label(
                add(*position, [1.2, -0.8]),
                text::chamfer(*length, *angle_deg, prefix, units, sheet.standard),
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        CenterMark {
            feature, extension, ..
        } => art.center_mark(r.circle(feature)?, *extension, style),
        CenterLine {
            first,
            second,
            extension,
            ..
        } => {
            let a = r.circle(first)?;
            let b = r.circle(second)?;
            let dir = unit(sub(b.center, a.center))?;
            art.line(
                add(a.center, scale(dir, -a.radius - extension.max(0.))),
                add(b.center, scale(dir, b.radius + extension.max(0.))),
                &style.center,
                Ink::Center,
            );
            for c in [a, b] {
                art.circle(
                    c.center,
                    0.48,
                    &DrawingLineStyleDto {
                        width_mm: 0.36,
                        dash_mm: vec![],
                    },
                    Ink::Center,
                );
                art.disc(c.center, 0.48, 0.36);
            }
        }
        CenterLineBetweenEdges {
            first,
            second,
            extension,
            ..
        } => {
            let [a, b] = center_between(r.line(first)?, r.line(second)?, *extension)?;
            art.line(a, b, &style.center, Ink::Center);
        }
        AutomaticSymmetryAxis {
            axis, extension, ..
        } => {
            let b = r.projection.bounds;
            let a = paper_point(r.view, [b[0], b[1]], r.projection);
            let b = paper_point(r.view, [b[2], b[3]], r.projection);
            let c = midpoint(a, b);
            let extra = extension.max(0.);
            if *axis != DrawingOrdinateAxis::Y {
                art.line(
                    [a[0].min(b[0]) - extra, c[1]],
                    [a[0].max(b[0]) + extra, c[1]],
                    &style.center,
                    Ink::Center,
                );
            }
            if *axis != DrawingOrdinateAxis::X {
                art.line(
                    [c[0], a[1].min(b[1]) - extra],
                    [c[0], a[1].max(b[1]) + extra],
                    &style.center,
                    Ink::Center,
                );
            }
        }
        BoltCircleCenterLine {
            features,
            extension,
            ..
        } => {
            let circles: Vec<_> = features
                .iter()
                .map(|f| r.circle(f))
                .collect::<Option<_>>()?;
            if circles.len() < 3 {
                return None;
            }
            let center = scale(
                circles.iter().fold([0., 0.], |sum, c| add(sum, c.center)),
                1. / circles.len() as f64,
            );
            let radius = circles
                .iter()
                .map(|c| length(sub(c.center, center)))
                .sum::<f64>()
                / circles.len() as f64;
            if radius < 1e-5
                || circles.iter().any(|c| {
                    (length(sub(c.center, center)) - radius).abs() > 0.35_f64.max(radius * 0.015)
                })
            {
                return None;
            }
            art.circle(center, radius, &style.center, Ink::Center);
            for c in circles {
                if !art.budget.ready() {
                    return Some(());
                }
                art.center_mark(c, *extension, style);
            }
        }
        ChainDimension {
            anchors,
            mode,
            layout,
            offset,
            spacing,
            prefix,
            suffix,
            precision,
            presentation,
            ..
        } => {
            let anchors: Vec<_> = anchors.iter().map(|a| r.anchor(a)).collect::<Option<_>>()?;
            for (index, second) in anchors.iter().skip(1).enumerate() {
                if !art.budget.ready() {
                    return Some(());
                }
                let first = anchors[if *layout == DrawingChainDimensionLayout::Baseline {
                    0
                } else {
                    index
                }];
                let offset = offset
                    + if *layout == DrawingChainDimensionLayout::Baseline {
                        index as f64 * spacing
                    } else {
                        0.
                    };
                let (value, _, _, start, end) =
                    dimension_span(*mode, first, *second, offset, r.view.scale)?;
                art.linear(
                    Linear {
                        first,
                        second: *second,
                        start,
                        end,
                        value,
                    },
                    text::dimension(value, *precision, prefix, suffix, units, presentation),
                    sheet,
                );
            }
        }
        OrdinateDimension {
            origin,
            target,
            axis,
            offset,
            precision,
            presentation,
            ..
        } => {
            let origin = r.anchor(origin)?;
            let target = r.anchor(target)?;
            let delta = sub(target, origin);
            if length(delta) < 1e-7 {
                return None;
            }
            let horizontal = delta[0].abs() >= delta[1].abs();
            let elbow = add(
                target,
                if horizontal {
                    [0., *offset]
                } else {
                    [*offset, 0.]
                },
            );
            let sign = if *offset < 0. { -1. } else { 1. };
            let position = add(
                elbow,
                if horizontal {
                    [0., sign * 2.]
                } else {
                    [sign * 2., 0.]
                },
            );
            let x = text::dimension(
                delta[0] / r.view.scale,
                *precision,
                "X",
                "",
                units,
                presentation,
            );
            let y = text::dimension(
                -delta[1] / r.view.scale,
                *precision,
                "Y",
                "",
                units,
                presentation,
            );
            let text = match axis {
                DrawingOrdinateAxis::X => x,
                DrawingOrdinateAxis::Y => y,
                DrawingOrdinateAxis::Both => format!("{x}  {y}"),
            };
            // Match OrdinateDimensionGraphic: an outlined datum origin,
            // complete leader/arrow, and start-anchored baseline text.
            art.fill(Fill {
                x: (origin[0] - 1.2) as f32,
                y: (origin[1] - 1.2) as f32,
                width: 2.4,
                height: 2.4,
                round: true,
            });
            art.circle(
                origin,
                1.2,
                &DrawingLineStyleDto {
                    width_mm: 0.45,
                    dash_mm: vec![],
                },
                Ink::Drawing,
            );
            art.polyline(&[target, elbow, position], &style.dimension, Ink::Drawing);
            art.arrow(target, elbow, style.arrow_size_mm, Ink::Drawing);
            art.label(
                add(position, [0., -0.7]),
                text,
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        ArcLengthDimension {
            feature,
            first,
            second,
            offset,
            precision,
            presentation,
            ..
        } => {
            let c = r.circle(feature)?;
            let a = sub(r.anchor(first)?, c.center);
            let b = sub(r.anchor(second)?, c.center);
            unit(a)?;
            unit(b)?;
            let start = a[1].atan2(a[0]);
            let mut sweep = (b[1].atan2(b[0]) - start).rem_euclid(std::f64::consts::TAU);
            if sweep > std::f64::consts::PI {
                sweep -= std::f64::consts::TAU;
            }
            if sweep.abs() < 1e-7 {
                return None;
            }
            let radius = c.radius + offset.max(1.);
            let points = arc(c.center, radius, start, sweep);
            art.polyline(&points, &style.dimension, Ink::Drawing);
            art.arrow(points[0], points[1], style.arrow_size_mm, Ink::Drawing);
            art.arrow(
                *points.last()?,
                points[points.len() - 2],
                style.arrow_size_mm,
                Ink::Drawing,
            );
            let angle = start + sweep * 0.5;
            let position = add(c.center, scale([angle.cos(), angle.sin()], radius + 3.));
            art.label(
                position,
                text::dimension(
                    c.model_radius * sweep.abs(),
                    *precision,
                    "⌒",
                    "",
                    units,
                    presentation,
                ),
                style.text_height_mm,
                0.,
                true,
                Ink::Drawing,
            );
        }
        JoggedRadiusDimension {
            feature,
            jog,
            position,
            precision,
            presentation,
            ..
        } => {
            let c = r.circle(feature)?;
            let dir = unit(sub(*position, c.center))?;
            let normal = normal(dir);
            let edge = add(c.center, scale(dir, c.radius));
            let before = sub(sub(*jog, scale(dir, 2.)), scale(normal, 1.5));
            let after = add(add(*jog, scale(dir, 2.)), scale(normal, 1.5));
            art.polyline(
                &[edge, before, after, *position],
                &style.dimension,
                Ink::Drawing,
            );
            art.arrow(edge, before, style.arrow_size_mm, Ink::Drawing);
            art.label(
                add(*position, [1.2, -0.8]),
                text::dimension(c.model_radius, *precision, "R", "", units, presentation),
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        DatumFeature {
            attachment,
            label,
            position,
            target_index,
            ..
        } => {
            art.leader(r.attachment(attachment)?, *position, style);
            art.rect(position[0], position[1] - 4.4, 7., 5.5, &style.leader);
            art.label(
                add(*position, [3.5, -0.5]),
                target_index.map_or(label.clone(), |n| format!("{label}{n}")),
                style.text_height_mm,
                0.,
                false,
                Ink::Drawing,
            );
        }
        GdtFrame {
            attachment,
            position,
            ..
        } => {
            art.leader(r.attachment(attachment)?, *position, style);
            let mut x = position[0];
            for cell in text::gdt_cells(annotation) {
                if !art.budget.ready() {
                    return Some(());
                }
                let width =
                    (cell.chars().filter(|c| *c != '\u{fe0e}').count() as f64 * 2.1 + 3.).max(7.);
                art.rect(x, position[1] - 5., width, 6., &style.leader);
                art.label(
                    [x + width * 0.5, position[1] - 0.8],
                    cell.into(),
                    style.text_height_mm,
                    0.,
                    false,
                    Ink::Drawing,
                );
                x += width;
            }
        }
        SurfaceTexture {
            attachment,
            position,
            roughness_ra,
            process,
            ..
        } => {
            art.leader(r.attachment(attachment)?, *position, style);
            art.polyline(
                &[
                    *position,
                    add(*position, [3., -6.]),
                    add(*position, [6., 0.]),
                ],
                &style.leader,
                Ink::Drawing,
            );
            art.line(
                add(*position, [3., -6.]),
                add(*position, [9., -6.]),
                &style.leader,
                Ink::Drawing,
            );
            art.label(
                add(*position, [7., -2.]),
                format!(
                    "Ra {}{}",
                    text::trim(*roughness_ra, 4),
                    if process.is_empty() {
                        String::new()
                    } else {
                        format!(" {process}")
                    }
                ),
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        EdgeRequirement {
            attachment,
            position,
            upper_deviation,
            lower_deviation,
            note,
            ..
        } => {
            let [a, b] = r.line(attachment)?;
            art.leader(midpoint(a, b), *position, style);
            art.polyline(
                &[
                    *position,
                    add(*position, [0., -5.]),
                    add(*position, [5., -5.]),
                ],
                &style.leader,
                Ink::Drawing,
            );
            let value = format!(
                "{}{} / {}{}",
                if *upper_deviation >= 0. { "+" } else { "" },
                text::trim(*upper_deviation, 4),
                text::trim(*lower_deviation, 4),
                if note.is_empty() {
                    String::new()
                } else {
                    format!(" {note}")
                }
            );
            art.label(
                add(*position, [6., -1.5]),
                value,
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
        }
        WeldSymbol {
            attachment,
            position,
            weld_type,
            size,
            length,
            pitch,
            all_around,
            field_weld,
            tail,
            ..
        } => {
            let [a, b] = r.line(attachment)?;
            art.leader(midpoint(a, b), *position, style);
            art.line(
                *position,
                add(*position, [25., 0.]),
                &style.leader,
                Ink::Drawing,
            );
            weld(art, *position, *weld_type, &style.leader);
            if *all_around {
                art.circle(*position, 1.8, &style.leader, Ink::Drawing);
                art.disc(*position, 1.8, style.leader.width_mm);
            }
            if *field_weld {
                art.polyline(
                    &[
                        add(*position, [0., -1.]),
                        add(*position, [0., -7.]),
                        add(*position, [4., -5.]),
                        add(*position, [0., -3.]),
                    ],
                    &style.leader,
                    Ink::Drawing,
                );
            }
            let value = format!(
                "{}{}{}",
                text::trim(*size, 4),
                length.map_or(String::new(), |v| format!("-{}", text::trim(v, 4))),
                pitch.map_or(String::new(), |v| format!(" ({})", text::trim(v, 4)))
            );
            art.label(
                add(*position, [8., -1.5]),
                value,
                style.text_height_mm,
                1.,
                true,
                Ink::Drawing,
            );
            if !tail.is_empty() {
                art.label(
                    add(*position, [26., -1.5]),
                    tail.clone(),
                    style.text_height_mm,
                    1.,
                    true,
                    Ink::Drawing,
                );
            }
        }
        ItemBalloon {
            attachment,
            position,
            bom_item_id,
            ..
        } => {
            art.leader(r.attachment(attachment)?, *position, style);
            let center = add(*position, [4., -2.]);
            art.circle(center, 4., &style.leader, Ink::Drawing);
            art.disc(center, 4., style.leader.width_mm);
            art.label(
                add(*position, [4., -0.8]),
                sheet
                    .bom
                    .iter()
                    .find(|item| item.id == *bom_item_id)
                    .map_or(bom_item_id.to_string(), |item| item.item_number.clone()),
                style.text_height_mm,
                0.,
                false,
                Ink::Drawing,
            );
        }
    }
    Some(())
}

fn weld(art: &mut CheckedArt, position: P, kind: DrawingWeldType, style: &DrawingLineStyleDto) {
    use DrawingWeldType::*;
    let points: &[P] = match kind {
        Fillet => &[[3., 0.], [7., 0.], [7., -4.], [3., 0.]],
        SquareGroove => &[],
        VGroove => &[[3., -4.], [6., 0.], [9., -4.]],
        BevelGroove => &[[4., -4.], [4., 0.], [8., -4.]],
        PlugSlot => &[[3., -4.], [9., -4.], [9., 0.], [3., 0.], [3., -4.]],
        Seam => &[],
        Spot | UGroove | JGroove | Surfacing => &[],
    };
    if !points.is_empty() {
        art.polyline(
            &points.iter().map(|p| add(position, *p)).collect::<Vec<_>>(),
            style,
            Ink::Drawing,
        );
    }
    match kind {
        SquareGroove => {
            for x in [4., 7.] {
                art.line(
                    add(position, [x, -4.]),
                    add(position, [x, 4.]),
                    style,
                    Ink::Drawing,
                );
            }
        }
        Seam => {
            for y in [-2., -4.] {
                art.line(
                    add(position, [3., y]),
                    add(position, [9., y]),
                    style,
                    Ink::Drawing,
                );
            }
        }
        Spot => art.circle(add(position, [6., 0.]), 2., style, Ink::Drawing),
        UGroove | JGroove | Surfacing => {
            let curve = |a: P, b: P, c: P| {
                (0..=16)
                    .map(|i| {
                        let t = i as f64 / 16.;
                        add(
                            position,
                            add(
                                add(scale(a, (1. - t) * (1. - t)), scale(b, 2. * (1. - t) * t)),
                                scale(c, t * t),
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            if kind == Surfacing {
                art.polyline(&curve([3., -1.], [6., -6.], [9., -1.]), style, Ink::Drawing);
            } else if kind == UGroove {
                art.polyline(&curve([3., -4.], [3., 0.], [6., 0.]), style, Ink::Drawing);
                art.polyline(&curve([6., 0.], [9., 0.], [9., -4.]), style, Ink::Drawing);
            } else {
                art.line(
                    add(position, [3., -4.]),
                    add(position, [3., 0.]),
                    style,
                    Ink::Drawing,
                );
                art.polyline(&curve([3., 0.], [7., 0.], [7., -4.]), style, Ink::Drawing);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "drawing_annotations/budget_tests.rs"]
mod budget_tests;
#[cfg(test)]
#[path = "drawing_annotations/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "drawing_annotations/series_tests.rs"]
mod series_tests;
