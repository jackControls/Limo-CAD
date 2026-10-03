import { getSessionCamera, subscribeSessionCamera, type ViewportCameraApi } from './components/viewport/cameraApi';
import type { ViewCameraDto } from './engine/types';
import type { AppState } from './store/appStore';
import { armNamedViewCameraRestore } from './namedViewCamera';

export { translateByPartOffset } from './namedViewOffsets';

/** MCP capture data uses model body IDs and independent copies of display state. */
export function inspectNamedViewState(
  state: Pick<AppState, 'solidScene' | 'projectVisibility' | 'viewPartOffsets' | 'activeNamedView' | 'mode'>,
  camera: Pick<ViewportCameraApi, 'getSnapshot'> | null = getSessionCamera(),
) {
  const hidden = new Set(state.projectVisibility.hidden_body_ids);
  const pose = camera?.getSnapshot();
  return {
    camera: pose ? {position: [...pose.position], target: [...pose.target], up: [...pose.up]} : null,
    visible_body_ids: state.solidScene.bodies.filter(body => !hidden.has(body.id)).map(body => body.id),
    part_offsets: state.viewPartOffsets.map(offset => ({body_id: offset.body_id, translation: [...offset.translation]})),
    active_named_view: state.activeNamedView,
    mode: state.mode,
  };
}

/** Restore a saved camera once the modeling viewport is mounted. */
export function restoreNamedViewCamera(camera: ViewCameraDto, isCurrent: () => boolean): void {
  armNamedViewCameraRestore(camera, getSessionCamera, subscribeSessionCamera, isCurrent);
}
