//! Definition-level manufacturing requests over the shared, guarded document engine.
use super::*;
use crate::session_bridge::parse_engine_envelope;
use limo_cad_core::{PrintIntentDocumentDto, PrintIntentPresetDto, PrintSettingsDto};
use limo_cad_interface::{ChoiceOption, ControlInput, Field as ControlField, KeyChord};

mod panel;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Field {
    Scope,
    Body,
    Walls,
    Density,
    Pattern,
    Top,
    Bottom,
    Preset,
    PresetName,
    CopyFrom,
    Target,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Field(Field),
    Apply,
    Inherit,
    Discard,
    SavePreset,
    DeletePreset,
    Copy,
    Close,
    Scroll(i32),
    Info,
}
#[derive(Resource, Default)]
struct State {
    visible: bool,
    owner: Option<DocumentContext>,
    loaded_revision: Option<u64>,
    generation: u64,
    body: u64,
    project: bool,
    document: Option<PrintIntentDocumentDto>,
    original: PrintSettingsDto,
    draft: PrintSettingsDto,
    expected_model: String,
    effective: Value,
    target: String,
    preset: String,
    preset_name: String,
    copy_from: String,
    errors: std::collections::BTreeMap<&'static str, (String, String)>,
    error: Option<String>,
    scroll: usize,
    widgets: chrome::Widgets,
}
impl State {
    fn dirty(&self) -> bool {
        self.original != self.draft || !self.errors.is_empty()
    }
    fn current(&self, document: &PrintIntentDocumentDto) -> PrintSettingsDto {
        if self.project {
            document.defaults.clone()
        } else {
            document
                .parts
                .iter()
                .find(|p| p.body_id.0 == self.body)
                .map(|p| p.settings.clone())
                .unwrap_or_default()
        }
    }
    fn accept(
        &mut self,
        document: PrintIntentDocumentDto,
        model: String,
        effective: Value,
        revision: u64,
    ) {
        let canonical = self.current(&document);
        if self.document.is_none() || !self.dirty() {
            self.original = canonical;
            self.draft = self.original.clone();
            self.errors.clear();
            self.expected_model = model;
            self.error = None;
        } else if canonical == self.original {
            self.expected_model = model;
        } else {
            self.error = Some(
                "Saved settings changed. Discard draft to reload them before applying.".into(),
            );
        }
        self.document = Some(document);
        self.effective = effective;
        self.loaded_revision = Some(revision);
    }
}

pub(crate) fn active(world: &World) -> bool {
    world.get_resource::<State>().is_some_and(|s| s.visible)
}

pub(super) fn after_history(world: &mut World, owner: &DocumentContext) {
    if let Some(mut state) = world.get_resource_mut::<State>().filter(|s| {
        s.owner.as_ref().is_some_and(|previous| {
            previous.window_id == owner.window_id && previous.document_id == owner.document_id
        }) && !s.dirty()
    }) {
        state.owner = Some(owner.clone());
        state.document = None;
        state.loaded_revision = None;
        state.generation = state.generation.saturating_add(1);
    }
}
pub(crate) fn ensure_clean(world: &World) -> Result<(), String> {
    if world
        .get_resource::<State>()
        .is_some_and(|s| s.visible && s.dirty())
    {
        Err("Apply or discard the Print Settings draft before continuing".into())
    } else {
        Ok(())
    }
}
pub(crate) fn open(
    world: &mut World,
    engine: &AppState,
    owner: &DocumentContext,
    body: Option<u64>,
) -> Result<Value, String> {
    ensure_clean(world)?;
    if named_views::presentation_locked(world) || crate::native_editor::active(engine)?.is_some() {
        return Err("Finish the source feature, sketch, joint, motion or study editor before editing print settings".into());
    }
    let (document, _, presentation, _) = native_viewport::interface_view_snapshot(world);
    if document != owner.document_id {
        return Err("The viewport is still changing documents".into());
    }
    let body = body.or_else(|| {
        (presentation.selected_body_ids.len() == 1).then(|| presentation.selected_body_ids[0])
    });
    let body = body.ok_or("Select one CAD body to edit its print settings")?;
    if !engine
        .viewport_snapshot()
        .2
        .bodies
        .iter()
        .any(|b| b.id.0 == body)
    {
        return Err("The selected source body was removed".into());
    }
    world.init_resource::<State>();
    let mut state = world.resource_mut::<State>();
    state.visible = true;
    state.owner = Some(owner.clone());
    state.body = body;
    state.project = false;
    state.document = None;
    state.loaded_revision = None;
    state.generation = state
        .generation
        .checked_add(1)
        .ok_or("Print settings generation exhausted")?;
    state.target = "portable".into();
    state.scroll = 0;
    Ok(json!({"opened":true,"body_id":body,"all_occurrences":true}))
}

fn field_key(field: Field) -> &'static str {
    match field {
        Field::Walls => "wall_count",
        Field::Density => "infill_density_percent",
        Field::Pattern => "infill_pattern",
        Field::Top => "top_shell_layers",
        Field::Bottom => "bottom_shell_layers",
        _ => "",
    }
}
fn settings_text(settings: &PrintSettingsDto, field: Field) -> String {
    serde_json::to_value(settings)
        .ok()
        .and_then(|v| v.get(field_key(field)).cloned())
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        })
        .unwrap_or_default()
}
fn edit_settings(settings: &mut PrintSettingsDto, field: Field, text: &str) -> Result<(), String> {
    let text = text.trim();
    let value = if text.is_empty() || text.eq_ignore_ascii_case("inherit") {
        Value::Null
    } else if field == Field::Pattern {
        json!(text)
    } else if field == Field::Density {
        let value = text
            .parse::<f64>()
            .map_err(|_| "Enter an infill percentage from 0 to 100, or leave blank to inherit")?;
        if !value.is_finite() {
            return Err("Infill percentage must be finite".into());
        }
        json!(value)
    } else {
        json!(text
            .parse::<u32>()
            .map_err(|_| "Enter a whole count from 0 to 1000, or leave blank to inherit")?)
    };
    let mut candidate = serde_json::to_value(&*settings).map_err(|e| e.to_string())?;
    candidate[field_key(field)] = value;
    let candidate: PrintSettingsDto =
        serde_json::from_value(candidate).map_err(|e| e.to_string())?;
    candidate.validate()?;
    *settings = candidate;
    Ok(())
}
fn option(value: impl Into<String>, label: impl Into<String>) -> ChoiceOption {
    ChoiceOption {
        value: value.into(),
        label: label.into(),
        disabled: false,
    }
}
fn body_choices(world: &World, state: &State) -> Vec<ChoiceOption> {
    let mut options: Vec<_> = native_viewport::interface_geometry(world)
        .scene
        .bodies
        .iter()
        .map(|b| option(b.id.0.to_string(), format!("{} (body {})", b.name, b.id.0)))
        .collect();
    if let Some(document) = &state.document {
        for part in &document.parts {
            let id = part.body_id.0.to_string();
            if !options.iter().any(|o| o.value == id) {
                options.push(option(
                    id,
                    format!("Retained/orphan source body {}", part.body_id.0),
                ));
            }
        }
    }
    options
}
fn choices(world: &World, state: &State, field: Field) -> Option<Vec<ChoiceOption>> {
    Some(match field {
        Field::Body | Field::CopyFrom => body_choices(world, state),
        Field::Scope => vec![
            option("part", "Selected part · all occurrences"),
            option("project", "Project defaults"),
        ],
        Field::Pattern => std::iter::once(option("", "Inherit"))
            .chain(
                [
                    "grid",
                    "gyroid",
                    "rectilinear",
                    "concentric",
                    "cubic",
                    "honeycomb",
                    "lightning",
                ]
                .into_iter()
                .map(|s| option(s, s)),
            )
            .collect(),
        Field::Target => vec![
            option("portable", "Portable model"),
            option("bambu_studio", "Bambu Studio"),
            option("orca_slicer", "OrcaSlicer"),
            option("prusa_slicer", "PrusaSlicer"),
        ],
        Field::Preset => std::iter::once(option("", "Choose preset"))
            .chain(
                state
                    .document
                    .as_ref()?
                    .presets
                    .iter()
                    .map(|p| option(&p.name, &p.name)),
            )
            .collect(),
        _ => return None,
    })
}
fn text(state: &State, field: Field) -> String {
    match field {
        Field::Body => state.body.to_string(),
        Field::Scope => if state.project { "project" } else { "part" }.into(),
        Field::Target => state.target.clone(),
        Field::Preset => state.preset.clone(),
        Field::PresetName => state.preset_name.clone(),
        Field::CopyFrom => state.copy_from.clone(),
        _ => state
            .errors
            .get(field_key(field))
            .map(|e| e.0.clone())
            .unwrap_or_else(|| settings_text(&state.draft, field)),
    }
}
pub(crate) fn reduce(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    engine: &AppState,
    bridge: &SessionBridgeState,
    action: &NativeInterfaceAction,
    generation: u64,
    command: &Command,
) -> Result<Value, String> {
    let receipt = bridge.native_document_receipt(engine, &action.context)?;
    handle.validate_action(action)?;
    let state = world.get_resource::<State>().ok_or("Open Print Settings")?;
    if !state.visible
        || state.owner.as_ref() != Some(&receipt.owner)
        || state.generation != generation
    {
        return Err("The print settings controls changed".into());
    }
    if let Command::Field(field) = command {
        let value = if let Some(options) = choices(world, state, *field) {
            workbench::cam::choose(&options, &text(state, *field), &action.control.input)?
        } else if let ControlInput::SetValue(value) = &action.control.input {
            value.clone()
        } else {
            return Ok(json!({"focused":true}));
        };
        let mut state = world.resource_mut::<State>();
        match field {
            Field::Scope | Field::Body => {
                if state.dirty() {
                    return Err(
                        "Apply or discard the print settings draft before changing parts or scope"
                            .into(),
                    );
                }
                if *field == Field::Body {
                    state.body = value.parse().map_err(|_| "Choose a source body")?;
                } else {
                    state.project = value == "project";
                }
                state.document = None;
                state.loaded_revision = None;
                state.scroll = 0;
                state.preset.clear();
                state.preset_name.clear();
                state.copy_from.clear();
                state.generation = state
                    .generation
                    .checked_add(1)
                    .ok_or("Print settings generation exhausted")?;
            }
            Field::Target => {
                state.target = value;
                state.loaded_revision = None;
                state.generation = state
                    .generation
                    .checked_add(1)
                    .ok_or("Print settings generation exhausted")?;
            }
            Field::Preset => {
                if let Some(preset) = state
                    .document
                    .as_ref()
                    .and_then(|d| d.presets.iter().find(|p| p.name == value))
                    .cloned()
                {
                    state.draft = preset.settings;
                    state.errors.clear();
                    state.preset_name = preset.name;
                }
                state.preset = value;
            }
            Field::PresetName => state.preset_name = value,
            Field::CopyFrom => state.copy_from = value,
            _ => {
                if let Err(error) = edit_settings(&mut state.draft, *field, &value) {
                    state
                        .errors
                        .insert(field_key(*field), (value, error.clone()));
                    return Ok(json!({"handled":true,"valid":false,"error":error}));
                }
                state.errors.remove(field_key(*field));
            }
        }
        return Ok(json!({"handled":true}));
    }
    if !super::super::is_activation(&action.control.input) {
        return Err("Activate a print settings control".into());
    }
    let mut state = world.resource_mut::<State>();
    match command {
        Command::Info => return Ok(json!({"read_only":true})),
        Command::Scroll(delta) => {
            state.scroll = state.scroll.saturating_add_signed(*delta as isize);
            return Ok(json!({"handled":true}));
        }
        Command::Close => {
            if state.dirty() {
                return Err("Apply or discard the print settings draft before closing".into());
            }
            state.visible = false;
            return Ok(json!({"closed":true}));
        }
        Command::Discard => {
            state.draft = state.current(state.document.as_ref().ok_or("Wait for print settings")?);
            state.original = state.draft.clone();
            state.errors.clear();
            state.error = None;
            state.loaded_revision = None;
            return Ok(json!({"discarded":true}));
        }
        _ => {}
    }
    if state.loaded_revision != Some(receipt.revision) || state.document.is_none() {
        return Err("Wait for current print settings to load".into());
    }
    if let Some(error) = state
        .errors
        .values()
        .next()
        .filter(|_| !matches!(command, Command::Inherit))
    {
        return Err(error.1.clone());
    }
    let (operation, mut args) = match command {
        Command::Apply | Command::Inherit => {
            let settings = if matches!(command, Command::Inherit) {
                PrintSettingsDto::default()
            } else {
                state.draft.clone()
            };
            if state.project {
                let mut document = state.document.clone().unwrap();
                document.defaults = settings;
                ("print_intent_set_document", json!({"document":document}))
            } else if matches!(command, Command::Inherit) {
                ("print_intent_reset_part", json!({"body_id":state.body}))
            } else {
                (
                    "print_intent_set_part",
                    json!({"body_id":state.body,"settings":settings}),
                )
            }
        }
        Command::SavePreset => {
            let name = state.preset_name.trim();
            if name.is_empty() {
                return Err("Enter a name for the print preset".into());
            }
            (
                "print_intent_upsert_preset",
                json!({"preset":PrintIntentPresetDto{name:name.into(),settings:state.draft.clone()}}),
            )
        }
        Command::DeletePreset => {
            if state.preset.is_empty() {
                return Err("Choose a print preset to delete".into());
            }
            ("print_intent_remove_preset", json!({"name":state.preset}))
        }
        Command::Copy => {
            if state.project {
                return Err("Copy requests applies to part definitions".into());
            }
            if state.dirty() {
                return Err(
                    "Apply or discard the draft before copying another part's requests".into(),
                );
            }
            let source = state
                .copy_from
                .parse::<u64>()
                .map_err(|_| "Choose a source part to copy")?;
            (
                "print_intent_copy_part",
                json!({"source_body_id":source,"target_body_ids":[state.body]}),
            )
        }
        _ => unreachable!(),
    };
    args["expected_model_json"] = json!(state.expected_model);
    drop(state);
    worker::enqueue_operation(
        world,
        receipt.owner,
        receipt.revision,
        operation.into(),
        args,
        move |world, services, result| {
            let result = result?;
            if operation == "print_intent_remove_preset" {
                if let Some(mut state) = world.get_resource_mut::<State>() {
                    state.preset.clear();
                }
            }
            if matches!(
                operation,
                "print_intent_set_part"
                    | "print_intent_reset_part"
                    | "print_intent_copy_part"
                    | "print_intent_set_document"
            ) {
                if let Some(mut state) = world.get_resource_mut::<State>() {
                    state.document = None;
                    state.errors.clear();
                }
                if operation == "print_intent_reset_part" {
                    let bodies = services.engine.viewport_snapshot().2.bodies;
                    if let Some(mut state) = world.get_resource_mut::<State>() {
                        if !bodies.iter().any(|b| b.id.0 == state.body) {
                            if let Some(body) = bodies.first() {
                                state.body = body.id.0;
                            } else {
                                state.visible = false;
                            }
                        }
                    }
                }
            }
            Ok(finish_mutation(
                &services.engine,
                &services.bridge,
                world,
                operation,
                result,
            ))
        },
    )
}

pub(super) fn synchronize(
    world: &mut World,
    camera: Entity,
    services: &NativeServices,
    owner: &DocumentContext,
    width: f32,
    height: f32,
) -> Result<(), String> {
    let mut state = world.remove_resource::<State>().unwrap_or_default();
    state.widgets.begin();
    if state.owner.as_ref() != Some(owner) {
        state.visible = false;
        state.document = None;
        state.loaded_revision = None;
        state.errors.clear();
        state.draft = Default::default();
        state.original = Default::default();
    }
    let can_paint = state.visible
        && !named_views::presentation_locked(world)
        && native_viewport::interface_view_snapshot(world).2.mode
            != native_viewport::ViewportMode::Sketch;
    let result = if can_paint {
        panel::paint(world, camera, &mut state, width, height)
    } else {
        Ok(())
    };
    state.widgets.finish(world);
    let revision = services
        .bridge
        .native_document_receipt(&services.engine, owner)?
        .revision;
    let reload = can_paint && state.loaded_revision != Some(revision) && !worker::busy(world);
    let generation = state.generation;
    let target = state.target.clone();
    let body = state.body;
    world.insert_resource(state);
    result?;
    if reload {
        let owner = owner.clone();
        worker::enqueue_document_io(
            world,
            "print-intent-load".into(),
            move |services, guard| {
                services.bridge.with_native_document_receipt(&services.engine,&owner,|current| {
                if current!=revision {return Err("Print settings changed while loading".into());}
                guard.validate()?;
                let read=|op:&str,args:Value|parse_engine_envelope(services.engine.engine_call(op,&args.to_string()));
                Ok(NativeMutationResult {context:owner.clone(),engine_revision:revision,value:json!({
                    "document":read("print_intent_get",json!({}))?,"model":read("project_export_model",json!({}))?,
                    "effective":read("print_intent_effective",json!({"body_ids":[body],"target":target}))?
                })})
            })
            },
            move |world, services, result| {
                let result = match result {
                    Ok(result) => result,
                    Err(error) => {
                        if let Some(mut state) = world
                            .get_resource_mut::<State>()
                            .filter(|s| s.generation == generation && s.body == body)
                        {
                            state.error=Some(format!("Could not load settings: {error}. Close and reopen Print Settings to retry."));
                            state.loaded_revision = Some(revision);
                        }
                        return Err(error);
                    }
                };
                services.bridge.with_native_document_receipt(
                    &services.engine,
                    &result.context,
                    |current| {
                        let state = world
                            .get_resource_mut::<State>()
                            .ok_or("Print settings closed")?;
                        if current != revision
                            || state.owner.as_ref() != Some(&result.context)
                            || state.generation != generation
                            || state.body != body
                        {
                            return Err("The print settings editor changed while loading".into());
                        }
                        let mut state = state;
                        state.accept(
                            serde_json::from_value(result.value["document"].clone())
                                .map_err(|e| e.to_string())?,
                            result.value["model"]
                                .as_str()
                                .ok_or("Missing print intent model precondition")?
                                .into(),
                            result.value["effective"].clone(),
                            revision,
                        );
                        Ok(json!({"loaded":true}))
                    },
                )
            },
        )?;
    }
    Ok(())
}

pub(crate) fn caption(world: &World) -> Option<String> {
    let state = world.get_resource::<State>().filter(|s| s.visible)?;
    Some(format!("Requested print settings apply to every occurrence of body {}. Blank = Inherit; 0 = explicit. {}",state.body,state.error.as_deref().unwrap_or("")))
}
