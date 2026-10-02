#[test]
fn flipped_roughing_uses_source_removal_and_invalidates_when_source_changes() {
    use crate::{simulate_setup, CamSimulationRequestDto};
    let mut doc = fixture(vec![cuboid([6., 5., -3.], [10., 9., 0.])]);
    let mut rest = doc.setups[0].clone();
    rest.id = 2;
    rest.name = "Flipped remaining stock".into();
    rest.stock_spec = CamStockSpecDto::RestFromSetup { setup_id: 1 };
    rest.resolved_stock = CamResolvedStockDto::Rest { source_setup_id: 1 };
    rest.wcs.y_axis = [0., -1., 0.];
    rest.wcs.z_axis = [0., 0., -1.];
    rest.stock = rest.wcs.stock_from(doc.setups[0].wcs, &doc.setups[0].stock);
    if let CamOperationDto::Adaptive3d {
        id,
        top_z,
        bottom_z,
        clearance_z,
        retract_z,
        feed_height_z,
        ..
    } = &mut rest.operations[0]
    {
        *id = 2;
        *top_z = 3.;
        *bottom_z = 1.;
        *clearance_z = 8.;
        *retract_z = 6.;
        *feed_height_z = 4.;
    }
    doc.setups.push(rest);
    doc.next_setup_id = 3;
    doc.next_operation_id = 3;
    let request = |setup_id| CamSimulationRequestDto {
        setup_id,
        voxel_size: Some(0.5),
        max_voxels: None,
        stock_mesh: None,
        target: None,
        through_operation_id: None,
        completed_steps: None,
        playback_time_seconds: None,
    };
    let source = simulate_setup(&doc, &request(1)).unwrap();
    let result = simulate_setup(&doc, &request(2)).unwrap();
    assert_eq!(result.initial_voxels, source.remaining_voxels);
    assert!(result.removed_voxels > 0);
    assert!(result.collisions.is_empty(), "{:?}", result.collisions);
    let first = plan_setup(&doc, 2).unwrap();
    assert_eq!(first, plan_setup(&doc, 2).unwrap());
    if let CamOperationDto::Adaptive3d { bottom_z, .. } = &mut doc.setups[0].operations[0] {
        *bottom_z = -3.;
    }
    match plan_setup(&doc, 2) {
        Ok(changed) => assert!(
            changed.stats.cutting_distance < first.stats.cutting_distance,
            "upstream removal must participate in the adaptive cache key"
        ),
        Err(error) => assert!(
            error.0.contains("no accessible cutting area"),
            "{}",
            error.0
        ),
    }
}
