import { getSessionCamera, subscribeSessionCamera } from './components/viewport/cameraApi';
import type { ViewCameraDto } from './engine/types';

export { translateByPartOffset } from './namedViewOffsets';

/** Restore a saved camera once the modeling viewport is mounted. */
export function restoreNamedViewCamera(camera: ViewCameraDto): void {
  const apply = () => {
    const api = getSessionCamera();
    if (!api) return false;
    api.restore(camera);
    return true;
  };
  if (apply()) return;
  const stop = subscribeSessionCamera(() => {
    if (!apply()) return;
    stop();
  });
}
