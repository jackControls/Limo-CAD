//! Batched projected edges for drawing paper. OCCT projections remain complete
//! for associative annotations; one retained physical-resolution image replaces
//! the old per-segment UI entities.
use super::{paper_point, raster_stroke_width};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use nbcad_interface::DocumentContext;
use nbcad_occt::DrawingProjectionDto;
use nbcad_sketch::{
    DrawingLineStyleDto, DrawingSheetDto, DrawingViewDerivationDto, DrawingViewDto,
};
use resvg::tiny_skia::{
    LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform,
};
use std::collections::BTreeMap;

pub(super) type Projections = BTreeMap<u64, (DrawingViewDto, DrawingProjectionDto)>;

#[derive(Clone, PartialEq)]
pub(super) struct SourceKey {
    owner: DocumentContext,
    document_revision: u64,
    geometry_revision: u64,
    sheet_id: u64,
    views: Vec<DrawingViewDto>,
    visible: DrawingLineStyleDto,
    hidden: DrawingLineStyleDto,
}
impl SourceKey {
    pub(super) fn new(
        owner: DocumentContext,
        document_revision: u64,
        geometry_revision: u64,
        sheet: &DrawingSheetDto,
    ) -> Self {
        Self {
            owner,
            document_revision,
            geometry_revision,
            sheet_id: sheet.id,
            views: sheet.views.clone(),
            visible: sheet.style.visible.clone(),
            hidden: sheet.style.hidden.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RasterKey {
    pub sheet_mm: [f32; 2],
    pub paper_scale: f32,
    pub render_scale: f32,
    /// Visible paper x, y, width, height in millimetres, before stroke guard.
    /// Derived from the same inverse paper transform used for picking.
    pub visible_mm: [f64; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RasterRegion {
    pub origin_mm: [f64; 2],
    pub size_mm: [f64; 2],
    dimensions: [u32; 2],
}

#[derive(Clone, Copy)]
struct Limits {
    points: usize,
    stroke_steps: f64,
    pixels: u64,
    dimension: u32,
    metadata_bytes: usize,
    retained_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            points: 2_000_000,
            stroke_steps: 4_000_000.,
            pixels: 16_777_216,
            dimension: 8192,
            metadata_bytes: 32 * 1024 * 1024,
            retained_bytes: 128 * 1024 * 1024,
        }
    }
}

struct Source {
    key: SourceKey,
    projections: Projections,
}

#[derive(Resource, Default)]
pub(super) struct EdgeCache {
    source: Option<Source>,
    raster: Option<(RasterKey, RasterRegion, Handle<Image>)>,
    failure: Option<(SourceKey, RasterKey, String)>,
}

pub(super) struct Prepared<'a> {
    pub image: Handle<Image>,
    /// Place this image inside the paper parent at this exact paper rectangle.
    /// It is already at native physical resolution; never stretch to a sheet.
    pub region: RasterRegion,
    pub projections: &'a Projections,
    pub source_changed: bool,
}

impl EdgeCache {
    /// All source and raster work succeeds before replacing retained assets.
    /// Callers must hide the edge image/annotations on Err and show its message;
    /// an old document or partial sheet is never returned as a fallback.
    pub(super) fn prepare<'a>(
        &'a mut self,
        images: &mut Assets<Image>,
        key: SourceKey,
        raster: RasterKey,
        project: impl FnMut(&DrawingViewDto) -> Result<DrawingProjectionDto, String>,
    ) -> Result<Prepared<'a>, String> {
        self.prepare_with_limits(images, key, raster, project, Limits::default())
    }
    fn prepare_with_limits<'a>(
        &'a mut self,
        images: &mut Assets<Image>,
        key: SourceKey,
        raster: RasterKey,
        project: impl FnMut(&DrawingViewDto) -> Result<DrawingProjectionDto, String>,
        limits: Limits,
    ) -> Result<Prepared<'a>, String> {
        if let Some((failed_key, failed_raster, error)) = &self.failure {
            if failed_key == &key && *failed_raster == raster {
                return Err(error.clone());
            }
        }
        let source_changed = self.source.as_ref().is_none_or(|source| source.key != key);
        let raster_changed = source_changed
            || self
                .raster
                .as_ref()
                .is_none_or(|(saved, _, handle)| *saved != raster || !images.contains(handle.id()));
        if raster_changed {
            let result = (|| {
                // Check physical memory first, before any expensive projection.
                let region = raster.region(&key, limits)?;
                let next = if source_changed {
                    Some(Source::project(key.clone(), project, limits)?)
                } else {
                    None
                };
                let source = next
                    .as_ref()
                    .or(self.source.as_ref())
                    .ok_or("Drawing edge source is missing")?;
                let image = source.rasterize(raster, region)?;
                Ok::<_, String>((next, image, region))
            })();
            let (next, image, region) = match result {
                Ok(value) => value,
                Err(error) => {
                    self.failure = Some((key, raster, error.clone()));
                    return Err(error);
                }
            };
            let handle = if let Some((_, _, handle)) = &self.raster {
                if images.contains(handle.id()) {
                    *images.get_mut(handle).unwrap() = image;
                    handle.clone()
                } else {
                    images.add(image)
                }
            } else {
                images.add(image)
            };
            if let Some(source) = next {
                self.source = Some(source);
            }
            self.raster = Some((raster, region, handle));
            self.failure = None;
        }
        Ok(Prepared {
            image: self.raster.as_ref().unwrap().2.clone(),
            region: self.raster.as_ref().unwrap().1,
            projections: &self.source.as_ref().unwrap().projections,
            source_changed,
        })
    }
}

impl RasterKey {
    fn region(self, source: &SourceKey, limits: Limits) -> Result<RasterRegion, String> {
        if self
            .sheet_mm
            .into_iter()
            .chain([self.paper_scale, self.render_scale])
            .any(|v| !v.is_finite() || v <= 0.)
        {
            return Err("Drawing paper size or DPI is invalid".into());
        }
        if self.visible_mm.iter().any(|n| !n.is_finite())
            || self.visible_mm[2..].iter().any(|n| *n <= 0.)
        {
            return Err("Drawing visible paper region is invalid".into());
        }
        let scale = f64::from(self.paper_scale) * f64::from(self.render_scale);
        let mut stroke = 0f64;
        for style in [&source.visible, &source.hidden] {
            let width =
                raster_stroke_width(style.width_mm as f32, self.paper_scale, self.render_scale)
                    * self.render_scale;
            if !style.width_mm.is_finite()
                || style.width_mm <= 0.
                || !width.is_finite()
                || width <= 0.
            {
                return Err("Drawing edge width cannot be rendered at this scale".into());
            }
            stroke = stroke.max(f64::from(width));
        }
        // tiny-skia's default miter limit is four. Keep its full extension and
        // two pixels for antialias/filter coverage outside the visible pane.
        let guard = stroke * 2. + 2.;
        let mut start = [0.; 2];
        let mut end = [0.; 2];
        for i in 0..2 {
            let min = self.visible_mm[i].max(0.);
            let max =
                (self.visible_mm[i] + self.visible_mm[i + 2]).min(f64::from(self.sheet_mm[i]));
            if !max.is_finite() || max <= min {
                return Err("Drawing visible region does not intersect its paper".into());
            }
            start[i] = (min * scale - guard).floor().max(0.);
            end[i] = (max * scale + guard)
                .ceil()
                .min((f64::from(self.sheet_mm[i]) * scale).ceil());
        }
        let physical = [end[0] - start[0], end[1] - start[1]];
        if physical
            .iter()
            .any(|v| !v.is_finite() || *v > limits.dimension as f64)
        {
            return Err(format!(
                "Drawing visible region and stroke guard exceed {} physical pixels on one axis; resize the window or reduce its display scale",
                limits.dimension
            ));
        }
        let [width, height] = physical.map(|v| v.ceil().max(1.) as u32);
        if u64::from(width) * u64::from(height) > limits.pixels {
            return Err(format!(
                "Drawing visible region and stroke guard exceed the {} pixel rendering budget; resize the window or reduce its display scale",
                limits.pixels
            ));
        }
        Ok(RasterRegion {
            origin_mm: start.map(|n| n / scale),
            size_mm: [f64::from(width) / scale, f64::from(height) / scale],
            dimensions: [width, height],
        })
    }
}

impl Source {
    fn project(
        key: SourceKey,
        mut project: impl FnMut(&DrawingViewDto) -> Result<DrawingProjectionDto, String>,
        limits: Limits,
    ) -> Result<Self, String> {
        let mut projections = Projections::new();
        let mut points = 0usize;
        let mut metadata = 0usize;
        let mut retained = 0usize;
        let mut steps = 0.;
        for view in &key.views {
            if !view.scale.is_finite()
                || view.scale <= 0.
                || view.position.iter().any(|v| !v.is_finite())
            {
                return Err(format!(
                    "Drawing view '{}' has invalid paper placement or scale",
                    view.name
                ));
            }
            let projection = project(view)
                .map_err(|error| format!("Cannot project drawing view '{}': {error}", view.name))?;
            if projection.bounds.iter().any(|v| !v.is_finite()) {
                return Err(format!(
                    "Drawing view '{}' has non-finite projection bounds",
                    view.name
                ));
            }
            // Retain all curves and every associative anchor; visibility only
            // selects which paths are stroked, never what source is retained.
            let count = projection
                .visible
                .iter()
                .chain(&projection.hidden)
                .chain(&projection.section)
                .map(|line| line.points.len())
                .sum::<usize>();
            points = points
                .saturating_add(count)
                .saturating_add(projection.anchors.len())
                .saturating_add(projection.circles.len());
            metadata = metadata.saturating_add(
                projection
                    .topology_signatures
                    .iter()
                    .map(|(a, b)| a.len().saturating_add(b.len()))
                    .sum::<usize>(),
            );
            metadata = metadata.saturating_add(
                projection
                    .anchors
                    .iter()
                    .map(|a| a.edge_key.len())
                    .sum::<usize>(),
            );
            metadata = metadata.saturating_add(
                projection
                    .circles
                    .iter()
                    .map(|a| a.edge_key.len())
                    .sum::<usize>(),
            );
            // Include vector capacities and fixed record storage, not just
            // coordinate counts or string lengths. The map estimate includes
            // owned key/value strings and conservative per-entry node overhead.
            for lines in [&projection.visible, &projection.hidden, &projection.section] {
                retained = retained.saturating_add(
                    lines.capacity() * std::mem::size_of::<nbcad_occt::DrawingPolylineDto>(),
                );
                for line in lines {
                    retained = retained
                        .saturating_add(line.points.capacity() * std::mem::size_of::<[f64; 2]>());
                }
            }
            retained = retained.saturating_add(
                projection.anchors.capacity()
                    * std::mem::size_of::<nbcad_occt::DrawingProjectionAnchorDto>(),
            );
            retained = retained.saturating_add(
                projection.circles.capacity()
                    * std::mem::size_of::<nbcad_occt::DrawingProjectedCircleDto>(),
            );
            retained = retained.saturating_add(
                projection
                    .topology_signatures
                    .iter()
                    .map(|(a, b)| {
                        128usize
                            .saturating_add(a.capacity())
                            .saturating_add(b.capacity())
                    })
                    .sum::<usize>(),
            );
            retained = retained.saturating_add(
                projection
                    .anchors
                    .iter()
                    .map(|a| a.edge_key.capacity())
                    .sum::<usize>(),
            );
            retained = retained.saturating_add(
                projection
                    .circles
                    .iter()
                    .map(|a| a.edge_key.capacity())
                    .sum::<usize>(),
            );
            if points > limits.points
                || metadata > limits.metadata_bytes
                || retained > limits.retained_bytes
            {
                return Err(format!(
                    "Drawing projection exceeds the retained geometry budget ({} points, {} metadata bytes, {} retained bytes); simplify the sheet's views",
                    limits.points, limits.metadata_bytes, limits.retained_bytes
                ));
            }
            for (lines, style, _) in paths(view, &projection, &key) {
                let interval = style.dash_mm.iter().copied().reduce(f64::min);
                if !style.width_mm.is_finite()
                    || style.width_mm <= 0.
                    || style.dash_mm.iter().any(|n| !n.is_finite() || *n <= 0.)
                {
                    return Err("Drawing edge style has invalid stroke or dash lengths".into());
                }
                for line in lines {
                    if line.points.iter().flatten().any(|v| !v.is_finite()) {
                        return Err(format!(
                            "Drawing view '{}' has non-finite projected edges",
                            view.name
                        ));
                    }
                    for pair in line.points.windows(2) {
                        let distance =
                            (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]) * view.scale;
                        steps += 1. + interval.map_or(0., |dash| (distance / dash).ceil());
                        if !steps.is_finite() || steps > limits.stroke_steps {
                            return Err(format!(
                                "Drawing edge detail exceeds the {} stroke-step rendering budget; simplify the sheet's views or dash pattern",
                                limits.stroke_steps
                            ));
                        }
                    }
                }
            }
            if projections
                .insert(view.id, (view.clone(), projection))
                .is_some()
            {
                return Err("Drawing sheet contains duplicate view identities".into());
            }
        }
        Ok(Self { key, projections })
    }
    fn rasterize(&self, key: RasterKey, region: RasterRegion) -> Result<Image, String> {
        let [width, height] = region.dimensions;
        let mut pixmap =
            Pixmap::new(width, height).ok_or("Unable to allocate drawing paper image")?;
        // One isotropic physical scale, independent of integer image rounding.
        // This keeps image strokes aligned with vector labels and paper picks.
        let factor = f64::from(key.paper_scale) * f64::from(key.render_scale);
        for (view, projection) in self.projections.values() {
            for (lines, style, hidden) in paths(view, projection, &self.key) {
                let mut builder = PathBuilder::new();
                let mut paths = 0;
                for line in lines {
                    if line.points.len() < 2 {
                        continue;
                    }
                    for (index, point) in line.points.iter().enumerate() {
                        let paper = paper_point(view, *point, projection);
                        let [x, y] = [
                            (paper[0] - region.origin_mm[0]) * factor,
                            (paper[1] - region.origin_mm[1]) * factor,
                        ]
                        .map(|v| v as f32);
                        if !x.is_finite() || !y.is_finite() {
                            return Err(format!(
                                "Drawing view '{}' lies outside finite render coordinates",
                                view.name
                            ));
                        }
                        if index == 0 {
                            builder.move_to(x, y);
                        } else {
                            builder.line_to(x, y);
                        }
                    }
                    paths += 1;
                }
                if paths == 0 {
                    continue;
                }
                let path = builder
                    .finish()
                    .ok_or("Unable to build drawing edge paths")?;
                let mut paint = Paint::default();
                if hidden {
                    paint.set_color_rgba8(132, 138, 146, 255);
                } else {
                    paint.set_color_rgba8(36, 40, 45, 255);
                }
                paint.anti_alias = true;
                let mut stroke = Stroke {
                    line_cap: LineCap::Round,
                    line_join: LineJoin::Round,
                    width: raster_stroke_width(
                        style.width_mm as f32,
                        key.paper_scale,
                        key.render_scale,
                    ) * key.render_scale,
                    ..Default::default()
                };
                if !style.dash_mm.is_empty() {
                    let mut pattern = style
                        .dash_mm
                        .iter()
                        .map(|v| (*v * factor) as f32)
                        .collect::<Vec<_>>();
                    // SVG repeats an odd dash list, while tiny-skia requires an even one.
                    if pattern.len() % 2 != 0 {
                        pattern.extend_from_within(..);
                    }
                    stroke.dash = Some(
                        StrokeDash::new(pattern, 0.)
                            .ok_or("Drawing dash pattern cannot be rendered at this scale")?,
                    );
                }
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        // tiny-skia stores premultiplied RGBA; ImageNode expects straight RGBA.
        // Convert in place to keep the peak raster allocation bounded.
        for pixel in pixmap.data_mut().chunks_exact_mut(4) {
            let alpha = u32::from(pixel[3]);
            if alpha != 0 && alpha != 255 {
                for channel in &mut pixel[..3] {
                    *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
                }
            }
        }
        Ok(Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixmap.take(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    }
}

/// Same edge-layer selection as DrawingWorkspace's ViewGraphic. Section and
/// removed-section derivations render cut loops; removed sections omit the
/// ordinary projection. The complete projection remains available to anchors.
fn paths<'a>(
    view: &DrawingViewDto,
    projection: &'a DrawingProjectionDto,
    key: &'a SourceKey,
) -> impl Iterator<
    Item = (
        &'a Vec<nbcad_occt::DrawingPolylineDto>,
        &'a DrawingLineStyleDto,
        bool,
    ),
> {
    let removed = matches!(
        view.derivation,
        Some(DrawingViewDerivationDto::RemovedSection { .. })
    );
    let section = removed
        || matches!(
            view.derivation,
            Some(DrawingViewDerivationDto::Section { .. })
        );
    [
        (&projection.visible, &key.visible, false, !removed),
        (
            &projection.hidden,
            &key.hidden,
            true,
            !removed && view.show_hidden_lines,
        ),
        (&projection.section, &key.visible, false, section),
    ]
    .into_iter()
    .filter(|(_, _, _, enabled)| *enabled)
    .map(|(lines, style, hidden, _)| (lines, style, hidden))
}

#[cfg(test)]
#[path = "drawing_edges/tests.rs"]
mod tests;
