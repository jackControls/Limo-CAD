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
