// Exercise the production inbox, Tauri adapter and history controller; only IPC is simulated.
import {undoApplicationHistory, redoApplicationHistory, canUndoApplicationHistory} from './controller';
import {getEngine} from './index';
import {dropApplicationHistory, recordDrawingHistory} from './applicationHistory';
import {canonicalHistoryJson, printHistoryMechanicalModel} from './printIntentHistoryModel';
import {useAppStore} from '../store/appStore';
import {applyInboxNow, publishCurrentSession} from '../sessionBridge';
import {presentation} from '../operationPlayback';
import {createProjectTab, switchProjectTab, installProjectTabRetention} from '../files/projectTabs';
import type {DocumentDto, DrawingDocumentDto} from './types';

export async function checkPrintIntentHistory() {
  const check = (condition: unknown, message: string) => {if (!condition) throw new Error(message);};
  const initial = useAppStore.getState();
  const project = 'print-history-tab';
  const session = 'print-history-session';
  const namespace = 'da89e94d-ce19-449c-80c8-2d247dcf7e6f';
  const emptySettings = () => ({wall_count: null as number | null, infill_density_percent: null, infill_pattern: null,
    top_shell_layers: null, bottom_shell_layers: null});
  const model = {
    format: 'nbcad-project', schema_version: 11,
    document: {name: 'Print history', settings: {units: 'mm'}, history: {features: [
      {id: 1, name: 'Source body', kind: 'extrude', suppressed: false, status: {state: 'ok'}},
    ], rollback_index: 1}},
    drawings: structuredClone(initial.drawingDocument), assembly: structuredClone(initial.assemblyDocument),
    print_intent: {version: 1, source_document_id: null as string | null, selected_process: null,
      defaults: emptySettings(), parts: [], presets: []},
    counters: {extrude: 1}, visibility: structuredClone(initial.projectVisibility),
    views: [], body_appearances: [], cam: structuredClone(initial.camDocument),
  };
  const blankModel = structuredClone(model);
  blankModel.document.history = {features: [], rollback_index: 0};
  blankModel.counters.extrude = 0;
  const nativeModels = new Map<string, typeof model>([[project, structuredClone(model)]]);
  const nativeSessions = new Map([[project, session]]);
  const archives = new Map<string, typeof model>();
  let activeNative = project;
  let incarnations = 0;
  const activate = (id: string, fresh: boolean) => {
    nativeModels.set(activeNative, structuredClone(model));
    if (fresh) {nativeModels.set(id, structuredClone(blankModel)); nativeSessions.set(id, `${session}-${++incarnations}`);}
    Object.assign(model, structuredClone(nativeModels.get(id)!));
    activeNative = id;
  };
  const nativeDocument = (): DocumentDto => ({name: model.document.name, settings: {units: 'mm'},
    features: model.document.history.features as DocumentDto['features'],
    rollback_index: model.document.history.rollback_index, browser: []});
  const scene = {bodies: [], errors: []};
  const update = () => ({document: structuredClone(nativeDocument()), scene});
  const ok = (value: unknown) => JSON.stringify({ok: true, value});
  const calls: string[] = [];
  let queued = true;
  let rejectRestore = false;
  let delayedRestore: {entered(): void; wait: Promise<void>} | null = null;
  let receipt: {id: string; before: typeof model.print_intent; after: typeof model.print_intent; mechanical_model: Record<string, unknown>};
  const w = window as typeof window & {__TAURI_INTERNALS__?: {invoke(command: string, args?: Record<string, unknown>): Promise<unknown>}};
  w.__TAURI_INTERNALS__ = {async invoke(command, args = {}) {
    calls.push(command);
    if (command === 'mcp_session_bridge_reserve') return {session_id: nativeSessions.get(activeNative), project_session_id: activeNative, generation: 1};
    if (command === 'mcp_session_bridge_write') return {skipped: false};
    if (command === 'mcp_session_bridge_apply_inbox') {
      if (!queued) return {applied: false};
      queued = false;
      const before = structuredClone(model.print_intent);
      model.print_intent.source_document_id = namespace;
      model.print_intent.defaults.wall_count = 6;
      before.source_document_id = namespace;
      receipt = {id: 'native-recorded-receipt', before, after: structuredClone(model.print_intent),
        mechanical_model: printHistoryMechanicalModel(structuredClone(model))};
      return {applied: true, name: 'print_intent_set_document', result: model.print_intent, print_intent_history: receipt};
    }
    if (command === 'mcp_session_bridge_restore_print_intent') {
      if (delayedRestore) {const gate = delayedRestore; delayedRestore = null; gate.entered(); await gate.wait;}
      if (rejectRestore) throw new Error('Native owner no longer matches');
      check(args.document === activeNative && args.session === nativeSessions.get(activeNative), 'Restore must carry the owning native session');
      check(args.expectedModelJson === JSON.stringify(model), 'Restore must carry the current full model');
      model.print_intent = structuredClone(args.redo ? receipt.after : receipt.before);
      return model.print_intent;
    }
    if (command === 'mcp_session_bridge_replay_history') {
      check(args.expectedModelJson === JSON.stringify(model), 'Geometry replay must be guarded by the current model');
      const replay = JSON.parse(args.modelJson as string) as typeof model;
      check(canonicalHistoryJson(replay.print_intent) === canonicalHistoryJson(model.print_intent),
        'Geometry replay must retain current print metadata');
      Object.assign(model, replay);
      return ok(update());
    }
    if (command === 'mcp_session_bridge_restore_cold_project') {
      const archived = archives.get(args.document as string);
      check(archived && args.document === activeNative && model.document.history.features.length === 0,
        'Cold restore must target its fresh owning context');
      check(canonicalHistoryJson(archived) === canonicalHistoryJson(JSON.parse(args.modelJson as string)),
        'Cached tab JSON must reflect metadata Undo and match the native archive');
      Object.assign(model, structuredClone(archived));
      nativeModels.set(activeNative, structuredClone(model));
      return {update: update(), document_id: activeNative, session_id: nativeSessions.get(activeNative), receipt_ids: [receipt.id]};
    }
    if (command === 'engine_solid_delete_feature') {
      model.document.history.features.pop();
      model.document.history.rollback_index = model.document.history.features.length;
      return ok(update());
    }
    if (command === 'get_document') return nativeDocument();
    if (command === 'engine_project_export_model') return ok(JSON.stringify(model));
    if (command === 'engine_active_sketch') return ok(null);
    if (command === 'engine_project_session_bind') return ok(null);
    if (command === 'engine_project_session_create') {activate(args.sessionId as string, true); return ok(update());}
    if (command === 'engine_project_session_activate') {
      const id = args.sessionId as string;
      if (!nativeModels.has(id)) return ok(false);
      activate(id, false);
      return ok(true);
    }
    if (command === 'engine_project_session_drop') {
      const id = args.sessionId as string;
      check(id !== activeNative, 'A native drop must release an inactive context');
      if (args.retainHistory) archives.set(id, structuredClone(nativeModels.get(id)!));
      else if (!args.preserveHistoryArchive) archives.delete(id);
      nativeModels.delete(id); nativeSessions.delete(id);
      return ok(null);
    }
    if (command === 'system_memory_status') return {totalBytes: 100, availableBytes: 0, pressure: 'critical'};
    if (command === 'engine_solid_scene') return ok(scene);
    if (['engine_finished_sketches', 'engine_datum_plane_definitions', 'engine_body_appearances'].includes(command)) return ok([]);
    if (command === 'engine_drawing_document') return ok(model.drawings);
    if (command === 'engine_drawing_set_document') {model.drawings = JSON.parse(args.payload as string) as DrawingDocumentDto; return ok(model.drawings);}
    if (command === 'engine_assembly_document') return ok(model.assembly);
    if (command === 'engine_assembly_solution') return ok(initial.assemblySolution);
    if (command === 'engine_cam_document') return ok(model.cam);
    if (command === 'engine_project_visibility') return ok(model.visibility);
    if (command === 'engine_project_set_visibility') {model.visibility = JSON.parse(args.payload as string); return ok(model.visibility);}
    if (['engine_named_views', 'engine_set_named_views', 'engine_clear_named_view'].includes(command)) return ok({views: [], active: null});
    throw new Error(`Unexpected print history command: ${command}`);
  }};
  useAppStore.setState({engineKind: 'tauri', activeProjectTabId: project, activeTab: 'solid',
    projectTabs: [{id: project, name: model.document.name, fileName: null, dirty: false, workspaceTab: 'solid'}],
    mode: 'solid', solidBusy: false, projectBusy: false, historyEdit: null});
  useAppStore.getState().loadProjectState(update(), [], [], null, [], model.drawings, model.assembly,
    model.visibility, initial.assemblySolution, model.cam);
  dropApplicationHistory(project);
  try {
    check(await publishCurrentSession(), 'Inbox fixture must publish the owning session first');
    await applyInboxNow();
    check(model.print_intent.defaults.wall_count === 6, 'Production inbox must apply metadata');
    check(await undoApplicationHistory(), 'The recorded metadata change must Undo');
    check(model.print_intent.defaults.wall_count === null, 'Undo must restore inherited walls');
    check(model.document.history.features.length === 1 && !calls.includes('engine_solid_delete_feature'),
      'Metadata Undo must not delete the source feature');
    check(model.print_intent.source_document_id === namespace, 'First-edit Undo must retain the assigned identity');
    check(await redoApplicationHistory() && model.print_intent.defaults.wall_count === 6, 'Metadata Redo must restore explicit walls');

    model.document.history.features.push({id: 2, name: 'Later CAD feature', kind: 'extrude', suppressed: false, status: {state: 'ok'}});
    model.document.history.rollback_index = 2;
    model.counters.extrude = 2;
    useAppStore.getState().applySolidUpdate(update());
    await new Promise<void>(resolve => queueMicrotask(resolve));
    check(await undoApplicationHistory() && model.document.history.features.length === 1, 'A later CAD edit must Undo first');
    check(model.print_intent.defaults.wall_count === 6, 'CAD Undo must preserve explicit print intent');
    check(await undoApplicationHistory() && model.print_intent.defaults.wall_count === null, 'Metadata must Undo after the later CAD edit');
    check(await redoApplicationHistory() && model.print_intent.defaults.wall_count === 6, 'Metadata Redo must precede later CAD replay');
    check(await redoApplicationHistory() && model.document.history.features.length === 2, 'Later CAD replay must remain available');
    check(model.print_intent.defaults.wall_count === 6 && calls.includes('mcp_session_bridge_replay_history'),
      'Geometry Redo must use the owning replay route and preserve print settings');
    check(!calls.includes('engine_project_load'), 'History must not retire its document through the Open route');

    check(await undoApplicationHistory(), 'Return to the metadata landmark');
    const drawingBefore = structuredClone(model.drawings);
    const drawingAfter = {...drawingBefore, next_view_id: drawingBefore.next_view_id + 1};
    model.drawings = drawingAfter;
    useAppStore.setState({drawingDocument: drawingAfter, activeTab: 'drawing'});
    recordDrawingHistory(project, drawingBefore, drawingAfter);
    check(await undoApplicationHistory(), 'A later drawing command must Undo first');
    check(model.print_intent.defaults.wall_count === 6, 'Drawing Undo must leave print settings intact');
    check(await undoApplicationHistory() && model.print_intent.defaults.wall_count === null, 'Print metadata must Undo after the drawing landmark');
    check(await redoApplicationHistory(), 'Restore the print edit before ownership rejection');
    check(await createProjectTab(), 'Create a neighbour through real tab bookkeeping');
    const neighbour = useAppStore.getState().activeProjectTabId!;
    check(await switchProjectTab(project), 'Return to the populated source tab');
    check(await undoApplicationHistory(), 'Undo metadata after caching the populated tab');
    check(await switchProjectTab(neighbour), 'Snapshot the undone metadata before eviction');
    let retentionTick: (() => void) | undefined;
    const setInterval = window.setInterval;
    window.setInterval = ((handler: TimerHandler) => {retentionTick = handler as () => void; return 0;}) as typeof window.setInterval;
    const uninstallRetention = installProjectTabRetention();
    window.setInterval = setInterval;
    try {
      retentionTick!();
      for (let attempt = 0; attempt < 100 && nativeModels.has(project); attempt++) await new Promise(resolve => setTimeout(resolve, 0));
      for (let attempt = 0; attempt < 100 && useAppStore.getState().solidBusy; attempt++) await new Promise(resolve => setTimeout(resolve, 0));
      check(!nativeModels.has(project), 'The real retention worker must evict the inactive source tab');
    } finally {uninstallRetention();}
    check(await switchProjectTab(project), 'A cold tab must restore its authenticated metadata/history');
    check(model.print_intent.defaults.wall_count === null && model.print_intent.source_document_id === namespace,
      'Cold restore must retain undone settings and the source identity');
    check(await redoApplicationHistory() && model.print_intent.defaults.wall_count === 6,
      'The native-restored receipt must Redo under its new owning session');
    check(nativeSessions.get(project) !== session, 'Cold history must use a new live session rather than reviving the retired one');
    rejectRestore = true;
    const protectedModel = JSON.stringify(model);
    check(!await undoApplicationHistory() && JSON.stringify(model) === protectedModel,
      'Native owner rejection must retain metadata and geometry');
    rejectRestore = false;
    useAppStore.setState({constraintDialog: null});
    let entered!: () => void;
    let release!: () => void;
    const ready = new Promise<void>(resolve => {entered = resolve;});
    delayedRestore = {entered, wait: new Promise<void>(resolve => {release = resolve;})};
    const pending = undoApplicationHistory();
    await ready;
    presentation.documentChanged();
    useAppStore.setState({activeProjectTabId: 'replacement', solidBusy: true, dirty: false});
    release();
    check(!await pending && useAppStore.getState().solidBusy && !useAppStore.getState().dirty,
      'A delayed restore must not publish into or clear the replacement owner');
    useAppStore.setState({activeProjectTabId: project, solidBusy: false, dirty: true, constraintDialog: null});
    await (await getEngine()).activateProjectSession(neighbour);
    await (await getEngine()).dropProjectSession(project);
    const beforeExpiredUndo = JSON.stringify(model);
    check(!canUndoApplicationHistory() && !await undoApplicationHistory(),
      'Evicted native receipts must disable the matching print Undo');
    check(JSON.stringify(model) === beforeExpiredUndo
      && useAppStore.getState().constraintDialog?.message.includes('expired'),
      'Expired receipt Undo must explain unavailability without deleting a CAD feature');
    return {productionInbox: true, metadataUndoRedo: true, identityStable: true,
      geometryChronology: true, geometryReplayOwner: true, drawingChronology: true,
      rejectedOwnerUnchanged: true, replacementGuard: true, coldTabUndoRedo: true,
      cachedMetadataFresh: true, newColdSessionOwner: true, evictionBarrier: true};
  } finally {
    dropApplicationHistory(project);
    useAppStore.setState(initial);
    delete w.__TAURI_INTERNALS__;
  }
}
