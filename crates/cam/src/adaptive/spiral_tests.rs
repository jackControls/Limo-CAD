#[test]
fn circular_roughing_is_continuous_with_full_retract_and_keep_down_off() {
    let center = Point2Dto::new(8.0, 7.0);
    let mut doc = with_linking(fixture(vec![cylinder(center, 2.5, -3.0, 0.0)]));
    doc.setups[0].stock_spec = CamStockSpecDto::FromModel {
        shape: CamStockShape::Cylinder,
        offsets: CamStockOffsetsDto::default(),
    };
    doc.setups[0].resolved_stock = CamResolvedStockDto::Cylinder {
        center,
        radius: 7.0,
    };
    doc.linking[0].keep_tool_down = false;
    doc.linking[0].maximum_stay_down = 0.01;
    doc.linking[0].retraction_policy = crate::CamRetractionPolicy::Full;
    for angle in [0.0_f64, 0.73] {
        let mut doc = doc.clone();
        let (sin, cos) = angle.sin_cos();
        let origin = Point3Dto::new(13.0, -8.0, 4.0);
        doc.setups[0].wcs.origin = origin;
        doc.setups[0].wcs.y_axis = [0.0, cos, sin];
        doc.setups[0].wcs.z_axis = [0.0, -sin, cos];
        let CamOperationDto::Adaptive3d {
            geometry: Some(geometry),
            ..
        } = &mut doc.setups[0].operations[0]
        else {
            unreachable!()
        };
        for mesh in &mut geometry.targets {
            for v in mesh.positions.chunks_exact_mut(3) {
                let [x, y, z] = [v[0], v[1], v[2]];
                v.copy_from_slice(&[
                    origin.x + x,
                    origin.y + cos * y - sin * z,
                    origin.z + sin * y + cos * z,
                ]);
            }
        }
        for kind in [
            CamToolKind::FlatEndMill,
            CamToolKind::BullNoseEndMill,
            CamToolKind::FaceMill,
        ] {
            doc.tools[0].kind = kind;
            doc.tools[0].corner_radius = (kind != CamToolKind::FlatEndMill).then_some(0.4);
            doc.tools[0].maximum_axial_depth = (kind == CamToolKind::FaceMill).then_some(1.0);
            let program = plan_setup(&doc, 1).unwrap();
            assert!(
                program.stats.rapid_distance < 100.0,
                "{kind:?}: {:?}",
                program.stats
            );
            for z in [-1.0, -2.0] {
                let arcs: Vec<_> = program
                    .commands
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| match c {
                        CamCommandDto::Circular {
                            to,
                            feed,
                            clockwise: true,
                            ..
                        } if (to.z - z).abs() < EPS && (*feed - 600.0).abs() < EPS => Some(i),
                        _ => None,
                    })
                    .collect();
                assert!(
                    arcs.len() >= 8,
                    "must exercise multiple connected revolutions"
                );
                assert!(
                    program.commands[arcs[0]..=*arcs.last().unwrap()]
                        .iter()
                        .all(|c| matches!(c, CamCommandDto::Circular { .. })),
                    "no exit, retract or separate entry inside the cutting pass"
                );
            }
            assert_adaptive_nc_roundtrip(doc.clone());
        }
    }
}

#[derive(Clone, Copy)]
struct AuditedSpiralArc {
    from: Point2Dto,
    to: Point2Dto,
    center: Point2Dto,
}
impl AuditedSpiralArc {
    fn radius(self) -> f64 {
        dist(self.from, self.center)
    }
    fn distance(self, p: Point2Dto) -> f64 {
        let a = (self.from.y - self.center.y).atan2(self.from.x - self.center.x);
        let b = (self.to.y - self.center.y).atan2(self.to.x - self.center.x);
        let t = (p.y - self.center.y).atan2(p.x - self.center.x);
        if (a - t).rem_euclid(TAU) <= (a - b).rem_euclid(TAU) + 1e-9 {
            (dist(p, self.center) - self.radius()).abs()
        } else {
            dist(p, self.from).min(dist(p, self.to))
        }
    }
}

#[test]
fn spiral_sweeps_preserve_target_cover_stock_and_bound_section_engagement() {
    // Independent contact audit: original billet minus completed swept arcs,
    // not the planner's remaining-stock certificate. Sample the advancing
    // half of each cutter section at stations throughout every half-circle.
    let doc = fixture(vec![]);
    let CamOperationDto::Adaptive3d { parameters, .. } = &doc.setups[0].operations[0] else {
        unreachable!()
    };
    for (floor, ae, protected, angle) in [
        (2.0, 1.0, 2.6, 0.0),
        (1.6, 0.5, 2.6, 0.7),
        (1.6, 4.0, 2.6, 2.4),
        (2.0, 1.0, 6.6, 0.4),
        (2.0, 1.0, 0.01, 1.3),
    ] {
        let mut p = parameters.clone();
        p.optimal_load = ae;
        let mut b = ProgramBuilder::new();
        b.clearance_z = 5.0;
        b.retract_z = 3.0;
        b.feed_height_z = 1.0;
        b.linking = Some(crate::CamLinkingDto {
            entry_positions: vec![polar(Point2Dto::new(0.0, 0.0), 20.0, angle)],
            ..Default::default()
        });
        let footprint: Vec<_> = (0..128)
            .map(|i| polar(Point2Dto::new(0.0, 0.0), 7.0, TAU * i as f64 / 128.0))
            .collect();
        spiral::clear(
            &mut b,
            &footprint,
            Point2Dto::new(0.0, 0.0),
            protected,
            2.0,
            floor,
            -1.0,
            &p,
            600.0,
            100.0,
            &mut Work::default(),
        )
        .unwrap();
        let mut arcs = vec![];
        let mut position = None;
        for command in &b.commands {
            match command {
                CamCommandDto::Circular {
                    to,
                    center,
                    feed,
                    clockwise,
                    ..
                } => {
                    if (*feed - 600.0).abs() < EPS {
                        assert!(*clockwise);
                        arcs.push(AuditedSpiralArc {
                            from: position.unwrap(),
                            to: Point2Dto::new(to.x, to.y),
                            center: Point2Dto::new(center.x, center.y),
                        });
                    }
                    position = Some(Point2Dto::new(to.x, to.y));
                }
                CamCommandDto::Linear { to, .. } | CamCommandDto::Rapid { to } => {
                    position = Some(Point2Dto::new(to.x, to.y))
                }
                _ => {}
            }
        }
        for pair in arcs.windows(2) {
            assert!(dist(pair[0].to, pair[1].from) < EPS);
            let u = Point2Dto::new(
                (pair[0].to.x - pair[0].center.x) / pair[0].radius(),
                (pair[0].to.y - pair[0].center.y) / pair[0].radius(),
            );
            let v = Point2Dto::new(
                (pair[1].from.x - pair[1].center.x) / pair[1].radius(),
                (pair[1].from.y - pair[1].center.y) / pair[1].radius(),
            );
            assert!(u.x * v.x + u.y * v.y > 1.0 - 1e-9, "C1 tangent join");
        }
        for s in [floor, (floor + 2.0) / 2.0, 2.0] {
            for (i, arc) in arcs.iter().enumerate() {
                let start = (arc.from.y - arc.center.y).atan2(arc.from.x - arc.center.x);
                for station in 0..=12 {
                    let a = start - PI * station as f64 / 12.0;
                    let c = polar(arc.center, arc.radius(), a);
                    assert!(
                        dist(c, Point2Dto::new(0.0, 0.0)) - 2.0 >= protected - 1e-7,
                        "target clearance"
                    );
                    let mut contact = 0;
                    const N: usize = 360;
                    for k in 0..N {
                        let theta = a - PI + (k as f64 + 0.5) * PI / N as f64;
                        let point = polar(c, s, theta);
                        if dist(point, Point2Dto::new(0.0, 0.0)) <= 7.0
                            && arcs[..i]
                                .iter()
                                .all(|prior| prior.distance(point) >= s - 1e-7)
                        {
                            contact += 1;
                        }
                    }
                    let measured = contact as f64 * PI / N as f64;
                    assert!(
                        measured <= (1.0 - ae / 2.0).acos() + 2.0 * PI / N as f64,
                        "section {s}, Ae {ae}, arc {i}, station {station}: {measured}"
                    );
                }
            }
            let residual = protected + 2.0 - s;
            for x in -30..=30 {
                for y in -30..=30 {
                    let point = Point2Dto::new(x as f64 * 7.0 / 30.0, y as f64 * 7.0 / 30.0);
                    let d = dist(point, Point2Dto::new(0.0, 0.0));
                    if d <= 7.0 && d > residual + 1e-7 {
                        assert!(
                            arcs.iter().any(|arc| arc.distance(point) <= s + 1e-7),
                            "uncut exterior sample {point:?}"
                        );
                    }
                }
            }
        }
    }
}
