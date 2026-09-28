#!/usr/bin/env bash
# All processes and displays belong to this explicit disposable measurement job.
set -euo pipefail
test "${NBCAD_SWITCHING_CI:-}" = 1
test "${GITHUB_ACTIONS:-}" = true
test "${RUNNER_ENVIRONMENT:-}" = github-hosted
test "${RUNNER_OS:-}" = Linux
test "${GITHUB_REPOSITORY:-}" = jackControls/noBS-CAD
evidence="$RUNNER_TEMP/native-switching"
binaries="$RUNNER_TEMP/switching-binaries"
fixtures="$GITHUB_WORKSPACE/candidate/xtask/fixtures/switching"
test "$(git rev-parse HEAD)" = "$(cat "$evidence/builds/candidate.sha")"
mkdir -p "$evidence/inputs"
cp "$fixtures/part-a.nbcad" "$fixtures/part-b.nbcad" "$fixtures/manifest.json" \
  "$fixtures/sheets.nbcad" "$fixtures/sheets-manifest.json" "$evidence/inputs/"
sha256sum "$evidence"/inputs/*.nbcad > "$evidence/inputs/archive.sha256"
failures=0
for scenario in document-tabs drawing-sheets; do
  if [[ "$scenario" == document-tabs ]]; then
    models=(--model-a "$fixtures/part-a.nbcad" --model-b "$fixtures/part-b.nbcad")
  else
    models=(--model-a "$fixtures/sheets.nbcad")
  fi
  mkdir -p "$evidence/$scenario"
for repeat in 1 2; do
  if [[ "$repeat" == 1 ]]; then
    hosts=(baseline-react candidate-native)
  else
    hosts=(candidate-native baseline-react)
  fi
  for instances in 1 2; do
    for host in "${hosts[@]}"; do
      source=${host%-*}
      shell=${host##*-}
      out="$evidence/$scenario/repeat-$repeat-instances-$instances-$host"
      # The driver creates its own empty output directory. The wrapper log is
      # adjacent so pre-launch failures are retained without violating that guard.
      if ! WINIT_X11_SCALE_FACTOR=1 GDK_BACKEND=x11 dbus-run-session -- \
        xvfb-run --auto-servernum --server-args='-screen 0 3200x2160x24' \
        bash -c '
          set -euo pipefail
          openbox > "$1.openbox.log" 2>&1 &
          wm=$!
          trap '\''kill "$wm" 2>/dev/null || true; wait "$wm" 2>/dev/null || true'\'' EXIT
          for attempt in $(seq 1 50); do
            xprop -root _NET_SUPPORTING_WM_CHECK | grep -q "window id" && break
            sleep 0.1
          done
          xprop -root _NET_SUPPORTING_WM_CHECK | grep -q "window id"
          vulkaninfo --summary > "$1.vulkan.txt" 2>&1
          shift
          "$@"
        ' switching "$out" "$binaries/xtask" test-mcp switching-measurement \
          --server "$binaries/$host" --out "$out" \
          --scenario "$scenario" "${models[@]}" \
          --shell "$shell" --commit "$(cat "$evidence/builds/$source.sha")" \
          --profile release --cycles 20 --instances "$instances" \
          > "$out.driver.log" 2>&1; then
        failures=$((failures + 1))
      fi
    done
  done
done
done
python3 - "$evidence" "$failures" <<'PY'
import json
import hashlib
import sys
from pathlib import Path
root = Path(sys.argv[1])
scenarios = {}
for scenario, names in [('document-tabs', ['part-a', 'part-b']), ('drawing-sheets', ['sheets'])]:
    reports = sorted((root / scenario).glob('repeat-*/report.json'))
    metadata = sorted((root / scenario).glob('repeat-*/metadata.json'))
    frozen_hashes = [hashlib.sha256((root / 'inputs' / f'{name}.nbcad').read_bytes()).hexdigest()
                     for name in names]
    inputs_match = len(metadata) == 8 and all(
        json.loads(p.read_text())['input_sha256'] == frozen_hashes
        and json.loads(p.read_text())['scenario'] == scenario for p in metadata)
    models = [sorted((root / scenario).glob(f'repeat-*/instance-*/loaded-{n}.json'))
              for n in range(len(names))]
    equal = all(paths and all(json.loads(p.read_text()) == json.loads(paths[0].read_text())
                             for p in paths) for paths in models)
    data = {
        'reports': {str(p.relative_to(root)): json.loads(p.read_text()) for p in reports},
        'all_expected_reports_present': len(reports) == 8,
        'loaded_model_counts': [len(paths) for paths in models],
        'all_expected_loaded_models_present': all(len(paths) == 12 for paths in models),
        'exact_loaded_models_equal_across_hosts': equal,
        'all_input_hashes_match_frozen_archives': inputs_match,
    }
    data['complete'] = (data['all_expected_reports_present']
                        and data['all_expected_loaded_models_present'] and equal and inputs_match
                        and all(r.get('completed') for r in data['reports'].values()))
    scenarios[scenario] = data
summary = {
    'failed_invocations': int(sys.argv[2]),
    'scenarios': scenarios,
    'process_statistics_scope': 'bounded owned process tree plus top-level counters; inspect each sample for partial observations; summed RSS is not unique physical memory',
    'measurement': 'application acknowledgment; no equivalent compositor/GPU presentation claim',
    'performance_acceptance': 'not established',
    'user_report_cause': 'not established',
}
complete = not summary['failed_invocations'] and all(s['complete'] for s in scenarios.values())
summary['complete_matched_measurement'] = complete
(root / 'comparison.json').write_text(json.dumps(summary, indent=2) + '\n')
sys.exit(0 if complete else 1)
PY
