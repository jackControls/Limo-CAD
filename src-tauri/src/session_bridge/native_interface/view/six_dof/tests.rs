use super::*;
fn camera() -> ViewportCamera {
    ViewportCamera {
        position: [0., 0., 10.],
        target: [0., 0., 0.],
        up: [0., 1., 0.],
        vertical_fov_degrees: 15.2,
    }
}
fn close(a: Vec3, b: Vec3) {
    assert!((a - b).length() < 1e-4, "{a:?} != {b:?}");
}

#[test]
fn native_object_motion_matches_release_pan_dolly_and_speed_bounds() {
    let before = camera();
    let motion = Motion {
        translation: [1., 0., 1.],
        ..Default::default()
    };
    let moved = move_camera(before, Vec3::ZERO, motion, 1. / 60., 1.5).unwrap();
    close(
        Vec3::from_array(moved.position),
        Vec3::new(-0.225, -0.225, 10.),
    );
    close(
        Vec3::from_array(moved.target),
        Vec3::new(-0.225, -0.225, 0.),
    );
    assert_eq!(moved.up, before.up);
    let dolly = Motion {
        translation: [0., 1., 0.],
        ..Default::default()
    };
    let moved = move_camera(before, Vec3::ZERO, dolly, 1. / 60., 1.5).unwrap();
    assert!((moved.position[2] - 10. * 0.0225_f32.exp()).abs() < 1e-4);
    assert_eq!(moved.target, before.target);
    assert_eq!(
        move_camera(before, Vec3::ZERO, motion, 10., 20.).unwrap(),
        move_camera(before, Vec3::ZERO, motion, 0.05, 3.).unwrap()
    );
    assert_eq!(
        move_camera(before, Vec3::ZERO, motion, 0., 0.).unwrap(),
        move_camera(before, Vec3::ZERO, motion, 0.001, 0.25).unwrap()
    );
}

#[test]
fn rotation_turns_the_entire_camera_rig_around_the_visible_solid_without_recentering() {
    let mut before = camera();
    before.target = [2., 0., 0.];
    let pivot = Vec3::new(3., 4., 5.);
    let motion = Motion {
        rotation: [0.5, -0.2, 1.],
        ..Default::default()
    };
    let moved = move_camera(before, pivot, motion, 1. / 60., 1.5).unwrap();
    let distance = |point: [f32; 3]| Vec3::from_array(point).distance(pivot);
    assert!((distance(moved.position) - distance(before.position)).abs() < 1e-4);
    assert!((distance(moved.target) - distance(before.target)).abs() < 1e-4);
    let rig = |camera: ViewportCamera| {
        Vec3::from_array(camera.position).distance(Vec3::from_array(camera.target))
    };
    assert!((rig(moved) - rig(before)).abs() < 1e-4);
    assert_ne!(moved.target, before.target);
    assert!((Vec3::from_array(moved.up).length() - 1.).abs() < 1e-5);
    assert!(move_camera(before, pivot, motion, f32::NAN, 1.).is_err());
    assert!(move_camera(before, Vec3::NAN, motion, 1. / 60., 1.).is_err());
}
