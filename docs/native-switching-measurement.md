# Bounded switching measurement

`cargo xtask test-mcp switching-measurement` measures the existing published
document-tab click from request to application acknowledgment. It preserves raw
samples and exact model checks. It launches only after the existing private
Xvfb process-tree guard succeeds; do not run it on the user's desktop.

Prepare two `.nbcad` archives with distinct document names and real solids.
Keep these exact archive bytes for every run. Both tabs start in Solid. The
fixture copies them into a new evidence directory, launches one or two owned
applications with an isolated session registry/config directory, opens the
archives through File, and alternates the visible tab controls. Two warm-up
cycles precede the measured cycles. Lookup, attachment, model verification,
and screenshots are excluded from the timed click. Each request has a
45-second deadline; the measurement loop has a 15-minute budget. Failure stops
the run and preserves partial output.

On a disposable Linux runner with the repository's usual desktop dependencies,
window manager, and private Xvfb launch, invoke the current xtask binary against
each separately built host:

```sh
cargo xtask test-mcp switching-measurement \
  --server /absolute/build/nbcad \
  --out /absolute/empty/evidence \
  --model-a /absolute/fixtures/part-a.nbcad \
  --model-b /absolute/fixtures/part-b.nbcad \
  --shell native --commit FULL_BUILD_SOURCE_SHA --profile release \
  --cycles 20 --instances 1
```

Use `--shell react` for the default release shell. The same current driver can
control a main React binary, branch React binary, and branch native binary; it
does not require adding the driver to old source. Build React frontend assets
as normally required by that host. Native builds require
`--features dev-bevy-host`. Record build logs alongside output: the supplied
commit/profile are declarations, while binary and input SHA-256 hashes are
computed. Use equivalent profiles and adapters and avoid concurrent builds.
Repeat single-instance runs before `--instances 2`. The latter brings each
owned window forward through the existing window control and records that
request separately; it does not synthesize or validate physical Alt+Tab.

`samples.jsonl` retains each timed action, original control identity, raw
receipt fields, owned-process CPU ticks/RSS pages/I/O, and errors. Summary
statistics exclude warmups. Loaded canonical models and initial UI snapshots
are saved separately; compare those across hosts before comparing timings.
An action may complete before the OS displays a new image: React reports UI
visibility, while the native host separately reports GPU submission. The
driver explicitly does not equate those receipts or establish a performance
acceptance threshold. It does not measure GPU time, actual GPU memory,
physical input, monitor transitions, or the cause of the user's observation.

This initial fixture measures Solid project tabs and optional process
foreground requests. Drawing-sheet switching remains a separate experiment:
its synchronous paper work and projection-cache behavior must be timed without
mixing it with different workspace-restoration behavior between hosts.

## Headless lifecycle evidence

The native File lifecycle regression uses real OCCT extrusions and the normal
renderer mesh rebuild system without a window or GPU. Before the successful-close
retirement fix, closing the second tab left its cache entry and seven renderer
entities, including strong mesh/material handles, after the engine had removed
that tab. The fixture also retains an unrelated renderer owner to catch overly
broad cache pruning. With the fix, three create/switch/close cycles each leave
exactly the live tab and unrelated owner: two cache/geometry sessions and zero
closed-tab entities. Cancelled and rejected closes preserve both open tabs.

This proves an ownership leak and its bounded retirement; it measures neither
GPU memory nor switching latency. It does not explain the user's performance
observation. Ordinary activation can invalidate the shared instance-layout
revision, a behavior also present in the baseline renderer; exact warm mesh
reuse needs a separate matched measurement.

At source `c63a4829`, the feature-enabled Windows build passed all 16 native File
tests and all four Workbench tests. These include restoring a document's Drawing
workspace and canonical active sheet after switching or successfully closing a
different tab, unchanged exact project models, cancelled/stale Close, history
epochs, and window ownership. New tabs start in Solid. Only the workspace choice
is retained: editor drafts and paper/pick receipts remain transient. Paper
pan/zoom refits on document/sheet changes in both native and React implementations.

Reproduce the headless checks with the platform's OCCT runtime available:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --features dev-bevy-host --lib \
  session_bridge::native_interface::controller::files::tests:: -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml --features dev-bevy-host --lib \
  session_bridge::native_interface::controller::workbench::tests:: -- --test-threads=1
```

The disposable switching driver compiles and its statistics check passes, but
no live measurement or matched React/native timing comparison has been run yet.

The existing `native-host-tests.yml` dispatcher now accepts
`desktop-input=true,input-family=switching` to call the dedicated disposable
comparison workflow. It builds pinned main `f62248e1310fd511df20ee8bf6f2b8b268c39d50`,
the selected branch's React host, and its feature-enabled native host in the
same release profile before any measurement. Both React hosts embed their own
desktop assets with `custom-protocol`. Twelve sequential invocations cover
one/two instances and two repetitions with reversed host order. The comparison
rejects missing runs, different archive hashes, or different loaded models.
The initial CPU/RSS/I/O samples cover the top-level host only; WebKit child
processes are excluded, so those samples do not compare total application use.
