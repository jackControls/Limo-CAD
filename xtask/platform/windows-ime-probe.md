# Disposable Windows IME prerequisite probe

Dispatch `Windows IME prerequisite probe` (`windows-ime-probe.yml`) on the
reviewed branch. Both inputs default to false: the first run only records OS/image,
runner identity, current-user languages, registered/enabled TSF profiles, Japanese
capability state, text-input services/processes, and interactive desktop identity.
It requires no native CAD build. Inspect the uploaded `report.json`, including
per-query errors; `inventory-complete` does not mean an IME is ready.

GitHub requires a manually dispatched workflow to be present on the default
branch. A newly added draft-branch workflow may therefore not be dispatchable
yet; `--ref` does not itself register it. Do not merge the Bevy transition to
bootstrap this probe. An already-registered workflow can provide a separately
reviewed opt-in probe-only entry point if CI evidence is needed before then.
[GitHub dispatch requirement](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow).

`provision-japanese=true` additionally installs the discovered Japanese Basic and
Jpan font capabilities through DISM and enables the Microsoft Japanese profile
for the disposable runner user. It does not use the client-only `Install-Language`
cmdlet, change display/system locale, configure services, reboot, or sign out.
Installation attempts are written before starting; results retain elapsed time,
HRESULTs, state changes, DISM logs, and `RestartNeeded`. A required restart fails
the probe and leaves that environment prerequisite unresolved.

`exercise-ime=true` separately opts into ordinary virtual-key SendInput on a fresh
owned stock WinForms textbox. The helper verifies the foreground/focus and reads
the exact active Microsoft Japanese profile on its own UI thread. It selects
Hiragana, types ASCII `haru`, requires received `WM_IME_COMPOSITION/GCS_COMPSTR`
preedit `はる`, commits once with Enter, then checks a second composition cancels.
It records the actual IMM preedit/result strings, message sequence, profile,
key actions, owned HWND/PID, and final text. It never posts IME messages, injects
Unicode packets, or synthesizes Bevy events. A 20-second owned-window deadline
bounds this check. Missing profile/focus/composition is a failing result.

Provisioning and input are guarded to `jackControls/noBS-CAD` on an explicitly
opted-in GitHub-hosted Windows job with evidence beneath `RUNNER_TEMP`. The VM is
discarded by GitHub afterward. The inventory-only script may be run locally:

```powershell
powershell.exe -NoProfile -File xtask/platform/windows-ime-probe.ps1 -Out C:\absolute\fresh-evidence
```

Local capability queries may record elevation errors. Do not elevate or enable
the mutation/input switches on a developer machine just to satisfy the probe.
For syntax/compile validation, parse the PowerShell file and compile the C# with
`Add-Type -ReferencedAssemblies System.Windows.Forms,System.Drawing`; compiling
does not invoke `Exercise` or open a window.

**A stock-control success is environment feasibility only.** It does not validate
the Bevy text adapter, candidate popup ownership/placement/pixels, DPI transitions,
or physical keyboard hardware. Those remain separate checks. The workflow always
uploads its evidence, including failed/in-progress attempts. It performs no CAD
build, does not select a user document, and does not change the release host.

Primary references:

- [Server-supported Add-WindowsCapability](https://learn.microsoft.com/en-us/powershell/module/dism/add-windowscapability?view=windowsserver2025-ps)
- [LanguagePackManagement is client-only](https://learn.microsoft.com/en-us/powershell/module/languagepackmanagement/?view=windowsserver2025-ps)
- [Current-user language and input methods](https://learn.microsoft.com/en-us/powershell/module/international/set-winuserlanguagelist?view=windowsserver2025-ps)
- [Japanese IME keys and `haru` example](https://learn.microsoft.com/en-us/globalization/input/japanese-ime)
- [IMM composition strings](https://learn.microsoft.com/en-us/windows/win32/api/imm/nf-imm-immgetcompositionstringw)
