use super::*;
use nbcad_occt::DrawingPolylineDto;
use serde_json::json;

fn sheet() -> DrawingSheetDto {
    serde_json::from_value(json!({"id":1,"name":"Dense sheet","format":"a4","orientation":"landscape",
        "views":[
            {"id":1,"name":"First","kind":"front","direction":[0.,-1.,0.],"up":[0.,0.,1.],"position":[25.,25.],"scale":1.,"show_hidden_lines":true},
            {"id":2,"name":"Later","kind":"front","direction":[0.,-1.,0.],"up":[0.,0.,1.],"position":[70.,50.],"scale":1.,"show_hidden_lines":true}
        ]})).unwrap()
}
pub(super) fn owner() -> DocumentContext {
    DocumentContext {
        window_id: "main".into(),
        document_id: "drawing-a".into(),
        epoch: 1,
    }
}
fn key() -> SourceKey {
    SourceKey::new(owner(), 19, 7, &sheet())
}
pub(super) fn raster() -> RasterKey {
    RasterKey {
        sheet_mm: [100., 80.],
        paper_scale: 1.,
        render_scale: 1.,
        visible_mm: [0., 0., 100., 80.],
    }
}
fn projection(dense: bool) -> DrawingProjectionDto {
    let mut points = if dense {
        (0..1024).map(|i| [i as f64 * 0.002, 0.]).collect()
    } else {
        vec![[0., 0.]]
    };
    points.push([30., 0.]);
    DrawingProjectionDto {
        topology_signatures: BTreeMap::from([("body".into(), "exact topology signature".into())]),
        visible: vec![DrawingPolylineDto { points }],
        hidden: vec![],
        anchors: vec![],
        circles: vec![],
        section: vec![],
        bounds: [0., 0., 30., 10.],
    }
}
fn pixel(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * image.texture_descriptor.size.width + x) * 4) as usize;
    image.data.as_ref().unwrap()[offset..offset + 4]
        .try_into()
        .unwrap()
}

#[test]
fn section_and_removed_section_stroke_cut_edges_with_react_layer_selection() {
    let anchor = json!({"body_id":1,"edge_id":1,"edge_key":"edge","endpoint":"start","fallback_point":[0.,0.,0.]});
    for (kind, ordinary, cut) in [
        ("section", true, true),
        ("removed_section", false, true),
        ("front", true, false),
    ] {
        let mut key = key();
        key.views.truncate(1);
        if kind != "front" {
            key.views[0].derivation = Some(
                serde_json::from_value(json!({
                    "type":kind,"parent_view_id":3,"first":anchor,"second":anchor,
                    "label":"A","hatch_angle_deg":45.,"hatch_spacing_mm":2.5
                }))
                .unwrap(),
            );
        }
        let mut data = projection(false);
        data.section.push(DrawingPolylineDto {
            points: vec![[0., 5.], [30., 5.], [30., 10.], [0., 10.], [0., 5.]],
        });
        let mut cache = EdgeCache::default();
        let mut images = Assets::<Image>::default();
        let ready = cache
            .prepare(&mut images, key, raster(), |_| Ok(data.clone()))
            .unwrap();
        let image = images.get(&ready.image).unwrap();
        assert_eq!(
            pixel(image, 25, 30)[3] > 0,
            ordinary,
            "ordinary layer {kind}"
        );
        assert_eq!(pixel(image, 25, 25)[3] > 0, cut, "section layer {kind}");
        assert_eq!(
            serde_json::to_value(&ready.projections[&1].1).unwrap(),
            serde_json::to_value(&data).unwrap(),
            "Visibility must not prune associations"
        );
    }
}

#[test]
fn dense_projection_retains_every_segment_and_all_later_view_associations() {
    let mut cache = EdgeCache::default();
    let mut images = Assets::<Image>::default();
    let ready = cache
        .prepare(&mut images, key(), raster(), |view| {
            Ok(projection(view.id == 1))
        })
        .unwrap();
    assert_eq!(ready.projections.len(), 2);
    assert_eq!(ready.projections[&1].1.visible[0].points.len(), 1025);
    assert_eq!(
        ready.projections[&2].1.topology_signatures["body"],
        "exact topology signature"
    );
    let image = images.get(&ready.image).unwrap();
    assert!(
        pixel(image, 38, 30)[3] > 0,
        "Final segment beyond the old 800 edge cap is missing"
    );
    assert!(
        pixel(image, 80, 55)[3] > 0,
        "A later view was silently skipped"
    );
}

#[test]
fn edge_cache_reuses_exact_source_and_repaints_at_changed_dpi_size_style_or_owner() {
    let mut cache = EdgeCache::default();
    let mut images = Assets::<Image>::default();
    let first = cache
        .prepare(&mut images, key(), raster(), |_| Ok(projection(false)))
        .unwrap()
        .image;
    let unchanged = cache
        .prepare(&mut images, key(), raster(), |_| {
            panic!("Unchanged view reprojected")
        })
        .unwrap();
    assert!(!unchanged.source_changed);
    assert_eq!(unchanged.image, first);
    let mut twice = raster();
    twice.render_scale = 2.;
    let scaled = cache
        .prepare(&mut images, key(), twice, |_| {
            panic!("DPI must not reproject OCCT")
        })
        .unwrap();
    assert!(!scaled.source_changed);
    assert_eq!(scaled.image, first);
    assert_eq!(
        images
            .get(&scaled.image)
            .unwrap()
            .texture_descriptor
            .size
            .width,
        200
    );
    assert!(pixel(images.get(&scaled.image).unwrap(), 160, 110)[3] > 0);
    // A close inspection at 423% and 2x DPI must retain exact polylines and
    // only rerasterize the visible crop, never call OCCT from navigation.
    for x in [10., 20.] {
        let zoom = RasterKey {
            paper_scale: 3. * 4.23,
            render_scale: 2.,
            visible_mm: [x, 10., 60., 40.],
            ..raster()
        };
        let ready = cache
            .prepare(&mut images, key(), zoom, |_| {
                panic!("Zoom/pan reprojected OCCT")
            })
            .unwrap();
        assert!(!ready.source_changed);
        assert_eq!(ready.image, first);
        assert_eq!(
            serde_json::to_value(&ready.projections[&1].1).unwrap(),
            serde_json::to_value(projection(false)).unwrap()
        );
        assert_eq!(images.len(), 1);
    }
    for next in [
        {
            let mut next = key();
            next.document_revision += 1;
            next
        },
        {
            let mut next = key();
            next.geometry_revision += 1;
            next
        },
        {
            let mut next = key();
            next.owner.epoch += 1;
            next
        },
        {
            let mut next = key();
            next.owner.document_id = "drawing-b".into();
            next
        },
        {
            let mut next = key();
            next.views[0].position[0] += 1.;
            next
        },
        {
            let mut next = key();
            next.visible.width_mm *= 2.;
            next
        },
    ] {
        let mut calls = 0;
        assert!(
            cache
                .prepare(&mut images, next, raster(), |_| {
                    calls += 1;
                    Ok(projection(false))
                })
                .unwrap()
                .source_changed
        );
        assert_eq!(calls, 2);
        assert_eq!(
            images.len(),
            1,
            "Resizing/editing must reuse the retained image allocation"
        );
    }
}

#[test]
fn failed_or_oversized_render_is_explicit_atomic_and_not_retried_without_a_change() {
    let mut cache = EdgeCache::default();
    let mut images = Assets::<Image>::default();
    let first = cache
        .prepare(&mut images, key(), raster(), |_| Ok(projection(false)))
        .unwrap()
        .image;
    let original = images.get(&first).unwrap().data.clone();
    let mut changed = key();
    changed.owner.epoch += 1;
    let mut calls = 0;
    let error = cache
        .prepare_with_limits(
            &mut images,
            changed.clone(),
            raster(),
            |_| {
                calls += 1;
                Ok(projection(true))
            },
            Limits {
                points: 100,
                ..Default::default()
            },
        )
        .err()
        .unwrap();
    assert!(error.contains("retained geometry budget"));
    assert_eq!(calls, 1);
    assert!(cache
        .prepare(&mut images, changed, raster(), |_| panic!(
            "Same failed source retried"
        ))
        .is_err());
    assert_eq!(images.get(&first).unwrap().data, original);
    assert_eq!(cache.source.as_ref().unwrap().key.owner, owner());
    let huge = RasterKey {
        paper_scale: 1000.,
        ..raster()
    };
    assert!(cache
        .prepare(&mut images, key(), huge, |_| panic!(
            "Oversized image must reject before OCCT"
        ))
        .err()
        .unwrap()
        .contains("physical pixels"));
    assert_eq!(images.get(&first).unwrap().data, original);
    let bad = cache
        .prepare(
            &mut images,
            {
                let mut k = key();
                k.geometry_revision += 1;
                k
            },
            raster(),
            |_| Err("HLR failed".into()),
        )
        .err()
        .unwrap();
    assert!(bad.contains("First") && bad.contains("HLR failed"));
}

#[test]
fn hidden_dash_phase_runs_across_tessellated_segments_and_odd_patterns_repeat() {
    let mut k = key();
    k.views.truncate(1);
    k.views[0].position = [50., 30.];
    for dash in [vec![4., 2.], vec![2.]] {
        k.hidden.dash_mm = dash.clone();
        let source = Source::project(
            k.clone(),
            |_| {
                Ok(DrawingProjectionDto {
                    visible: vec![],
                    hidden: vec![DrawingPolylineDto {
                        points: (0..=100).map(|x| [x as f64, 0.]).collect(),
                    }],
                    bounds: [0., 0., 100., 10.],
                    ..projection(false)
                })
            },
            |_, _| Ok(vec![]),
            Limits::default(),
        )
        .unwrap();
        let region = raster().region(&source.key, Limits::default()).unwrap();
        let image = source.rasterize(raster(), region).unwrap();
        assert!(pixel(&image, 1, 35)[3] > 0);
        let gap = if dash.len() == 2 { 5 } else { 3 };
        assert_eq!(
            pixel(&image, gap, 35)[3],
            0,
            "Dash phase restarted at every tessellation point"
        );
    }
}

#[test]
fn subpixel_widths_remain_visible_at_fractional_positions_and_both_dpi_scales() {
    let mut k = key();
    k.views.truncate(1);
    k.views[0].position = [25.25, 25.25];
    k.visible.width_mm = 0.05;
    let source = Source::project(
        k,
        |_| Ok(projection(false)),
        |_, _| Ok(vec![]),
        Limits::default(),
    )
    .unwrap();
    for dpi in [1., 2.] {
        let key = RasterKey {
            render_scale: dpi,
            ..raster()
        };
        let region = key.region(&source.key, Limits::default()).unwrap();
        let image = source.rasterize(key, region).unwrap();
        let x = (38. * dpi) as u32;
        assert!(((29. * dpi) as u32..=(32. * dpi) as u32).any(|y| pixel(&image, x, y)[3] > 0));
    }
}

#[test]
fn maximum_zoom_rasterizes_only_the_visible_region_without_losing_late_edges_or_projections() {
    let mut cache = EdgeCache::default();
    let mut images = Assets::<Image>::default();
    let raster = RasterKey {
        sheet_mm: [1000., 800.],
        paper_scale: 15.,
        render_scale: 2.,
        visible_mm: [35., 28., 12., 4.],
    };
    // The full sheet would be 30000x24000; the viewport is only 360x120.
    let first = cache
        .prepare(&mut images, key(), raster, |v| Ok(projection(v.id == 1)))
        .unwrap();
    assert_eq!(first.projections.len(), 2);
    assert_eq!(first.projections[&1].1.visible[0].points.len(), 1025);
    let handle = first.image.clone();
    let region = first.region;
    let image = images.get(&handle).unwrap();
    // Default 0.5mm visible stroke: 15px wide, 32px guard on each side.
    assert_eq!(image.texture_descriptor.size.width, 360 + 64);
    assert_eq!(image.texture_descriptor.size.height, 120 + 64);
    let factor = 30.;
    let point = [38., 30.];
    let local = [
        ((point[0] - region.origin_mm[0]) * factor) as u32,
        ((point[1] - region.origin_mm[1]) * factor) as u32,
    ];
    assert!(
        pixel(image, local[0], local[1])[3] > 0,
        "Last dense-polyline segment was lost after ROI clipping"
    );
    let pan = RasterKey {
        visible_mm: [75., 53., 12., 4.],
        ..raster
    };
    let second = cache
        .prepare(&mut images, key(), pan, |_| {
            panic!("Pan must not reproject the solid")
        })
        .unwrap();
    assert!(!second.source_changed);
    assert_eq!(second.image, handle);
    assert_eq!(
        second.projections[&2].1.topology_signatures["body"],
        "exact topology signature"
    );
    let local = [
        ((80. - second.region.origin_mm[0]) * factor) as u32,
        ((55. - second.region.origin_mm[1]) * factor) as u32,
    ];
    assert!(pixel(images.get(&second.image).unwrap(), local[0], local[1])[3] > 0);
    assert_eq!(images.len(), 1);
}

#[test]
fn region_clips_to_paper_and_uses_exact_scale_instead_of_stretching_fractional_pixel_dimensions() {
    let mut k = key();
    k.views.truncate(1);
    let raster = RasterKey {
        paper_scale: 1.37,
        render_scale: 2.,
        visible_mm: [-100., -20., 1000., 1000.],
        ..raster()
    };
    let region = raster.region(&k, Limits::default()).unwrap();
    assert_eq!(region.origin_mm, [0.; 2]);
    let factor = f64::from(raster.paper_scale) * f64::from(raster.render_scale);
    for i in 0..2 {
        assert!((region.size_mm[i] * factor - f64::from(region.dimensions[i])).abs() < 1e-9);
        assert!(region.size_mm[i] >= f64::from(raster.sheet_mm[i]));
        assert!(region.size_mm[i] - f64::from(raster.sheet_mm[i]) < factor.recip());
    }
    for bad in [
        [1000., 0., 10., 10.],
        [0., 0., 0., 10.],
        [f64::NAN, 0., 10., 10.],
    ] {
        assert!(RasterKey {
            visible_mm: bad,
            ..raster
        }
        .region(&k, Limits::default())
        .is_err());
    }
}

#[test]
fn panning_a_crop_preserves_hidden_dash_phase_from_the_complete_polyline() {
    let mut k = key();
    k.views.truncate(1);
    k.views[0].position = [50., 30.];
    let source = Source::project(
        k,
        |_| {
            Ok(DrawingProjectionDto {
                visible: vec![],
                hidden: vec![DrawingPolylineDto {
                    points: (0..=100).map(|x| [x as f64, 0.]).collect(),
                }],
                bounds: [0., 0., 100., 10.],
                ..projection(false)
            })
        },
        |_, _| Ok(vec![]),
        Limits::default(),
    )
    .unwrap();
    let full_key = raster();
    let full = source
        .rasterize(
            full_key,
            full_key.region(&source.key, Limits::default()).unwrap(),
        )
        .unwrap();
    let cropped_key = RasterKey {
        visible_mm: [9., 34., 12., 2.],
        ..full_key
    };
    let region = cropped_key.region(&source.key, Limits::default()).unwrap();
    let cropped = source.rasterize(cropped_key, region).unwrap();
    for x in 9..21 {
        for y in 34..36 {
            assert_eq!(
                pixel(&full, x, y),
                pixel(
                    &cropped,
                    x - region.origin_mm[0] as u32,
                    y - region.origin_mm[1] as u32
                ),
                "Cropping changed the retained polyline dash at ({x}, {y})"
            );
        }
    }
}

#[test]
fn resolved_projection_basis_is_cached_by_exact_owner_revision_without_rewriting_view_intent() {
    let mut cache = EdgeCache::default();
    let mut images = Assets::<Image>::default();
    let key = key();
    let saved = serde_json::to_value(&key.views).unwrap();
    let mut projected = 0;
    let basis = nbcad_occt::drawing_projection_basis([0., 0., 1.], [0., 1., 0.]).unwrap();
    cache
        .prepare_sheet(
            &mut images,
            key.clone(),
            raster(),
            |_| {
                projected += 1;
                Ok(ResolvedDrawingProjection {
                    projection: projection(false),
                    basis,
                })
            },
            |_, _| Ok(vec![]),
        )
        .unwrap();
    assert_eq!(projected, key.views.len());
    assert_eq!(cache.bases(&key).unwrap().len(), key.views.len());
    for (id, (view, _)) in cache.projections(&key).unwrap() {
        assert_eq!(cache.bases(&key).unwrap()[id], basis);
        assert_eq!(view.direction, [0., -1., 0.]);
    }
    assert_eq!(serde_json::to_value(&key.views).unwrap(), saved);
    cache
        .prepare_sheet(
            &mut images,
            key.clone(),
            RasterKey {
                render_scale: 2.,
                ..raster()
            },
            |_| panic!("DPI re-ran projection"),
            |_, _| panic!("DPI rebuilt source graphics"),
        )
        .unwrap();
    for variant in 0..3 {
        let mut changed = key.clone();
        match variant {
            0 => changed.document_revision += 1,
            1 => changed.owner.epoch += 1,
            _ => changed.owner.document_id = "other".into(),
        };
        assert!(cache.projections(&changed).is_none());
        assert!(cache.bases(&changed).is_none());
        assert!(cache
            .prepare_sheet(
                &mut images,
                changed.clone(),
                raster(),
                |_| Err("new owner projection failed".into()),
                |_, _| Ok(vec![])
            )
            .is_err());
        assert!(
            cache.bases(&changed).is_none(),
            "Failed request exposed previous owner basis"
        );
    }
    let mut fresh = key.clone();
    fresh.document_revision += 4;
    let flipped = nbcad_occt::drawing_projection_basis([0., 0., -1.], [0., 1., 0.]).unwrap();
    cache
        .prepare_sheet(
            &mut images,
            fresh.clone(),
            raster(),
            |_| {
                Ok(ResolvedDrawingProjection {
                    projection: projection(false),
                    basis: flipped,
                })
            },
            |_, _| Ok(vec![]),
        )
        .unwrap();
    assert!(cache.bases(&key).is_none());
    assert_eq!(cache.bases(&fresh).unwrap()[&1], flipped);
}
