# CI intent audit — 2026-10-09

This audit covers every application workflow and top-level job, their matrix
expansions, direct test commands, gating and artifact provenance. It reviews test
families and the assertions behind CI orchestration; it does not claim an
individual adversarial rewrite of every product unit test. Manual UI qualification
remains separate and paused during this CI task.

The starting revision was `05917334`. The final CAD inventory has 15 workflows
and 36 top-level jobs, before matrix expansion.

## Retained acceptance and its value

- **CodeQL:** native Rust/C++ and Actions/Python/JavaScript security-extended
  analysis. Rust extraction uses the checkout compiler, matching sysroot and
  proc-macro server; C++ compiles the OCCT bridge. Analysis upload success and
  the separate security-alert check must both be examined on the exact PR head.
- **Desktop packages:** version preflight and changed-file classification precede
  Windows x64/ARM64 ZIP, Ubuntu DEB, Ubuntu 22.04 AppImage and Apple silicon DMG
  builders. Packaged MCP checks without an installed SDK establish self-containment.
  Old-host AppImage build and new-host verification establish distinct portability
  properties of the same payload. Signing diagnostics do not certify production
  credentials or notarization.
- **Windows package input:** an isolated control-enabled source host qualifies
  owned input; hashes prove that the default executable and ZIP remain unchanged.
  A source-host input pass is not a claim that the default package enables control.
  Foreground, process, document and field ownership must hold before each key.
- **Release publication:** tag-only publication awaits all package and AppImage
  verification jobs, validates VERSION/main membership, downloads this run's
  packages, verifies five checksums and uploads eleven expected assets. A failed
  builder, missing payload or foreign-run artifact must prevent publication.
- **Linux engine:** full host-neutral workspace tests, drawing Unicode fonts,
  catalog/profile verification and viewport-fixture isolation. Native OCCT
  acceptance is separate; host-neutral success must not substitute for it.
- **MCP Windows/Ubuntu shards:** core, turbine and vise use a checked compiled test
  inventory. Core retains all remaining MCP tests, serialized native OCCT
  integration, the bench feature workshop and drawing/repeated/fillet scenarios.
  Independent shard outcomes remain observable with `fail-fast: false`.
- **MCP aggregates:** all three platform shards must succeed before downloading
  exactly their same-run demo inputs. Fresh output and commit/version/hash
  manifests establish publication provenance; upstream geometry assertions
  establish model semantics.
- **Native host tests on Ubuntu/macOS/Windows:** native all-target/all-feature
  Clippy, host unit tests and concurrent OCCT initialization. Ordinary PR checks
  do not enable actual OS desktop input.
- **Opt-in native input:** keyboard, paper, annotations, CAM, drawing output,
  centers, scripts, holes and mechanisms cover distinct input/result surfaces
  on disposable desktops with captures and geometry/export assertions. Diagnostic
  package input covers the packaging-to-input boundary at two scale factors.
- **IME probes and native IME:** stock-control prerequisites distinguish runner
  feasibility from Bevy behavior. Actual Windows/macOS Bevy callbacks and owned
  input remain separately qualified. Stock-control success is not Bevy acceptance.
- **Accessibility and print cancellation:** explicit platform diagnostics cover
  native UI Automation and cancellation; they do not establish submitted printing
  or a complete accessibility audit.
- **Native switching:** matched baseline/candidate, shared compiler and inputs,
  reversed-order repeats and provenance establish a bounded latency comparison.
  It is not a universal timing guarantee.
- **Native visual:** the GPU projected-boundary/thin-wall matrix is distinct from
  semantic control and headless host tests. Captures require visual review.
- **Knowledge Pages:** offline media tests, knowledge validation, search freshness
  and pinned public media establish help/reference correctness. Deployment is
  non-PR and depends on the successful build; it does not qualify CAD behavior.
- **Required interface/shared interface contracts:** the stable required-check
  wrapper, control identity/focus/ownership tests and WASM boundary compilation
  protect shared host contracts. The historical "Frontend regression tests" check
  name does not imply React or live browser-input coverage.
- **Rust web:** WASM lint, binding build and exported-facade execution in Chrome
  protect actual browser ABI/output. Repository tooling separately checks all
  workspace formatting, host-neutral lint, catalogs/profiles, orchestration,
  workflow/interface contracts, icons and version carriers.
- **Private session transport:** Windows/Linux policy tests and disposable
  privileged Unix ownership checks protect the different OS access primitives.
- **Version guard:** carrier consistency, release-tooling tests and release-tag
  main membership protect source/release identity. The package's reusable version
  gate remains valuable even when the standalone check also runs.
- **Windows ARM OCCT warmer:** default-branch-only verified SDK/cache provenance
  reduces package queue cost. It is not a product behavior test.

## Drift corrected in this batch

1. Package/publication and MCP provenance contracts now inspect parsed executable
   YAML, including expanded aliases, before retaining their existing shell/ABI
   detail assertions. Adversarial fixtures reject disabled/commented gates,
   ignored failures, missing dependencies/shards, wrong-platform aggregate results
   and foreign run/repository/token/pattern inputs. Source-text checks alone could
   previously mistake nearby examples for active gates.
2. Native font and visual diagnostics list the same compiled, ignored-test filter
   first. Exactly two font tests or one visual matrix must exist; zero, renamed,
   duplicated or benchmark selections fail. The original locked desktop library
   test invocation and failure propagation remain intact.
3. Windows package classification includes executed ARM preparation and shell
   preflight regression scripts, including renamed old paths. Windows-only input
   changes require Windows packages without unrelated platform builds.
4. The Windows keyboard fixture pins the retained Project name control key,
   binding and current document context. Each key, including after activation,
   rejects a replaced, missing, disabled, read-only or different field. Inspection
   IDs and text/selection may change. Failed or partial keys are never retried.
5. Sensitive authored labels, feature IDs and complete MCP error/model summaries
   no longer appear in six failure diagnostics. Original assertion conditions
   remain; naming tests protect successful labels and both rejection conditions.
6. CodeQL and session transport run once per open-PR candidate, plus main pushes
   and retained manual/scheduled applicability. Heavy workflows cancel only
   superseded PR work; main/tag/manual qualification receipts remain intact.

No CAD test is deleted, ignored, tolerance-relaxed or converted to
`continue-on-error`. Auxiliary partial diagnostic uploads can remain warning-only;
successful release/demo payload uploads and acceptance commands fail closed.

## Validation and status boundaries

Focused local verification passed 6 recipe tests, 11 workflow contracts, 4 CI
orchestration tests, 4 changed-file classification tests and 10 Windows keyboard
guard tests, plus formatting and xtask lint with the native-control harness enabled.
Mocked Windows preflight checks passed 8 account and 42 shell cases; they establish helper policy and do
not establish an actual hosted desktop. The new Bevy binding regression and full
platform acceptance must pass hosted CI on the pushed head.

Completion requires every applicable check on that exact head to be green,
including security alerts, MCP aggregates and packages. Intentional opt-in/tag-only
skips retain their documented applicability. Green CI does not complete the broad
UI/MCP coverage ledger, physical gesture qualification or three-model walkthrough.

CAD runtime, model, saved and recovery data were preserved.
