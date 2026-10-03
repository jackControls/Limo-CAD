import { getEngine } from './engine';
import { createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { flushSync } from 'react-dom';
import { BrowserTree } from './components/BrowserTree';
import { canRedoApplicationHistory } from './engine/controller';
import { dropApplicationHistory } from './engine/applicationHistory';
import { runNativeEditCommand } from './nativeEditMenu';
import { useAppStore } from './store/appStore';
import type { BodyDto, DocumentDto, NamedViewConfigurationDto, ProjectVisibilityDto } from './engine/types';

/** Same controller regression can run against simulated IPC or a native test bridge. */
export async function checkNamedViewHistory(native = false) {
  const check = (condition: unknown, message: string) => { if (!condition) throw new Error(message); };
  const initial = useAppStore.getState();
  const w = window as typeof window & { __TAURI_INTERNALS__?: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
  const previous = w.__TAURI_INTERNALS__;
  const emptyVisibility: ProjectVisibilityDto = { hidden_body_ids: [], hidden_datum_plane_ids: [], hidden_sketch_names: [] };
  const key = 'named-view-history';
  if (!native) {
    type Model = { document: DocumentDto; views: NamedViewConfigurationDto[]; visibility: ProjectVisibilityDto };
    let model: Model = { document: { name: key, settings: { units: 'mm' }, rollback_index: 2,
      features: [1, 2].map(id => ({ id, name: `Extrude${id}`, kind: 'extrude', suppressed: false, status: { state: 'ok' } })), browser: [] },
      views: [], visibility: emptyVisibility };
    let active: string | null = null;
    const update = () => ({ document: model.document, scene: { bodies: model.document.features.map(feature => ({
      id: feature.id, name: feature.name, feature_id: feature.id, faces: [], edges: [],
      mesh: { positions: [], normals: [], indices: [] },
    } as BodyDto)), errors: [] } });
    w.__TAURI_INTERNALS__ = { async invoke(command, args = {}) {
      const payload = typeof args.payload === 'string' ? JSON.parse(args.payload) : null;
      let value: unknown;
      switch (command) {
        case 'get_document': return model.document;
        case 'engine_document': value = model.document; break;
        case 'engine_solid_scene': value = update().scene; break;
        case 'engine_project_export_model': value = JSON.stringify(model); break;
        case 'engine_project_load': model = JSON.parse(payload); active = null; value = update(); break;
        case 'engine_project_visibility': value = model.visibility; break;
        case 'engine_project_set_visibility': model.visibility = payload; value = model.visibility; break;
        case 'engine_named_views': value = { views: model.views, active }; break;
        case 'engine_set_named_views': model.views = payload.views; active = null; value = { views: model.views, active }; break;
        case 'engine_upsert_named_view':
          model.views = [...model.views.filter(view => view.name !== payload.name), payload];
          active = null; value = { views: model.views, active }; break;
        case 'engine_rename_named_view':
          model.views = model.views.map(view => view.name === payload.name ? {...view, name: payload.new_name} : view);
          active = null; value = { views: model.views, active }; break;
        case 'engine_delete_named_view':
          model.views = model.views.filter(view => view.name !== payload.name);
          active = null; value = { views: model.views, active }; break;
        case 'engine_clear_named_view': active = null; value = { views: model.views, active }; break;
        case 'engine_recall_named_view': {
          const view = model.views.find(view => view.name === payload.name)!;
          active = view.name;
          model.visibility = { ...model.visibility, hidden_body_ids: update().scene.bodies.map(body => body.id).filter(id => !view.visible_body_ids.includes(id)) };
          value = { view, visibility: model.visibility }; break;
        }
        case 'engine_solid_delete_feature':
          model.document = { ...model.document, features: model.document.features.filter(feature => feature.id !== payload.feature_id), rollback_index: model.document.rollback_index - 1 };
          active = null; value = update(); break;
        case 'engine_finished_sketches': case 'engine_datum_plane_definitions': case 'engine_body_appearances': value = []; break;
        case 'engine_drawing_document': value = initial.drawingDocument; break;
        case 'engine_assembly_document': value = initial.assemblyDocument; break;
        case 'engine_assembly_solution': value = initial.assemblySolution; break;
        case 'engine_cam_document': value = initial.camDocument; break;
        default: throw new Error(`Unexpected named-view history command: ${command}`);
      }
      return JSON.stringify({ ok: true, value });
    } };
  }
  try {
    const engine = await getEngine();
    if (native) {
      await engine.newProject();
      for (let index = 0; index < 2; index++) {
        await engine.beginSketch({ type: 'origin_plane', plane: 'xy' });
        await engine.addRectangle({ mode: 'two_point', p1: { x: index * 20, y: 0 }, p2: { x: index * 20 + 10, y: 10 }, ctrl_held: false });
        await engine.endSketch();
        await engine.extrude({ sketch_name: `Sketch${index + 1}`, profile_indices: [0], operation: 'new_body',
          extent: { type: 'distance', distance: 10 }, taper_angle_deg: 0, flip: false, target_body_ids: [] });
      }
    }
    const scene = await engine.solidScene();
    const bodies = scene.bodies.map(body => body.id);
    check(bodies.length === 2, 'History fixture requires two bodies');
    const view = (name: string, visible: number[]): NamedViewConfigurationDto => ({ name,
      camera: { position: [80, -40, 30], target: [0, 0, 8], up: [0, 0, 1] }, visible_body_ids: visible,
      part_offsets: [{ body_id: bodies[0], translation: [0, 14, 0] }] });
    await engine.setNamedViews([view('first', [bodies[0]]), view('second', [bodies[1]])]);
    const document = await engine.getDocument();
    useAppStore.getState().loadProjectState({ document, scene }, await engine.finishedSketches(), [], null,
      [], initial.drawingDocument, initial.assemblyDocument, emptyVisibility, initial.assemblySolution);
    useAppStore.setState({ activeProjectTabId: key, engineKind: 'tauri', dirty: false });
    if (native) {
      const mount = window.document.createElement('div'); window.document.body.append(mount);
      const root = createRoot(mount);
      try {
        flushSync(() => root.render(createElement(BrowserTree)));
        const row = mount.querySelector<HTMLElement>('[data-named-view="second"]');
        check(row, 'The native model must expose its saved view in the rendered Browser');
        row!.click();
        const deadline = Date.now() + 3000;
        while (useAppStore.getState().activeNamedView !== 'second' || useAppStore.getState().solidBusy) {
          if (Date.now() > deadline) throw new Error('Native Browser recall timed out');
          await new Promise(resolve => setTimeout(resolve, 0));
        }
        check(JSON.stringify(await engine.solidScene()) === JSON.stringify(scene), 'Browser recall must not move native geometry');
        check(useAppStore.getState().viewPartOffsets.length === 1, 'Browser recall must publish the display offsets');
      } finally { root.unmount(); mount.remove(); }
    } else await useAppStore.getState().recallNamedView('second');
    check(useAppStore.getState().dirty && useAppStore.getState().document?.features.length === document.features.length,
      'Recall visibility marks the file dirty without adding a feature/history step');
    await runNativeEditCommand('undo');
    check(useAppStore.getState().document?.features.length === document.features.length - 1,
      'Undo after recall must undo the last model feature');
    check(!useAppStore.getState().viewPartOffsets.length && useAppStore.getState().activeNamedView === null,
      'Undo must return to assembled poses');
    check(useAppStore.getState().projectVisibility.hidden_body_ids.includes(bodies[0]), 'Undo must preserve visibility intent');
    await useAppStore.getState().recallNamedView('first');
    await engine.renameNamedView('first', 'updated');
    await useAppStore.getState().refreshAfterInboxApply('rename_named_view');
    check(canRedoApplicationHistory(), 'Renaming a view must preserve feature Redo');
    await engine.upsertNamedView(view('updated', [bodies[0]]));
    await useAppStore.getState().refreshAfterInboxApply('upsert_named_view');
    check(canRedoApplicationHistory(), 'Updating a view must preserve feature Redo');
    const stored = await engine.deleteNamedView('second');
    await useAppStore.getState().refreshAfterInboxApply('delete_named_view');
    await new Promise<void>(resolve => queueMicrotask(resolve));
    check(canRedoApplicationHistory(), 'A saved view edit must not invalidate feature Redo');
    const visibility = useAppStore.getState().projectVisibility;
    await runNativeEditCommand('redo');
    check(useAppStore.getState().document?.features.length === document.features.length && !useAppStore.getState().constraintDialog,
      'Redo must restore the deleted model feature');
    check(JSON.stringify((await engine.namedViews()).views) === JSON.stringify(stored.views), 'Redo must retain later saved-view edits');
    check(JSON.stringify(await engine.projectVisibility()) === JSON.stringify(visibility)
      && JSON.stringify(useAppStore.getState().projectVisibility) === JSON.stringify(visibility),
      'Redo must retain later visibility choices in both engine and UI');
    check((await engine.namedViews()).active == null && !useAppStore.getState().viewPartOffsets.length,
      'Redo must leave both engine and UI in assembled poses');
    await useAppStore.getState().recallNamedView('updated');
    useAppStore.getState().openHoleDialog();
    const { exportProjectModelWithVisibility } = await import('./store/appStore');
    await exportProjectModelWithVisibility(engine);
    check((await engine.namedViews()).active == null, 'Saving an editing project must not retain a dismissed native presentation');
    useAppStore.getState().closeHoleDialog();
    await useAppStore.getState().recallNamedView('updated');
    if (native) {
      await engine.setGroundedBody(bodies[0]);
      await useAppStore.getState().refreshAfterInboxApply('assembly_set_grounded_body');
      check((await engine.namedViews()).active == null, 'Native assembly edits must clear the active marker');
    } else {
      useAppStore.setState(state => ({ assemblyDocument: { ...state.assemblyDocument, next_position_id: state.assemblyDocument.next_position_id + 1 } }));
    }
    check(useAppStore.getState().activeNamedView === null && !useAppStore.getState().viewPartOffsets.length,
      'Assembly mutations must dismiss the view');
    return { undo: 'passed', redo: 'passed', laterViewEdits: 'passed', visibility: 'passed', assembledReset: 'passed', native };
  } finally {
    dropApplicationHistory(key);
    if (previous) w.__TAURI_INTERNALS__ = previous; else delete w.__TAURI_INTERNALS__;
    useAppStore.setState(initial);
  }
}
