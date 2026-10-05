/** Public UI + browser engine contract for the Flat finishing editor. */
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
    const simulate = engine.camSimulate.bind(engine);
    engine.camSimulate = (request) => simulate({ ...request, voxel_size: 0.5, max_voxels: 20000 });
    const store = window.__appStore.getState();
    store.applySolidUpdate(await engine.newProject());
    await engine.beginSketch({ type: 'origin_plane', plane: 'xy' });
    await engine.addRectangle({ mode: 'two_point', p1: { x: 4, y: 4 }, p2: { x: 12, y: 10 }, ctrl_held: true });
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
    cam.tools = [{ id: 1, number: 1, name: 'EM4 test', kind: 'flat_end_mill', diameter: 4,
      flute_length: 10, overall_length: 30, center_cutting: true, flute_count: 3,
      point_angle_degrees: null, corner_radius: null, cutting, cutting_presets: [],
      default_step_down: null, default_step_over: null }];
    cam.setups = [{ id: 1, name: 'Flat fixture', wcs: { origin: { x: 0, y: 0, z: 0 },
      x_axis: [1, 0, 0], y_axis: [0, 1, 0], z_axis: [0, 0, 1] }, wcs_origin: { mode: 'explicit' },
      work_offset: 'g54', work_offset_count: 1, stock_spec: { mode: 'legacy_box' }, resolved_stock: { shape: 'box' },
      stock: { min: { x: 0, y: 0, z: -3 }, max: { x: 16, y: 14, z: 0 } }, stock_model_box: null,
      body_ids: update.scene.bodies.map((b) => b.id), operations: [] }];
    cam.active_setup_id = 1; cam.next_setup_id = 2; cam.next_tool_id = 2; cam.next_operation_id = 1;
    await store.setCamDocument(cam);
    store.setSelectedCamSetupId(1); store.setSelectedCamOperationId(null); store.setActiveTab('cam');
  });
  await page.getByRole('button', { name: 'Flat', exact: true }).click();
  const dialog = page.getByTestId('cam-flat-dialog');
  await dialog.waitFor();
  // Flat has no linking page of its own.
  assert.equal(await dialog.getByRole('navigation', { name: 'Operation pages' }).locator('svg').count(), 4);
  for (const tab of ['Geometry', 'Heights', 'Passes', 'Tool']) {
    await dialog.getByRole('button', { name: tab, exact: true }).click();
    assert.equal(await dialog.getByRole('button', { name: tab, exact: true }).getAttribute('aria-pressed'), 'true');
  }
  await dialog.getByRole('button', { name: 'Passes', exact: true }).click();
  assert.equal(await dialog.getByRole('spinbutton', { name: /^Stepover/ }).inputValue(), '1.2');
  await dialog.getByRole('button', { name: 'Heights', exact: true }).click();
  const height = (label) => dialog.locator('section').filter({ has: page.getByText(label, { exact: true }) });
  assert.equal(await height('TOP HEIGHT').getByLabel('From', { exact: true }).inputValue(), 'stock_top');
  assert.equal(await height('BOTTOM HEIGHT').getByLabel('From', { exact: true }).inputValue(), 'model_bottom');
  await dialog.getByRole('button', { name: 'Save & generate', exact: true }).click({ timeout: 60000 });
  await dialog.waitFor({ state: 'detached', timeout: 60000 });
  const result = await page.evaluate(async () => {
    const engine = window.__engine;
    const cam = await engine.camDocument();
    const statuses = await engine.camToolpathStatuses();
    const program = await engine.camPlan(1);
    const op = cam.setups[0].operations[0];
    const cuts = program.commands.filter((c) => c.kind === 'linear' && Math.abs(c.to.z) < 1e-9);
    const xs = window.__appStore.getState().solidScene.bodies[0].mesh.positions.filter((_, i) => i % 3 === 0);
    return { kind: op.kind, targets: op.geometry?.targets.length, status: statuses[0].state,
      bodyMinX: Math.min(...xs), bodyMaxX: Math.max(...xs),
      cuts: cuts.length, minX: Math.min(...cuts.map((c) => c.to.x)), maxX: Math.max(...cuts.map((c) => c.to.x)) };
  });
  assert.equal(result.kind, 'flat3d');
  assert.equal(result.targets, 1);
  assert.equal(result.status, 'current');
  assert.ok(result.cuts > 0, 'passes on the top floor');
  // The open top face is cut past its edges by half the 2 mm radius.
  assert.ok(Math.abs(result.minX - (result.bodyMinX - 1)) < 0.05 && Math.abs(result.maxX - (result.bodyMaxX + 1)) < 0.05, JSON.stringify(result));
  await page.evaluate(() => window.__appStore.getState().setCamDialog({ type: 'operation', kind: 'flat3d', editId: 1 }));
  await dialog.waitFor();
  assert.equal(await dialog.getByLabel('Operation name', { exact: true }).inputValue(), 'Flat');
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  assert.deepEqual(errors, []);
  console.log('PASS: Flat finishing tabs, defaults, current CAD capture, generation and editing');
} finally { await browser.close(); }
