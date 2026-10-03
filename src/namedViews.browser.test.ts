import { useAppStore } from './store/appStore';
import { pendingEngineOperations } from './engine/activity';
import type { DocumentDto, RecallNamedViewDto } from './engine/types';
import { projectTransitions } from './files/projectTransitions';
import { collectAppViewportPickFeedback } from './modeling/viewportPickFeedback';
import { registerSessionCamera, unregisterSessionCamera, type ViewportCameraApi } from './components/viewport/cameraApi';

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}

/** Exercise real store/adapter publication with controllable native replies. */
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
  });
  w.__TAURI_INTERNALS__ = { async invoke(command, args = {}) {
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

    // A queued camera must not survive replacing the project while unmounted.
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
    return { ownership: 'passed', serialization: 'passed', camera: 'passed', pointFeedback: 'passed' };
  } finally {
    if (previous) w.__TAURI_INTERNALS__ = previous; else delete w.__TAURI_INTERNALS__;
    useAppStore.setState(initial);
  }
}
