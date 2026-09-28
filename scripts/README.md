# Test ownership

The desktop release is the native Bevy application. Browser Playwright suites
exercise the separate React/WASM application. `npm run e2e:browser` runs the
retained browser suite; `e2e:sketch-smoke` and `e2e:sketch-regression` are smaller
browser groups. The former `e2e:release` name was retired because these tests do
not validate the desktop release.

The desktop renderer packet and fake desktop IPC harnesses were removed with
their adapters. Their workflow areas now belong to these existing native
fixtures in `xtask/src`:

- Extrude/profile previews and internal datum support: `native-lifecycle`,
  `native-support`, and `native-planes`.
- Revolve, Sweep, Loft, and Rib: `native-build`.
- Multi-position holes and internal threads: `native-hole`; external threads:
  `native-thread`.
- CAM setup/tool/operation editing, playback, and private tool/post libraries:
  `native-cam`, `native-cam-platform`, `native-cam-geometry`, and `native-cam-nc`.
- Native camera, interaction, controls, and OS input: `native-view` and
  `native-platform`. These replace the ownership of the former browser-side
  Bevy interaction harness, not its implementation.

These names identify where validation belongs; they do not claim that every
retired case has identical coverage or that a native fixture has passed.
Native captures and platform runs supply separate evidence. Browser CPU
geometry assertions are not proof of rendered Bevy pixels.

Browser ribbon hover/focus behavior remains covered by
`src/components/RibbonMenu.browser.test.tsx` through
`node xtask/mcp/contracts.mjs`, and responsive layout by `npm run e2e:ribbon`.
Browser six-DOF tests retain the optional installed-driver and WebHID paths;
the removed fake desktop startup case does not test a native device driver.

`scripts/run-e2e.mjs` starts and stops only its own headless-test Vite process.
Set `NBCAD_E2E_PORT` for suites that honor `NBCAD_E2E_BASE_URL`; older browser
suites still use the default isolated test port 7199. Native fixture commands
are dispatched by `cargo xtask test-mcp` and require their documented owned
session/output arguments. Platform input fixtures require an isolated desktop
or CI runner, not an operator's active session.
