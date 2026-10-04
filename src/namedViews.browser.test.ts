import { useAppStore } from './store/appStore';
import { createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { flushSync } from 'react-dom';
import { BrowserTree } from './components/BrowserTree';
import { pendingEngineOperations } from './engine/activity';
import { HoleDialog } from './components/HoleDialog';
import { modelPointFromDisplay } from './namedViewOffsets';
import type { BodyDto, DocumentDto, RecallNamedViewDto } from './engine/types';
import { projectTransitions } from './files/projectTransitions';
import { collectAppViewportPickFeedback } from './modeling/viewportPickFeedback';
import { registerSessionCamera, unregisterSessionCamera, type ViewportCameraApi } from './components/viewport/cameraApi';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}

export async function checkNamedViewOwnership() {
  const check = (condition: unknown, message: string) => { if (!condition) throw new Error(message); };
  const settled = async (promise: Promise<unknown>) => {
    try { await promise; return ''; } catch (error) { return String(error); }
  };
  const initial = useAppStore.getState();
  const w = window as typeof window & { __TAURI_INTERNALS__?: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
  const previous = w.__TAURI_INTERNALS__;
  let gate: { started: ReturnType<typeof deferred<void>>; reply: ReturnType<typeof deferred<void>> } | undefined;
  const writes: string[] = [];
  const result = (name: string): RecallNamedViewDto => ({
    view: { name, camera: { position: [10, 0, 0], target: [0, 0, 0], up: [0, 0, 1] },
      visible_body_ids: [], part_offsets: [{ body_id: 1, translation: [0, 14, 0] }] },
    visibility: { hidden_body_ids: name === 'second' ? [2] : [1], hidden_datum_plane_ids: [], hidden_sketch_names: [] },
    solution: structuredClone(initial.assemblySolution),
  });
  w.__TAURI_INTERNALS__ = { async invoke(command, args = {}) {
    if (command === 'engine_clear_named_view') {
      writes.push('clear');
      return JSON.stringify({ ok: true, value: { views: [result('first').view], active: null } });
    }
    if (command === 'engine_assembly_document') return JSON.stringify({ ok: true, value: initial.assemblyDocument });
    if (command === 'engine_assembly_solution') return JSON.stringify({ ok: true, value: initial.assemblySolution });
    if (command !== 'engine_recall_named_view') throw new Error(`Unexpected command ${command}`);
    const name = JSON.parse(args.payload as string).name as string;
    writes.push(name);
    const hold = gate; gate = undefined;
    if (hold) { hold.started.resolve(); await hold.reply.promise; }
    if (name === 'missing') return JSON.stringify({ ok: false, error: 'Named view missing' });
    return JSON.stringify({ ok: true, value: result(name) });
  } };
  const replace = (name: string, sameTab = false) => {
    const document: DocumentDto = { name, settings: { units: 'mm' }, features: [], rollback_index: 0, browser: [] };
    useAppStore.getState().loadProjectState({ document, scene: { bodies: [], errors: [] } }, [], [], null);
    useAppStore.setState({ activeProjectTabId: sameTab ? 'A' : name, engineKind: 'tauri', dirty: false });
  };
  const idle = () => check(!pendingEngineOperations() && projectTransitions.isSettled()
    && !useAppStore.getState().solidBusy, 'Recall must release busy and project fences');
  try {
    for (const sameTab of [false, true]) {
      replace('A');
      const count = writes.length;
      const pending = settled(useAppStore.getState().recallNamedView('first'));
      replace('B', sameTab);
      check(/document changed/i.test(await pending) && writes.length === count,
        'Open/switch before dispatch must prevent mutation of the replacement model');
      idle();

      replace('A');
      const hold = { started: deferred<void>(), reply: deferred<void>() }; gate = hold;
      const late = useAppStore.getState().recallNamedView('first');
      await hold.started.promise;
      replace('B', sameTab);
      hold.reply.resolve();
      await late;
      const state = useAppStore.getState();
      check(!state.dirty && state.activeNamedView === null && !state.viewPartOffsets.length,
        'A late reply must not publish offsets or visibility to another model in the same tab');
      idle();
    }

    replace('A');
    const hold = { started: deferred<void>(), reply: deferred<void>() }; gate = hold;
    const recall = useAppStore.getState().recallNamedView('first');
    await hold.started.promise;
    const transition = projectTransitions.begin();
    let published = false;
    const unsubscribe = useAppStore.subscribe(state => { if (state.activeNamedView === 'first') published = true; });
    const replacement = transition.waitForSnapshots().then(() => {
      check(published, 'A waiting replacement must allow recall publication in its owning model');
      replace('B'); transition();
    });
    hold.reply.resolve();
    await Promise.all([recall, replacement]); unsubscribe(); idle();

    replace('A');
    const serial = { started: deferred<void>(), reply: deferred<void>() }; gate = serial;
    const first = useAppStore.getState().recallNamedView('first');
    await serial.started.promise;
    const second = useAppStore.getState().recallNamedView('second');
    const count = writes.length;
    await Promise.resolve();
    check(writes.length === count, 'Rapid recalls must serialize native mutation');
    serial.reply.resolve();
    await Promise.all([first, second]);
    check(useAppStore.getState().activeNamedView === 'second', 'The newest recall owns visibility and offsets');
    idle();

    const good = useAppStore.getState().recallNamedView('first');
    const missing = settled(useAppStore.getState().recallNamedView('missing'));
    await good;
    check((await missing).includes('Named view missing') && useAppStore.getState().activeNamedView === 'first'
      && JSON.stringify(useAppStore.getState().projectVisibility) === JSON.stringify(result('first').visibility),
      'A failed rapid successor must preserve the last successfully applied native visibility');
    idle();

    replace('B', true);
    let restores = 0;
    const camera = { restore: () => { restores++; }, getSnapshot: () => result('first').view.camera } as unknown as ViewportCameraApi;
    registerSessionCamera(camera); unregisterSessionCamera(camera);
    check(restores === 0, 'Replaced project must never receive the previous view camera');

    useAppStore.setState({ selectedBody: 1, selectedFacePoint: { x: 1, y: 2, z: 3 },
      viewPartOffsets: [{ body_id: 1, translation: [0, 14, 0] }], modelingPickTarget: 'move_from' });
    const feedback = collectAppViewportPickFeedback(useAppStore.getState());
    check(JSON.stringify(feedback.selectedSurfacePoint) === JSON.stringify({ x: 1, y: 16, z: 3 }),
      'Selected point feedback must use display coordinates while the store keeps modeling coordinates');

    const waitFor = async (condition: () => boolean) => {
      const deadline = Date.now() + 3000;
      while (!condition()) {
        if (Date.now() > deadline) throw new Error('Browser view action timed out');
        await new Promise(resolve => setTimeout(resolve, 0));
      }
    };
    replace('A');
    const document = useAppStore.getState().document!;
    useAppStore.setState({ document: { ...document, browser: [{ id: 1, kind: 'named_views', name: null,
      reference_id: null, visible: true, children: [{ id: 2, kind: 'named_view', name: 'first',
        reference_id: null, visible: true, children: [] }] }] } });
    const container = window.document.createElement('div'); window.document.body.append(container);
    const root = createRoot(container);
    try {
      flushSync(() => root.render(createElement(BrowserTree)));
      container.querySelector<HTMLElement>('[data-named-view="first"]')!.click();
      await waitFor(() => useAppStore.getState().activeNamedView === 'first' && !useAppStore.getState().solidBusy);
      await waitFor(() => container.querySelector('[data-testid="clear-named-view"]') !== null);
      useAppStore.getState().markClean();
      const visibility = useAppStore.getState().projectVisibility;
      container.querySelector<HTMLButtonElement>('[data-testid="clear-named-view"]')!.click();
      await waitFor(() => useAppStore.getState().activeNamedView === null && !useAppStore.getState().solidBusy);
      check(writes[writes.length - 1] === 'clear' && !useAppStore.getState().viewPartOffsets.length
        && useAppStore.getState().projectVisibility === visibility && !useAppStore.getState().dirty,
        'The Browser assembled-view action must clear offsets through the adapter without editing visibility or the model');
    } finally { root.unmount(); container.remove(); }

    for (const edit of [() => useAppStore.getState().setMode('pickPlane'),
      () => useAppStore.getState().openHoleDialog(),
      () => useAppStore.getState().openBodyFeatureDialog('move_copy'),
      () => useAppStore.getState().applySolidUpdate({ document: document, scene: { bodies: [], errors: [] } })]) {
      replace('A');
      await useAppStore.getState().recallNamedView('first');
      edit();
      check(!useAppStore.getState().viewPartOffsets.length && useAppStore.getState().activeNamedView === null,
        'Entering editing or publishing a changed solid must return to the assembled pose');
    }
    return { ownership: 'passed', serialization: 'passed', camera: 'passed', pointFeedback: 'passed',
      browserRecallAndClear: 'passed', editResets: 'passed' };
  } finally {
    if (previous) w.__TAURI_INTERNALS__ = previous; else delete w.__TAURI_INTERNALS__;
    useAppStore.setState(initial);
  }
}


export async function checkNamedLayoutHolePlacement() {
  const check = (condition: unknown, message: string) => { if (!condition) throw new Error(message); };
  const initial = useAppStore.getState();
  const w = window as typeof window & { __TAURI_INTERNALS__?: { invoke(command: string): Promise<unknown> } };
  const previous = w.__TAURI_INTERNALS__;
  w.__TAURI_INTERNALS__ = { async invoke(command) {
    if (command !== 'engine_hole_definitions') throw new Error('Unexpected hole command ' + command);
    return JSON.stringify({ ok: true, value: [] });
  } };
  const body: BodyDto = { id: 1, name: 'Repeated part', feature_id: 1,
    mesh: { positions: [0,0,5, 30,0,5, 0,30,5], normals: [], indices: [0,1,2] }, edges: [],
    faces: [{ id: 11, key: 'top', first_index: 0, index_count: 3,
      plane: { origin: [0,0,5], u: [1,0,0], v: [0,1,0], normal: [0,0,1] } }] };
  const half = Math.SQRT1_2;
  const instances = [
    { occurrence_id: 1, component_id: 1, body_id: 1, translation: [100,0,0] as [number,number,number], rotation: [0,0,0,1] as [number,number,number,number], visible: true },
    { occurrence_id: 2, component_id: 1, body_id: 1, translation: [200,0,0] as [number,number,number], rotation: [0,0,half,half] as [number,number,number,number], visible: true },
  ];
  const container = document.createElement('div'); document.body.append(container);
  const root = createRoot(container);
  const waitForPreview = async (u: number, v: number) => {
    const deadline = Date.now() + 3000;
    while (true) {
      const preview = useAppStore.getState().solidCommandPreview;
      if (preview?.kind === 'hole' && preview.bodyId === 1
          && preview.positions[0]?.x === u && preview.positions[0]?.y === v) return;
      if (Date.now() > deadline) throw new Error('Hole preview did not use source-face UV (' + u + ', ' + v + '): ' + JSON.stringify(preview));
      await new Promise(resolve => setTimeout(resolve, 0));
    }
  };
  try {
    for (const [occurrenceId, hit] of [[1, { x:106,y:9,z:5 }], [2, { x:191,y:6,z:5 }]] as const) {
      useAppStore.setState({ mode:'solid', engineKind:'tauri', solidBusy:false, projectBusy:false,
        holeDialogFeature:null, solidCommandPreview:null, solidScene:{ bodies:[body], errors:[] },
        selectedOccurrenceId:occurrenceId, activeNamedView:'Print', viewPartOffsets:[],
        viewAssemblySolution:{ ...initial.assemblySolution, instance_body_poses:instances } });
      const state = useAppStore.getState();
      state.selectSolidFeature('face', 1, 11, modelPointFromDisplay(hit, 1, state.viewPartOffsets));
      state.openHoleDialog();
      const editing = useAppStore.getState();
      check(editing.selectedBody === 1 && editing.selectedFace === 11
        && editing.selectedOccurrenceId === occurrenceId, 'Leaving a layout must retain stable body, face and repeated occurrence selection');
      check(editing.selectedFacePoint === null && editing.activeNamedView === null,
        'Layout-space face points must be discarded when returning to modeling');
      flushSync(() => root.render(createElement(HoleDialog)));
      await waitForPreview(10, 10);
      flushSync(() => root.render(null));
      useAppStore.getState().closeHoleDialog();
    }
    useAppStore.setState({ activeNamedView:null, viewPartOffsets:[], viewAssemblySolution:null,
      holeDialogFeature:null, solidCommandPreview:null });
    useAppStore.getState().selectSolidFeature('face', 1, 11, { x:6,y:9,z:5 });
    useAppStore.getState().openHoleDialog();
    check(useAppStore.getState().selectedFacePoint?.x === 6,
      'Ordinary assembled picks must remain usable when opening a modeling command');
    flushSync(() => root.render(createElement(HoleDialog)));
    await waitForPreview(6, 9);
    return { translatedAndRotatedRepeats:'passed', sourceFaceFallback:'passed', assembledPickPreserved:'passed' };
  } finally {
    root.unmount(); container.remove();
    if (previous) w.__TAURI_INTERNALS__ = previous; else delete w.__TAURI_INTERNALS__;
    useAppStore.setState(initial);
  }
}
