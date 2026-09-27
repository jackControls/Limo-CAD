//! One transient projected-point session, backed by the existing form adapters.
use super::*;
use operation_editor::linking_points::picking as linking;
use setup::picking as wcs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SelectionState {
    Wcs(wcs::SelectionState),
    Linking(linking::SelectionState),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Key {
    Wcs(wcs::Key),
    Linking(linking::Key),
}
pub(super) struct Candidate {
    pub key: Key,
    pub point: [f64; 3],
}
pub(super) enum Source {
    Wcs(wcs::Source),
    Linking(Vec<linking::Candidate>),
}
pub(super) fn snapshot(draft: &Draft, path: &str) -> Result<SelectionState, String> {
    if path == wcs::BUTTON {
        wcs::snapshot(draft).map(SelectionState::Wcs)
    } else {
        linking::snapshot(draft, path).map(SelectionState::Linking)
    }
}
pub(super) fn current(draft: &Draft, expected: &SelectionState) -> Result<SelectionState, String> {
    let path = match expected {
        SelectionState::Wcs(_) => wcs::BUTTON.to_owned(),
        SelectionState::Linking(state) => linking::button(state.target.key()),
    };
    snapshot(draft, &path)
}
pub(super) fn target_matches(expected: &SelectionState, path: &str) -> bool {
    match expected {
        SelectionState::Wcs(_) => path == wcs::BUTTON,
        SelectionState::Linking(state) => path == linking::button(state.target.key()),
    }
}
pub(super) fn source(
    draft: &Draft,
    cam: &CamDocumentDto,
    state: &SelectionState,
) -> Result<Source, String> {
    match state {
        SelectionState::Wcs(state) => wcs::source(draft, cam, state).map(Source::Wcs),
        SelectionState::Linking(state) => linking::candidates(draft, state).map(Source::Linking),
    }
}
pub(super) fn candidates(source: Source) -> Result<Vec<Candidate>, String> {
    Ok(match source {
        Source::Wcs(source) => wcs::candidates(source)?
            .into_iter()
            .map(|point| Candidate {
                key: Key::Wcs(point.key),
                point: point.point,
            })
            .collect(),
        Source::Linking(points) => points
            .into_iter()
            .map(|point| Candidate {
                key: Key::Linking(point.key),
                point: point.point,
            })
            .collect(),
    })
}
pub(super) fn stage(
    draft: &mut Draft,
    cam: &CamDocumentDto,
    expected: &SelectionState,
    key: &Key,
) -> Result<(), String> {
    match (expected, key) {
        (SelectionState::Wcs(state), Key::Wcs(key)) => wcs::stage(draft, state, key),
        (SelectionState::Linking(state), Key::Linking(key)) => {
            linking::stage(draft, cam, state, *key)
        }
        _ => Err("The point picker target changed".into()),
    }
}
pub(super) fn staged_message(selection: &SelectionState) -> &'static str {
    match selection {
        SelectionState::Wcs(_) => "WCS origin is staged. Apply saves the setup.",
        SelectionState::Linking(_) => "Linking position is staged. Apply saves the toolpath.",
    }
}
