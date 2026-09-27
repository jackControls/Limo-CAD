use super::super::*;
use super::{
    anchors,
    draft::{Draft, Selection},
    fields::{self, Field},
    *,
};
use nbcad_sketch::{DrawingDocumentDto, DrawingSheetDto};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tool {
    Note,
    Linear,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Tool(Tool),
    Field(usize),
    Apply,
    Reset,
    Delete,
    Select(u64),
    Anchor(usize),
    Cancel,
    Fields(i32),
}

pub(super) struct Target {
    pub view_id: u64,
    pub reference: DrawingTopologyAnchorRefDto,
    pub paper: [f64; 2],
}
pub(super) struct Drag {
    pub stamp: Stamp,
    pub start: [f64; 2],
    pub draft: Draft,
    pub linear_points: Option<[[f64; 2]; 2]>,
    pub moved: bool,
}
#[derive(Resource, Default)]
pub(super) struct Editor {
    pub stamp: Option<Stamp>,
    pub document: DrawingDocumentDto,
    pub serial: u64,
    pub selected: Option<u64>,
    pub pending_selected: Option<u64>,
    pub draft: Option<Draft>,
    pub fields: Vec<Field>,
    pub tool: Option<Tool>,
    pub pair: LinearPlacement,
    pub targets: Vec<Target>,
    pub drag: Option<Drag>,
    pub message: String,
    pub page: usize,
    pub widgets: Widgets,
}
impl Editor {
    fn inactive(&mut self, owner: &DocumentContext) {
        if self
            .stamp
            .as_ref()
            .is_some_and(|stamp| &stamp.owner == owner)
        {
            self.drag = None;
            self.pair.cancel();
        } else {
            self.clear();
            self.stamp = None;
        }
    }
    pub fn dirty(&self) -> bool {
        self.draft.is_some() && fields::dirty(&self.fields)
    }
    pub fn select(&mut self, id: u64) -> Result<(), String> {
        let sheet_id = self.stamp.as_ref().ok_or("Create a sheet first")?.sheet_id;
        let draft = Draft::new(
            &self.document,
            Selection {
                sheet_id,
                annotation_id: id,
            },
        )?;
        self.fields = fields::from_annotation(draft.annotation());
        self.draft = Some(draft);
        self.selected = Some(id);
        self.tool = None;
        self.pair.cancel();
        self.page = 0;
        self.serial = self.serial.wrapping_add(1);
        self.message.clear();
        Ok(())
    }
    pub fn clear(&mut self) {
        self.draft = None;
        self.fields.clear();
        self.selected = None;
        self.pending_selected = None;
        self.tool = None;
        self.pair.cancel();
        self.drag = None;
        self.page = 0;
        self.message.clear();
        self.serial = self.serial.wrapping_add(1);
    }
}
pub(in super::super) fn native(serial: u64, command: Command) -> NativeCommand {
    NativeCommand::Drawing(drawing_editor::Command::Annotation(serial, command))
}
pub(in super::super) fn owns_panel(world: &World) -> bool {
    world
        .get_resource::<Editor>()
        .is_some_and(|e| e.tool.is_some() || e.selected.is_some())
}
pub(in super::super) fn guard(world: &World) -> Result<(), String> {
    if world.get_resource::<Editor>().is_some_and(Editor::dirty) {
        Err("Apply or reset the annotation edit first".into())
    } else {
        Ok(())
    }
}
pub(in super::super) fn cancel_input(world: &mut World) {
    let changed = if let Some(mut e) = world.get_resource_mut::<Editor>() {
        // A worker may only be reading the model between anchor clicks. Keep
        // the exact stamped pair; owner/revision refresh still retires it.
        e.drag.take().is_some()
    } else {
        false
    };
    if changed {
        let _ = super::input::refresh_preview(world);
    }
}
pub(in super::super) fn preview(
    world: &World,
    sheet: &DrawingSheetDto,
    owner: &DocumentContext,
    revision: u64,
) -> DrawingSheetDto {
    let Some(editor) = world.get_resource::<Editor>() else {
        return sheet.clone();
    };
    let Some(drag) = &editor.drag else {
        return sheet.clone();
    };
    if drag.stamp.sheet_id != sheet.id
        || &drag.stamp.owner != owner
        || drag.stamp.revision != revision
    {
        return sheet.clone();
    }
    let mut next = sheet.clone();
    if let Some(a) = next
        .annotations
        .iter_mut()
        .find(|a| a.id() == drag.draft.selection().annotation_id)
    {
        *a = drag.draft.annotation().clone();
    }
    next
}
pub(in super::super) fn synchronize(
    world: &mut World,
    camera: Entity,
    services: &NativeServices,
    owner: &DocumentContext,
    height: f32,
    side: f32,
    active: bool,
    state: &Workbench,
) -> Result<(), String> {
    let mut e = world.remove_resource::<Editor>().unwrap_or_default();
    e.widgets.begin();
    let result = (|| {
        if !active {
            e.inactive(owner);
            return Ok(());
        }
        let receipt = services
            .bridge
            .native_document_receipt(&services.engine, owner)?;
        let document = services.engine.drawing_snapshot();
        let Some(sheet) = document
            .sheets
            .iter()
            .find(|s| Some(s.id) == document.active_sheet_id)
        else {
            e.clear();
            e.stamp = None;
            return Ok(());
        };
        let stamp = Stamp {
            owner: owner.clone(),
            revision: receipt.revision,
            sheet_id: sheet.id,
        };
        if e.stamp.as_ref() != Some(&stamp) {
            let same = e
                .stamp
                .as_ref()
                .is_some_and(|s| s.owner == stamp.owner && s.sheet_id == stamp.sheet_id);
            let selected = if same {
                e.pending_selected.take().or(e.selected)
            } else {
                None
            };
            e.clear();
            e.document = document.clone();
            e.stamp = Some(stamp.clone());
            if let Some(id) = selected.filter(|id| sheet.annotations.iter().any(|a| a.id() == *id))
            {
                e.select(id)?;
            }
        }
        e.targets.clear();
        if e.tool == Some(Tool::Linear) {
            if let Some(result) =
                drawing_paper::with_projections(world, state, |projections| -> Result<(), String> {
                    for (view, projection) in projections.values() {
                        for a in anchors::endpoints(view, projection)? {
                            e.targets.push(Target {
                                view_id: view.id,
                                reference: anchors::endpoint_ref(a, projection),
                                paper: drawing_paper::paper_point(view, a.point, projection),
                            });
                        }
                        for a in anchors::circles(projection)? {
                            e.targets.push(Target {
                                view_id: view.id,
                                reference: anchors::circle_ref(a, projection),
                                paper: drawing_paper::paper_point(view, a.center, projection),
                            });
                        }
                    }
                    Ok(())
                })
            {
                result?;
            }
        }
        super::panel::paint(world, camera, &mut e, height, side, state)?;
        Ok(())
    })();
    e.widgets.finish(world);
    world.insert_resource(e);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn busy_read_cancels_pointer_drag_but_preserves_the_valid_anchor_pair() {
        let document = super::super::tests::document();
        let projection = super::super::tests::projection();
        let stamp = Stamp {
            owner: DocumentContext {
                window_id: "main".into(),
                document_id: "drawing".into(),
                epoch: 1,
            },
            revision: 12,
            sheet_id: 1,
        };
        let first = anchors::endpoint_ref(&projection.anchors[4], &projection);
        let second = anchors::endpoint_ref(&projection.anchors[5], &projection);
        let mut e = Editor {
            stamp: Some(stamp.clone()),
            document: document.clone(),
            ..default()
        };
        e.pair.click(&stamp, 1, first.clone());
        e.drag = Some(Drag {
            stamp: stamp.clone(),
            start: [20., 30.],
            draft: Draft::new(
                &document,
                Selection {
                    sheet_id: 1,
                    annotation_id: 1,
                },
            )
            .unwrap(),
            linear_points: None,
            moved: false,
        });
        let mut world = World::new();
        world.insert_resource(e);
        cancel_input(&mut world);
        let mut e = world.resource_mut::<Editor>();
        assert!(e.drag.is_none());
        let added = e
            .pair
            .click(&stamp, 1, second)
            .expect("Read-only work must preserve the first anchor");
        assert_eq!(added.first, first);
        e.pair.click(&stamp, 1, first.clone());
        let mut changed = stamp.clone();
        changed.revision += 1;
        e.pair.observe(&changed);
        assert!(e.pair.first.is_none());
        e.pair.click(&stamp, 1, first);
        changed = stamp;
        changed.owner.epoch += 1;
        e.pair.observe(&changed);
        assert!(e.pair.first.is_none());
    }
    #[test]
    fn workspace_switch_keeps_same_owner_form_but_retires_old_document_draft() {
        let owner = DocumentContext {
            window_id: "main".into(),
            document_id: "drawing".into(),
            epoch: 1,
        };
        let mut e = Editor {
            document: super::super::tests::document(),
            stamp: Some(Stamp {
                owner: owner.clone(),
                revision: 12,
                sheet_id: 1,
            }),
            ..default()
        };
        e.select(1).unwrap();
        e.fields[0].text = "Unapplied \u{96f6}\u{4ef6}".into();
        assert!(e.dirty());
        let serial = e.serial;
        e.inactive(&owner);
        assert!(e.dirty());
        assert_eq!(e.selected, Some(1));
        assert_eq!(e.fields[0].text, "Unapplied \u{96f6}\u{4ef6}");
        assert_eq!(e.serial, serial);
        let mut replacement = owner;
        replacement.epoch += 1;
        e.inactive(&replacement);
        assert!(!e.dirty());
        assert!(e.fields.is_empty());
        assert!(e.stamp.is_none());
        assert!(e.selected.is_none());
    }
}

pub(in super::super) fn submit(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    engine: &AppState,
    bridge: &SessionBridgeState,
    stamp: &Stamp,
    operation: &'static str,
    args: Value,
) -> Result<Value, String> {
    if worker::available(world) {
        let expected = stamp.owner.clone();
        return worker::enqueue_operation(
            world,
            stamp.owner.clone(),
            stamp.revision,
            operation.into(),
            args,
            move |world, services, result| {
                let result = result.map_err(|error| {
                    if let Some(mut e) = world.get_resource_mut::<Editor>() {
                        if e.stamp.as_ref().is_some_and(|s| s.owner == expected) {
                            e.message = error.clone();
                        }
                    }
                    error
                })?;
                Ok(finish_mutation(
                    &services.engine,
                    &services.bridge,
                    world,
                    operation,
                    result,
                ))
            },
        );
    }
    let result = bridge.apply_native_mutation_at(
        engine,
        &stamp.owner,
        stamp.revision,
        operation,
        &args,
        || {
            if handle.frame().is_some_and(|f| f.context == stamp.owner) {
                Ok(())
            } else {
                Err("Drawing document changed".into())
            }
        },
    )?;
    Ok(finish_mutation(engine, bridge, world, operation, result))
}

pub(in super::super) fn reduce(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    engine: &AppState,
    bridge: &SessionBridgeState,
    action: &NativeInterfaceAction,
    serial: u64,
    command: &Command,
) -> Result<Value, String> {
    bridge
        .with_native_document_owner(engine, &action.context, || handle.validate_action(action))?;
    let receipt = bridge.native_document_receipt(engine, &action.context)?;
    let mut e = world
        .remove_resource::<Editor>()
        .ok_or("Open the Drawing workspace")?;
    let result = (|| {
        let stamp = e.stamp.clone().ok_or("Create a sheet first")?;
        if workspace(world) != Workspace::Drawing
            || stamp.owner != receipt.owner
            || stamp.revision != receipt.revision
            || (serial != e.serial && !matches!(command, Command::Tool(_)))
        {
            return Err("Drawing changed; use the refreshed controls".into());
        }
        if let Command::Field(index) = command {
            let f = e
                .fields
                .get_mut(*index)
                .ok_or("Drawing field was removed")?;
            f.text = if matches!(f.kind, fields::Kind::Mode) {
                let options = [
                    ("aligned", "Aligned"),
                    ("horizontal", "Horizontal"),
                    ("vertical", "Vertical"),
                ]
                .map(|(value, label)| nbcad_interface::ChoiceOption {
                    value: value.into(),
                    label: label.into(),
                    disabled: false,
                });
                cam::choose(&options, &f.text, &action.control.input)?
            } else {
                match &action.control.input {
                    nbcad_interface::ControlInput::SetValue(v) => v.clone(),
                    _ => return Ok(json!({"handled":true})),
                }
            };
            e.message.clear();
            return Ok(json!({"changed":true}));
        }
        if !super::super::super::super::is_activation(&action.control.input) {
            return Ok(json!({"handled":true}));
        }
        if e.dirty()
            && matches!(
                command,
                Command::Tool(_) | Command::Select(_) | Command::Cancel | Command::Anchor(_)
            )
        {
            return Err("Apply or reset the annotation edit first".into());
        }
        let mut request = None;
        match command {
            Command::Tool(tool) => {
                drawing_editor::guard_sheet_edit(world)?;
                e.clear();
                e.tool = Some(*tool);
                if *tool == Tool::Note {
                    let size = drawing_paper::transform(world.resource::<Workbench>())
                        .ok_or("Open drawing paper")?
                        .sheet_mm;
                    e.fields = fields::note_creation(size.map(|n| n * 0.5));
                }
            }
            Command::Select(id) => {
                drawing_editor::guard_sheet_edit(world)?;
                if e.selected != Some(*id) {
                    e.select(*id)?;
                }
            }
            Command::Anchor(index) => {
                if e.tool != Some(Tool::Linear) {
                    return Err("Choose Linear dimension first".into());
                }
                let target = e.targets.get(*index).ok_or("Projection changed")?;
                if let Some(args) = e
                    .pair
                    .click(&stamp, target.view_id, target.reference.clone())
                {
                    e.pending_selected = Some(e.document.next_annotation_id);
                    request = Some((
                        "drawing_add_linear_dimension",
                        serde_json::to_value(args).map_err(|x| x.to_string())?,
                    ));
                }
            }
            Command::Apply => {
                if e.tool == Some(Tool::Note) {
                    let note = fields::note_request(stamp.sheet_id, &e.fields)?;
                    e.pending_selected = Some(e.document.next_annotation_id);
                    request = Some((
                        "drawing_add_note",
                        serde_json::to_value(note).map_err(|x| x.to_string())?,
                    ));
                } else if let Some(draft) = &mut e.draft {
                    fields::apply(draft, &e.fields)?;
                    if draft.dirty() {
                        request = Some((
                            "drawing_set_document",
                            serde_json::to_value(draft.apply(&e.document)?)
                                .map_err(|x| x.to_string())?,
                        ));
                    } else {
                        e.fields = fields::from_annotation(draft.annotation());
                    }
                }
            }
            Command::Reset => {
                if let Some(id) = e.selected {
                    e.select(id)?;
                } else if e.tool == Some(Tool::Note) {
                    let size = drawing_paper::transform(world.resource::<Workbench>())
                        .ok_or("Open drawing paper")?
                        .sheet_mm;
                    e.fields = fields::note_creation(size.map(|n| n * 0.5));
                }
            }
            Command::Delete => {
                let next = e
                    .draft
                    .as_ref()
                    .ok_or("Select an annotation")?
                    .delete(&e.document)?;
                request = Some((
                    "drawing_set_document",
                    serde_json::to_value(next).map_err(|x| x.to_string())?,
                ));
                e.pending_selected = None;
            }
            Command::Cancel => e.clear(),
            Command::Fields(delta) => e.page = e.page.saturating_add_signed(*delta as isize),
            Command::Field(_) => unreachable!(),
        }
        e.message.clear();
        if let Some((operation, args)) = request {
            return submit(world, handle, engine, bridge, &stamp, operation, args);
        }
        handle.invalidate_presentation();
        Ok(json!({"updated":true}))
    })();
    if let Err(error) = &result {
        e.message = error.clone();
    }
    world.insert_resource(e);
    result
}
