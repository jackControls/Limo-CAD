import { useEffect, useState } from 'react';
import type { CamHeightGeometryDto, CamOperationHeightExpressionsDto, CamSetupDto } from '../../engine/types';
import { heightGeometryCandidates, heightGeometryLabel, resolveHeightGeometry } from '../../cam/heightGeometry';
import { cancelCamPointPick, requestCamPointPick } from '../../cam/pointPick';
import { translate } from '../../i18n';
import { useAppStore } from '../../store/appStore';
import type { HeightFrom } from './camOperationFields';

export type HeightKey = 'bottom' | 'top' | 'feed' | 'retract' | 'clearance';
export function useCamHeightGeometry(setup: CamSetupDto | null | undefined, stored: CamOperationHeightExpressionsDto | null | undefined) {
  const picking = useAppStore(s => s.camPointPick !== null);
  const [refs, setRefs] = useState<Partial<Record<HeightKey, CamHeightGeometryDto>>>(() => {
    const initial: Partial<Record<HeightKey, CamHeightGeometryDto>> = {};
    for (const key of ['bottom', 'top', 'feed', 'retract', 'clearance'] as const) {
      if (stored?.[key]?.geometry) initial[key] = stored[key]!.geometry!;
    }
    return initial;
  });
  useEffect(() => () => cancelCamPointPick(), []);
  const resolve = (key: HeightKey) => {
    const state = useAppStore.getState();
    if (!setup || !refs[key]) throw new Error(translate('cam.operation.heightGeometryRequired'));
    return resolveHeightGeometry(refs[key]!, state.solidScene, state.finishedSketches, setup);
  };
  const field = (key: HeightKey, onFrom: (from: HeightFrom) => void) => ({
    geometryLabel: refs[key] ? heightGeometryLabel(refs[key]!) : undefined,
    onPickGeometry: () => {
      if (!setup) return;
      const state = useAppStore.getState();
      state.setCamWorkpieceView('model');
      const sketches = state.finishedSketches.filter(s => !state.projectVisibility.hidden_sketch_names.includes(s.name));
      const candidates = heightGeometryCandidates(state.solidScene, sketches, setup);
      void requestCamPointPick(candidates, translate('cam.operation.heightGeometryPrompt')).then(candidate => {
        if (!candidate) return;
        setRefs(previous => ({ ...previous, [key]: candidate.payload as CamHeightGeometryDto }));
        onFrom('geometry');
      });
    },
  });
  return { picking, refs, resolve, field };
}
