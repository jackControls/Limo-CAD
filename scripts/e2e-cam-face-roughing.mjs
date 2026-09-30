/** Public UI + browser engine contract for the High Speed Roughing editor. */
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
const errors = [];
page.on('pageerror', (error) => errors.push(String(error)));
try {
  await page.goto(process.env.NBCAD_E2E_BASE_URL ?? 'http://localhost:7199', { waitUntil: 'networkidle' });
  await page.waitForFunction(() => window.__appStore?.getState().document && window.__engine);
  await page.evaluate(async () => {
    const engine = window.__engine;
    // This is an editor/generation contract test, not a high-density browser
    // simulation benchmark. Keep the real Rust simulation but bound its grid;
    // native desktop runs the same kernel on its separate worker.
    const simulate = engine.camSimulate.bind(engine);
    engine.camSimulate = (request) => simulate({ ...request, voxel_size: 0.5, max_voxels: 20000 });
    const store = window.__appStore.getState();
    store.applySolidUpdate(await engine.newProject());
    await engine.beginSketch({ type: 'origin_plane', plane: 'xy' });
    await engine.addRectangle({ mode: 'two_point', p1: { x: 6, y: 5 }, p2: { x: 10, y: 9 }, ctrl_held: true });
    const ended = await engine.endSketch();
    store.setDocument(ended.document);
    store.setFinishedSketches(await engine.finishedSketches());
    store.setMode('solid');
    const catalog = await engine.profileCatalog();
    const update = await engine.extrude({ source_face: null, sketch_name: catalog[0].sketch_name,
      profile_indices: [catalog[0].profiles[0].index], operation: 'new_body',
      extent: { type: 'distance', distance: 3 }, taper_angle_deg: 0, flip: true, target_body_ids: [] });
    store.applySolidUpdate(update);
    const cam = await engine.camDocument();
    const cutting = { spindle_rpm: 8000, feed_xy: 600, feed_z: 100, coolant: 'flood' };
    // R1.5 is the recommended LNMU03 CAM proxy. The 2.5 mm profile
    // height represents that virtual cutter; maximum cutting Ap stays 1 mm.
    cam.tools = [{ id: 1, number: 1, name: 'Face16 test', kind: 'face_mill', diameter: 16,
      flute_length: 2.5, maximum_axial_depth: 1, overall_length: 100, center_cutting: false, flute_count: 2,
      point_angle_degrees: null, corner_radius: 1.5, cutting, cutting_presets: [],
      default_step_down: 0.8, default_step_over: 16 }];
    cam.setups = [{ id: 1, name: 'Roughing fixture', wcs: { origin: { x: 0, y: 0, z: 0 },
      x_axis: [1, 0, 0], y_axis: [0, 1, 0], z_axis: [0, 0, 1] }, wcs_origin: { mode: 'explicit' },
      work_offset: 'g54', work_offset_count: 1, stock_spec: { mode: 'legacy_box' }, resolved_stock: { shape: 'box' },
      stock: { min: { x: 0, y: 0, z: -3 }, max: { x: 16, y: 14, z: 0 } }, stock_model_box: null,
      body_ids: update.scene.bodies.map((b) => b.id), operations: [] }];
    cam.active_setup_id = 1; cam.next_setup_id = 2; cam.next_tool_id = 2; cam.next_operation_id = 1;
    await store.setCamDocument(cam);
    store.setSelectedCamSetupId(1); store.setSelectedCamOperationId(null); store.setActiveTab('cam');
  });
  // Exercise the actual library field and its unit conversion.
  await page.evaluate(() => window.__appStore.getState().setCamDialog({ type: 'tool', toolId: 1 }));
  const library = page.getByTestId('cam-tool-dialog');
  await library.getByRole('button', { name: 'Cutter', exact: true }).click();
  assert.equal(await library.getByLabel('Maximum axial depth (Ap)').inputValue(), '1');
  await library.getByLabel('Maximum axial depth (Ap)').fill('0.9');
  await library.getByRole('button', { name: 'Save tool', exact: true }).click();
  await library.getByRole('button', { name: 'Save tool', exact: true }).waitFor({ state: 'detached' });
  await page.evaluate(() => window.__appStore.getState().setCamDialog(null));
  assert.equal(await page.evaluate(async () => (await window.__engine.camDocument()).tools[0].maximum_axial_depth), 0.9);
  await page.getByRole('button', { name: 'High Speed Roughing', exact: true }).click();
  const dialog = page.getByTestId('cam-adaptive-dialog');
  await dialog.waitFor();
  await dialog.getByRole('button', { name: 'Passes', exact: true }).click();
  assert.equal(await dialog.getByLabel('Maximum roughing stepdown').inputValue(), '0.8');
  assert.equal(await dialog.getByLabel('Optimal radial load').inputValue(), '16');
  await dialog.getByRole('button', { name: 'Save & generate', exact: true }).click({ timeout: 60000 });
  await dialog.waitFor({ state: 'detached', timeout: 60000 });
  const result = await page.evaluate(async () => {
    const engine = window.__engine;
    const cam = await engine.camDocument();
    const program = await engine.camPlan(1);
    const sim = await engine.camSimulate({ setup_id: 1 });
    return { tool: cam.tools[0], op: cam.setups[0].operations[0], warnings: program.warnings,
      status: (await engine.camToolpathStatuses())[0].state,
      removed: sim.removed_volume_mm3, collisions: sim.collisions };
  });
  assert.equal(result.status, 'current');
  assert.equal(result.tool.corner_radius, 1.5);
  assert.equal(result.op.parameters.maximum_stepdown, 0.8);
  assert.equal(result.op.parameters.optimal_load, 16);
  assert.ok(result.warnings.some(w => w.includes('0 helical entries')));
  assert.ok(result.removed > 100);
  assert.deepEqual(result.collisions, []);
  await page.evaluate(async () => {
    const cam = await window.__engine.camDocument(); cam.units = 'inches';
    await window.__appStore.getState().setCamDocument(cam);
    window.__appStore.getState().setCamDialog({ type: 'tool', toolId: 1 });
  });
  await library.getByRole('button', { name: 'Cutter', exact: true }).click();
  assert.ok(Math.abs(Number(await library.getByLabel('Maximum axial depth (Ap)').inputValue()) * 25.4 - 0.9) < 1e-6);
  await library.getByRole('button', { name: 'Save tool', exact: true }).click();
  await library.getByRole('button', { name: 'Save tool', exact: true }).waitFor({ state: 'detached' });
  assert.ok(Math.abs(await page.evaluate(async () => (await window.__engine.camDocument()).tools[0].maximum_axial_depth) - 0.9) < 1e-6);
  await page.evaluate(async () => {
    const engine = window.__engine;
    const model = await engine.exportProjectModel();
    window.__appStore.getState().applySolidUpdate(await engine.loadProjectModel(model));
  });
  const saved = await page.evaluate(async () => (await window.__engine.camDocument()).tools[0]);
  assert.equal(saved.corner_radius, 1.5);
  assert.ok(Math.abs(saved.maximum_axial_depth - 0.9) < 1e-6);
  assert.ok(Math.abs(saved.default_step_down - 0.8) < 1e-6);
  assert.ok(Math.abs(saved.default_step_over - 16) < 1e-6);
  assert.deepEqual(errors, []);
  console.log('PASS: face-mill picker, maximum Ap editor and inch round-trip, 0.8 mm layer/16 mm engagement defaults, outside entry generation and stock removal');
} finally { await browser.close(); }
