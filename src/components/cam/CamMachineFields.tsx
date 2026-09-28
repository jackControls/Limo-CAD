import { compensationGuidance, createMachineAssignment, MACHINE_PRESETS } from '../../cam/machines';
import type { CamMachineAssignmentDto, CamPostDialect } from '../../engine/types';
import { CAM_DIALOG_INPUT, CAM_DIALOG_LABEL, DialogSection } from './camFields';

import { useTranslation } from '../../i18n';

export function CamMachineFields({ machine, onChange }: {
  machine: CamMachineAssignmentDto | null;
  onChange: (machine: CamMachineAssignmentDto | null) => void;
}) {
  const { t } = useTranslation();

  return <DialogSection title={t('cam.machine.sectionMachineController')}>
    <label className="block">
      <span className={CAM_DIALOG_LABEL}>{t('cam.machine.setupTarget')}</span>
      <select data-testid="cam-machine-select" className={CAM_DIALOG_INPUT}
        value={machine ? 'snapshot' : 'generic'}
        onChange={event => {
          const value = event.target.value;

          if (value !== 'snapshot') onChange(value === 'generic' ? null : createMachineAssignment(value as CamPostDialect));
        }}>
        <option value="generic">{t('cam.machine.genericOption')}</option>
        {machine && <option value="snapshot">{t('cam.machine.projectSnapshot').replace('{name}', machine.profile.name)}</option>}
        <optgroup label={t('cam.machine.newFromStarter')}>
          {MACHINE_PRESETS.map(p => <option key={p.dialect} value={p.dialect}>{t(p.labelKey)}</option>)}
        </optgroup>

      </select>
    </label>
    {machine && <>
      <label className="block">
        <span className={CAM_DIALOG_LABEL}>{t('cam.machine.shopMachineName')}</span>
        <input data-testid="cam-machine-name" className={CAM_DIALOG_INPUT}
          value={machine.profile.name} maxLength={128}
          onChange={event => onChange({ ...machine, profile: { ...machine.profile, name: event.target.value } })} />
      </label>
      <p className="text-[10px] leading-relaxed text-mute">
        {machine.profile.controller.model} · {machine.profile.controller.language.replace(/_/g, ' ')} · {t('cam.machine.revision')} {machine.profile.revision}
      </p>
      <p className="text-[10px] leading-relaxed text-mute">{compensationGuidance(machine.profile.post.dialect)}</p>
      {machine.profile.post.siemens_828d?.spindle_stop_subprogram && <p className="text-[10px] leading-relaxed text-warn">
        {t('cam.machine.privateSpindleStop').replace('{program}', machine.profile.post.siemens_828d.spindle_stop_subprogram)}
      </p>}

    </>}

    <p className="text-[10px] leading-relaxed text-mute">
      {machine ? t('cam.machine.starterHelp')
        : t('cam.machine.noMachineHelp')}
      {' '}{t('cam.machine.currentExecutionHelp')}
    </p>
  </DialogSection>;
}
