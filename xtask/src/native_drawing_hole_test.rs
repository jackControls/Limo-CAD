//! Bounded live native HoleNote proof on an explicitly supplied blank session.
use crate::native_fixture::start;
use anyhow::Result;

pub(super) fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let mut fixture = start(args, "native-drawing-hole")?;
    // The owned launcher sets this only after --desktop-input. Headless
    // native-drawing-hole keeps the published-control proof.
    let physical = std::env::var("NBCAD_NATIVE_HOLE_INPUT").as_deref() == Ok("1");
    let result = crate::native_drawing_authoring_test::exercise_hole(
        &mut fixture.client,
        &fixture.out,
        &fixture.server,
        physical,
    )?;
    std::fs::write(&fixture.report, serde_json::to_vec_pretty(&result)?)?;
    println!(
        "Hole note state/export checks passed; review original captures in {}",
        fixture.out.display()
    );
    Ok(())
}
