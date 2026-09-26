//! Cached native presentation of the shared CAM planner and stock simulator.
//! Jobs consume immutable document snapshots; their results cannot cross an
//! owner/revision/selection boundary. They never mutate machining intent.
use super::*;
use crate::native_viewport::{
    self, ViewportCamStock, ViewportCamTool, ViewportLineLayer, ViewportPresentation,
    ViewportPreview,
};
use nbcad_cam::{
    CamDocumentDto, CamResolvedStockDto, CamSetupDto, CamSimulationCancellation,
    CamSimulationRequestDto, CamSimulationResultDto, CamSimulationTargetDto, CamStockMeshDto,
};
use std::sync::mpsc;

mod geometry;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum View {
    Model,
    #[default]
    Stock,
    Compare,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    View(View),
    Paths,
    Simulate,
    Cancel,
}

#[derive(Clone, PartialEq, Eq)]
struct Key {
    owner: DocumentContext,
    revision: u64,
    selection: Option<cam::Selection>,
}
struct Prepared {
    paths: Vec<ViewportLineLayer>,
    tool: Option<ViewportCamTool>,
    simulation: Option<CamSimulationResultDto>,
    stock: Option<ViewportCamStock>,
    message: String,
}
struct Pending {
    key: Key,
    generation: u64,
    cancellation: CamSimulationCancellation,
    receiver: Mutex<mpsc::Receiver<Result<Prepared, String>>>,
}
struct Applied {
    owner: DocumentContext,
    preview_revision: u64,
    before_preview: ViewportPreview,
    before_presentation: ViewportPresentation,
    after_presentation: ViewportPresentation,
    before_stock: Option<ViewportCamStock>,
    stock_revision: u64,
}
#[derive(Resource, Default)]
struct State {
    key: Option<Key>,
    document: Option<CamDocumentDto>,
    setup: Option<u64>,
    operation: Option<u64>,
    warning: Option<String>,
    pending: Option<Pending>,
    prepared: Option<Prepared>,
    generation: u64,
    simulation_requested: bool,
    request_pending: bool,
    dirty: bool,
    view: View,
    paths: bool,
    error: String,
    applied: Option<Applied>,
    widgets: Widgets,
}

pub(crate) fn caption(world: &World) -> Option<String> {
    let state = world.get_resource::<State>()?;
    state.key.as_ref()?;
    let entity = state.widgets.entity("cam-view-status")?;
    world.get::<Text>(entity).map(|text| text.0.clone())
}

pub(super) fn execute(world: &mut World, command: &Command) -> Result<Value, String> {
    if matches!(command, Command::Simulate) && cam::editing_dirty(world) {
        return Err("Apply or cancel the CAM edits before simulating".into());
    }
    let mut state = world
        .get_resource_mut::<State>()
        .ok_or("Open the CAM workspace")?;
    match command {
        Command::View(view) => state.view = *view,
        Command::Paths => state.paths = !state.paths,
        Command::Simulate => {
            if state.setup.is_none() {
                return Err("Choose a setup to simulate".into());
            }
            if state.pending.is_some() {
                return Err("Wait for the current CAM preview or cancel it".into());
            }
            state.generation = state.generation.wrapping_add(1);
            state.simulation_requested = true;
            state.request_pending = true;
            state.error.clear();
        }
        Command::Cancel => {
            if let Some(pending) = &state.pending {
                pending.cancellation.cancel();
            }
            state.generation = state.generation.wrapping_add(1);
            state.request_pending = false;
            state.simulation_requested = false;
            state.error.clear();
        }
    }
    state.dirty = true;
    Ok(json!({"handled":true}))
}

fn selection(
    document: &CamDocumentDto,
    selected: Option<cam::Selection>,
) -> (Option<u64>, Option<u64>) {
    match selected {
        Some(cam::Selection::Setup(id)) => (document.setup(id).map(|s| s.id), None),
        Some(cam::Selection::Operation(id)) => (
            document
                .setups
                .iter()
                .find(|s| s.operations.iter().any(|op| op.id() == id))
                .map(|s| s.id),
            Some(id),
        ),
        _ => (
            document
                .active_setup_id
                .or_else(|| document.setups.first().map(|s| s.id)),
            None,
        ),
    }
}

fn simulation_request(
    world: &World,
    document: &CamDocumentDto,
    setup: &CamSetupDto,
    operation: Option<u64>,
) -> Result<CamSimulationRequestDto, String> {
    let scene = native_viewport::interface_geometry(world).scene;
    let mesh = |id| -> Result<CamStockMeshDto, String> {
        let body = scene
            .bodies
            .iter()
            .find(|b| b.id.0 == id)
            .ok_or("CAM body was removed")?;
        Ok(CamStockMeshDto {
            positions: body.mesh.positions.iter().map(|p| f64::from(*p)).collect(),
            indices: body.mesh.indices.clone(),
        })
    };
    let stock_id = stock_body(document, setup)?;
    let meshes = setup
        .body_ids
        .iter()
        .filter(|id| Some(id.0) != stock_id)
        .map(|id| mesh(id.0))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CamSimulationRequestDto {
        setup_id: setup.id,
        voxel_size: None,
        max_voxels: None,
        stock_mesh: stock_id.map(mesh).transpose()?,
        target: (!meshes.is_empty()).then_some(CamSimulationTargetDto {
            cache_key: None,
            meshes,
            tolerance_mm: 0.1,
        }),
        through_operation_id: operation,
        completed_steps: None,
        playback_time_seconds: None,
    })
}

fn stock_body(document: &CamDocumentDto, setup: &CamSetupDto) -> Result<Option<u64>, String> {
    let mut current = setup;
    for _ in 0..=document.setups.len() {
        match current.resolved_stock {
            CamResolvedStockDto::ModelBody { body_id } => return Ok(Some(body_id)),
            CamResolvedStockDto::Rest { source_setup_id } => {
                current = document
                    .setup(source_setup_id)
                    .ok_or("Rest-stock source was removed")?;
            }
            _ => return Ok(None),
        }
    }
    Err("Rest-stock sources contain a cycle".into())
}

fn prepare(
    document: &CamDocumentDto,
    setup_id: u64,
    operation: Option<u64>,
    request: Option<CamSimulationRequestDto>,
    cancellation: &CamSimulationCancellation,
    warning: Option<String>,
) -> Result<Prepared, String> {
    let setup = document.setup(setup_id).ok_or("CAM setup was removed")?;
    let program = match operation {
        Some(operation) => nbcad_cam::plan_setup_through(document, setup_id, operation),
        None => nbcad_cam::plan_setup(document, setup_id),
    }
    .map_err(|e| e.to_string())?;
    let (paths, tool) = geometry::paths(document, setup, &program, operation)?;
    let mut warnings = program.warnings.clone();
    if let Some(warning) = warning {
        warnings.insert(0, warning);
    }
    let mut simulation = request
        .map(|request| {
            nbcad_cam::simulate_setup_with_cancellation(document, &request, Some(cancellation))
                .map_err(|e| e.to_string())
        })
        .transpose()?;
    let stock = simulation.as_ref().and_then(crate::retained_cam_stock);
    let message = if let Some(result) = &mut simulation {
        // The immutable mesh goes directly to Bevy; metadata stays small.
        result.stock_mesh = None;
        result.native_stock_present = stock.is_some();
        warnings.extend(result.warnings.iter().cloned());
        format!(
            "Simulation: {:.1} s · {} contacts · {:.1} mm³ removed · voxel {:.3} mm{}",
            result.estimated_seconds,
            result.collisions.len(),
            result.removed_volume_mm3,
            result.cell_size[0],
            result
                .comparison
                .as_ref()
                .map_or(String::new(), |c| format!(
                    " · gouge {:.1} mm³",
                    c.gouged_volume_mm3
                ))
        )
    } else {
        format!(
            "Toolpath preview · {} operations · {:.1} s",
            program.stats.operation_count, program.stats.estimated_seconds
        )
    };
    let message = if warnings.is_empty() {
        message
    } else {
        format!("{message} · {}", warnings.join(" · "))
    };
    Ok(Prepared {
        paths,
        tool,
        simulation,
        stock,
        message,
    })
}

fn restore(world: &mut World, services: &NativeServices, state: &mut State) -> Result<(), String> {
    let Some(applied) = state.applied.take() else {
        return Ok(());
    };
    let current_owner = services
        .bridge
        .native_document_context(&applied.owner.window_id, &services.engine)?;
    if current_owner != applied.owner {
        return Ok(());
    }
    services
        .bridge
        .with_native_document_owner(&services.engine, &applied.owner, || {
            let (session, _, mut presentation, _) = native_viewport::interface_view_snapshot(world);
            if session != applied.owner.document_id {
                return Ok(());
            }
            if native_viewport::interface_preview_revision(world) == applied.preview_revision {
                native_viewport::apply_interface_preview(world, &session, applied.before_preview)?;
            }
            if presentation.hidden_body_ids == applied.after_presentation.hidden_body_ids {
                presentation.hidden_body_ids = applied.before_presentation.hidden_body_ids;
            }
            if presentation.ghosted_body_ids == applied.after_presentation.ghosted_body_ids {
                presentation.ghosted_body_ids = applied.before_presentation.ghosted_body_ids;
            }
            if presentation.cam_tool == applied.after_presentation.cam_tool {
                presentation.cam_tool = applied.before_presentation.cam_tool;
            }
            let (stock_revision, _) = native_viewport::interface_cam_stock_snapshot(world);
            if stock_revision == applied.stock_revision {
                if presentation.cam_stock_visible == applied.after_presentation.cam_stock_visible {
                    presentation.cam_stock_visible = applied.before_presentation.cam_stock_visible;
                }
                native_viewport::apply_interface_cam_stock(world, &session, applied.before_stock)?;
            }
            if presentation.cam_path_progress == applied.after_presentation.cam_path_progress {
                presentation.cam_path_progress = applied.before_presentation.cam_path_progress;
            }
            native_viewport::apply_interface_view(world, &session, None, Some(presentation))
        })
}

fn display(world: &mut World, services: &NativeServices, state: &mut State) -> Result<(), String> {
    restore(world, services, state)?;
    let Some(key) = &state.key else { return Ok(()) };
    let Some(setup) = state
        .document
        .as_ref()
        .and_then(|d| state.setup.and_then(|id| d.setup(id)))
    else {
        return Ok(());
    };
    let simulation = state.prepared.as_ref().and_then(|p| p.simulation.as_ref());
    let mut preview = geometry::stock(setup, simulation.is_none() && state.view != View::Model);
    if state.view == View::Model {
        preview.lines.clear();
    }
    if state.paths {
        if let Some(prepared) = &state.prepared {
            preview.lines.extend(prepared.paths.iter().cloned());
        }
    }
    services
        .bridge
        .with_native_document_receipt(&services.engine, &key.owner, |revision| {
            if revision != key.revision {
                return Err("CAM document changed during preview preparation".into());
            }
            let (session, _, before, _) = native_viewport::interface_view_snapshot(world);
            if session != key.owner.document_id {
                return Err("CAM viewport document changed".into());
            }
            let before_preview = native_viewport::interface_preview_snapshot(world);
            let (_, before_stock) = native_viewport::interface_cam_stock_snapshot(world);
            let mut presentation = before.clone();
            presentation.cam_tool = state
                .paths
                .then(|| state.prepared.as_ref().and_then(|p| p.tool))
                .flatten();
            presentation.cam_path_progress = None;
            presentation.cam_stock_visible = state.view != View::Model && simulation.is_some();
            if presentation.cam_stock_visible {
                for id in &setup.body_ids {
                    let list = if state.view == View::Compare {
                        &mut presentation.ghosted_body_ids
                    } else {
                        &mut presentation.hidden_body_ids
                    };
                    if !list.contains(&id.0) {
                        list.push(id.0);
                    }
                }
            }
            if let Some(body_id) = stock_body(state.document.as_ref().unwrap(), setup)? {
                if (state.view == View::Model || presentation.cam_stock_visible)
                    && !presentation.hidden_body_ids.contains(&body_id)
                {
                    presentation.hidden_body_ids.push(body_id);
                }
            }
            native_viewport::apply_interface_preview(world, &session, preview)?;
            native_viewport::apply_interface_cam_stock(
                world,
                &session,
                state.prepared.as_ref().and_then(|p| p.stock.clone()),
            )?;
            native_viewport::apply_interface_view(
                world,
                &session,
                None,
                Some(presentation.clone()),
            )?;
            state.applied = Some(Applied {
                owner: key.owner.clone(),
                preview_revision: native_viewport::interface_preview_revision(world),
                before_preview,
                before_presentation: before,
                after_presentation: presentation,
                before_stock,
                stock_revision: native_viewport::interface_cam_stock_snapshot(world).0,
            });
            Ok(())
        })
}

pub(super) fn synchronize(
    world: &mut World,
    camera: Entity,
    services: &NativeServices,
    owner: &DocumentContext,
    width: f32,
    side: f32,
    active: bool,
) -> Result<(), String> {
    let mut state = world.remove_resource::<State>().unwrap_or_default();
    state.widgets.begin();
    let result: Result<(), String> = (|| {
        if !active {
            if let Some(pending) = &state.pending {
                pending.cancellation.cancel();
            }
            restore(world, services, &mut state)?;
            state.key = None;
            state.prepared = None;
            return Ok(());
        }
        let receipt = services
            .bridge
            .native_document_receipt(&services.engine, owner)?;
        let key = Key {
            owner: owner.clone(),
            revision: receipt.revision,
            selection: cam::selected(world),
        };
        if state.key.as_ref() != Some(&key) {
            restore(world, services, &mut state)?;
            if let Some(pending) = &state.pending {
                pending.cancellation.cancel();
            }
            let (document, setup, operation, warning) = services
                .bridge
                .with_native_document_receipt(&services.engine, owner, |revision| {
                    if revision != key.revision {
                        return Err("CAM document changed while preparing preview".into());
                    }
                    let document = services.engine.cam_document_snapshot();
                    let (setup, operation) = selection(&document, key.selection);
                    let warning = setup
                        .map(|id| {
                            services
                                .engine
                                .cam_snapshot(id)
                                .map(|(_, _, warning)| warning)
                        })
                        .transpose()?
                        .flatten();
                    Ok((document, setup, operation, warning))
                })?;
            state.warning = warning;
            if state
                .key
                .as_ref()
                .is_none_or(|prior| prior.owner != key.owner)
            {
                state.view = View::Stock;
                state.paths = true;
            }
            state.key = Some(key.clone());
            state.document = Some(document);
            state.setup = setup;
            state.operation = operation;
            state.prepared = None;
            state.error.clear();
            state.simulation_requested = false;
            state.request_pending = setup.is_some();
            state.dirty = true;
            state.generation = state.generation.wrapping_add(1);
        }
        let completed = state.pending.as_ref().and_then(|pending| {
            match pending.receiver.lock().unwrap().try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("CAM preview worker stopped".into()))
                }
                Err(mpsc::TryRecvError::Empty) => None,
            }
        });
        if let Some(result) = completed {
            let pending = state.pending.take().unwrap();
            if state.key.as_ref() == Some(&pending.key) && state.generation == pending.generation {
                match result {
                    Ok(prepared) => state.prepared = Some(prepared),
                    Err(error) => state.error = error,
                }
                state.dirty = true;
            }
        }
        if state.request_pending && state.pending.is_none() {
            // One explicit request gets one attempt. Invalid geometry or a
            // failed thread launch must not retry on every render frame.
            state.request_pending = false;
            let document = state.document.as_ref().unwrap().clone();
            let setup_id = state.setup.ok_or("Choose a CAM setup")?;
            let operation = state.operation;
            let request = state
                .simulation_requested
                .then(|| {
                    simulation_request(
                        world,
                        &document,
                        document.setup(setup_id).unwrap(),
                        operation,
                    )
                })
                .transpose()?;
            let cancellation = CamSimulationCancellation::default();
            let cancel = cancellation.clone();
            let warning = state.warning.clone();
            let (send, receiver) = mpsc::channel();
            let wake = world.get_resource::<NativeInterfaceHandle>().cloned();
            std::thread::Builder::new()
                .name("cad-native-cam-view".into())
                .spawn(move || {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        prepare(&document, setup_id, operation, request, &cancel, warning)
                    }))
                    .unwrap_or_else(|_| Err("CAM preview worker stopped unexpectedly".into()));
                    let _ = send.send(result);
                    if let Some(handle) = wake {
                        handle.request_redraw();
                    }
                })
                .map_err(|e| e.to_string())?;
            state.pending = Some(Pending {
                key,
                generation: state.generation,
                cancellation,
                receiver: Mutex::new(receiver),
            });
            state.request_pending = false;
        }
        if state.dirty {
            display(world, services, &mut state)?;
            state.dirty = false;
        }
        let available = state.setup.is_some();
        let pending = state.pending.is_some();
        let x = (side + 12.).max(270.);
        let available_width = (width - x - 96.).max(100.);
        let columns = ((available_width / 100.).floor() as usize).clamp(1, 6);
        let cell_width = (available_width / columns as f32).min(110.);
        for (i, (label, command, selected, disabled)) in [
            (
                "Model",
                Command::View(View::Model),
                state.view == View::Model,
                !available,
            ),
            (
                "Stock",
                Command::View(View::Stock),
                state.view == View::Stock,
                !available,
            ),
            (
                "Compare",
                Command::View(View::Compare),
                state.view == View::Compare,
                !available,
            ),
            ("Show toolpaths", Command::Paths, state.paths, !available),
            (
                "Simulate",
                Command::Simulate,
                false,
                !available || pending || cam::editing_dirty(world),
            ),
            ("Cancel simulation", Command::Cancel, false, !pending),
        ]
        .into_iter()
        .enumerate()
        {
            let mut control = InterfaceControl::button("cam/view", label);
            control.disabled = disabled;
            control.selected = Some(selected);
            state.widgets.button(
                world,
                camera,
                &format!("cam-view-{i}"),
                control,
                Some(label),
                NativeCommand::Workbench(super::Command::CamView(command)),
                rect(
                    x + (i % columns) as f32 * cell_width,
                    128. + (i / columns) as f32 * 32.,
                    cell_width - 4.,
                    28.,
                ),
                None,
                45,
            )?;
        }
        let message = if !state.error.is_empty() {
            state.error.as_str()
        } else if pending {
            if state.simulation_requested {
                "Simulating applied CAM document…"
            } else {
                "Preparing CAM preview…"
            }
        } else {
            state
                .prepared
                .as_ref()
                .map_or("Choose a setup or toolpath", |p| p.message.as_str())
        };
        state.widgets.text(
            world,
            camera,
            "cam-view-status",
            rect(
                (side + 12.).max(270.),
                132. + 6_usize.div_ceil(columns) as f32 * 32.,
                (width - side - 24.).max(0.),
                48.,
            ),
            message,
            12.,
            45,
        );
        if let Some(entity) = state.widgets.entity("cam-view-status") {
            world
                .entity_mut(entity)
                .insert(TextColor(if state.error.is_empty() {
                    ViewportUiTheme::from_palette(&ViewportPalette::default()).ink
                } else {
                    Color::srgb(0.95, 0.35, 0.3)
                }));
        }
        Ok(())
    })();
    if let Err(error) = &result {
        state.error = error.clone();
    }
    state.widgets.finish(world);
    world.insert_resource(state);
    result
}
