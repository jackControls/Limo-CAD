use super::*;
use nbcad_interface::{Field, KeyChord};

fn button(
    world: &mut World,
    camera: Entity,
    widgets: &mut Widgets,
    key: &str,
    label: &str,
    command: Command,
    bounds: Node,
    disabled: bool,
) -> Result<(), String> {
    let mut control = InterfaceControl::button("drawing/document", label);
    control.disabled = disabled;
    widgets.button(
        world,
        camera,
        key,
        control,
        None,
        native(command),
        bounds,
        None,
        46,
    )?;
    Ok(())
}
fn choice(
    world: &mut World,
    camera: Entity,
    widgets: &mut Widgets,
    key: &str,
    label: &str,
    command: Command,
    value: String,
    options: Vec<ChoiceOption>,
    bounds: Node,
) -> Result<(), String> {
    let caption = options
        .iter()
        .find(|o| o.value == value)
        .map(|o| o.label.as_str())
        .unwrap_or("Choose…")
        .to_owned();
    let mut control = InterfaceControl::button("drawing/document", label);
    control.role = "combobox".into();
    control.disabled = options.is_empty();
    control.owned_keys = [
        "ArrowUp",
        "ArrowDown",
        "ArrowLeft",
        "ArrowRight",
        "Home",
        "End",
    ]
    .map(KeyChord::plain)
    .into();
    control.field = Field::Choice { value, options };
    widgets.button(
        world,
        camera,
        key,
        control,
        Some(&caption),
        native(command),
        bounds,
        None,
        46,
    )?;
    Ok(())
}
pub(super) fn paint(
    world: &mut World,
    camera: Entity,
    editor: &mut Editor,
    height: f32,
    side: f32,
) -> Result<(), String> {
    let theme = crate::native_viewport::ui::theme(world);
    let width = side.max(248.);
    let bottom = (height - 66.).max(280.);
    editor.widgets.panel(
        world,
        camera,
        "drawing-editor-panel",
        rect(0., 112., width, bottom - 112.),
        theme.panel,
        44,
    );
    editor.widgets.text(
        world,
        camera,
        "drawing-editor-title",
        rect(12., 121., width - 24., 18.),
        "Drawing sheets and views",
        12.,
        45,
    );
    choice(
        world,
        camera,
        &mut editor.widgets,
        "drawing-sheet-choice",
        "Sheet",
        Command::SheetChoice,
        editor
            .document
            .active_sheet_id
            .map_or(String::new(), |id| id.to_string()),
        options(model::sheets(&editor.document)),
        rect(10., 146., width - 20., 28.),
    )?;
    let view = editor
        .draft
        .as_ref()
        .and_then(|d| match d.selection {
            Selection::View(id) => Some(id.to_string()),
            _ => None,
        })
        .unwrap_or_default();
    choice(
        world,
        camera,
        &mut editor.widgets,
        "drawing-view-choice",
        "View",
        Command::ViewChoice,
        view,
        options(model::views(&editor.document)),
        rect(10., 179., width - 20., 28.),
    )?;
    let active_sheet = editor
        .document
        .sheets
        .iter()
        .find(|s| Some(s.id) == editor.document.active_sheet_id);
    button(
        world,
        camera,
        &mut editor.widgets,
        "drawing-sheet-edit",
        "Sheet setup",
        Command::Sheet,
        rect(10., 212., (width - 26.) / 2., 28.),
        active_sheet.is_none(),
    )?;
    button(
        world,
        camera,
        &mut editor.widgets,
        "drawing-auto-layout",
        "Auto-layout",
        Command::AutoLayout,
        rect(width / 2. + 3., 212., (width - 26.) / 2., 28.),
        active_sheet.is_none_or(|s| !s.views.is_empty()),
    )?;
    let Some(draft) = &editor.draft else {
        editor.widgets.text(
            world,
            camera,
            "drawing-editor-empty",
            rect(12., 258., width - 24., 54.),
            "Create a sheet to edit its setup and place views.",
            11.,
            45,
        );
        return Ok(());
    };
    let visible: Vec<_> = draft
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.path != "/tolerance_note/custom"
                || draft.fields.iter().any(|preset| {
                    preset.path == "/tolerance_note/preset" && preset.text == "custom"
                })
        })
        .collect();
    let page_size = (((bottom - 367.) / 46.).floor() as usize).clamp(1, 9);
    editor.page = editor.page.min(visible.len().saturating_sub(1) / page_size);
    for (position, (index, field)) in visible
        .iter()
        .copied()
        .enumerate()
        .skip(editor.page * page_size)
        .take(page_size)
    {
        let y = 250. + (position % page_size) as f32 * 46.;
        editor.widgets.text(
            world,
            camera,
            &format!("drawing-label-{index}"),
            rect(12., y, width - 24., 16.),
            field.label,
            10.,
            45,
        );
        let key = format!("drawing-field-{}", field.path);
        let command = Command::Edit(draft.selection, index);
        if let model::Kind::Choice(values) = field.kind {
            choice(
                world,
                camera,
                &mut editor.widgets,
                &key,
                field.label,
                command,
                field.text.clone(),
                values
                    .iter()
                    .map(|(value, label)| ChoiceOption {
                        value: (*value).into(),
                        label: (*label).into(),
                        disabled: false,
                    })
                    .collect(),
                rect(10., y + 16., width - 20., 28.),
            )?;
        } else {
            let mut control = InterfaceControl::button("drawing/document", field.label);
            control.field = Field::Text {
                value: field.text.clone(),
                read_only: false,
                selection: None,
            };
            editor.widgets.button(
                world,
                camera,
                &key,
                control,
                None,
                native(command),
                rect(10., y + 16., width - 20., 28.),
                None,
                46,
            )?;
        }
    }
    let y = 252. + page_size as f32 * 46.;
    if visible.len() > page_size {
        button(
            world,
            camera,
            &mut editor.widgets,
            "drawing-fields-prev",
            "Previous fields",
            Command::Fields(-1),
            rect(10., y, (width - 26.) / 2., 26.),
            editor.page == 0,
        )?;
        button(
            world,
            camera,
            &mut editor.widgets,
            "drawing-fields-next",
            "More fields",
            Command::Fields(1),
            rect(width / 2. + 3., y, (width - 26.) / 2., 26.),
            (editor.page + 1) * page_size >= visible.len(),
        )?;
    }
    button(
        world,
        camera,
        &mut editor.widgets,
        "drawing-apply",
        "Apply",
        Command::Apply,
        rect(10., y + 32., (width - 26.) / 2., 28.),
        false,
    )?;
    button(
        world,
        camera,
        &mut editor.widgets,
        "drawing-reset",
        "Reset",
        Command::Reset,
        rect(width / 2. + 3., y + 32., (width - 26.) / 2., 28.),
        false,
    )?;
    let message = if editor.message.is_empty() {
        if draft.dirty() {
            "Apply or reset to keep editing another item"
        } else {
            "Paper placement uses millimetres"
        }
    } else {
        &editor.message
    };
    editor.widgets.text(
        world,
        camera,
        "drawing-editor-message",
        rect(12., y + 66., width - 24., 48.),
        message,
        10.,
        45,
    );
    Ok(())
}
