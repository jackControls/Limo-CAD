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
    cam.tools = [{ id: 1, number: 1, name: 'EM4 test', kind: 'flat_end_mill', diameter: 4,
      flute_length: 10, overall_length: 30, center_cutting: true, flute_count: 3,
      point_angle_degrees: null, corner_radius: null, cutting, cutting_presets: [],
      default_step_down: 1, default_step_over: 1 }];
    cam.setups = [{ id: 1, name: 'Roughing fixture', wcs: { origin: { x: 0, y: 0, z: 0 },
      x_axis: [1, 0, 0], y_axis: [0, 1, 0], z_axis: [0, 0, 1] }, wcs_origin: { mode: 'explicit' },
      work_offset: 'g54', work_offset_count: 1, stock_spec: { mode: 'legacy_box' }, resolved_stock: { shape: 'box' },
      stock: { min: { x: 0, y: 0, z: -3 }, max: { x: 16, y: 14, z: 0 } }, stock_model_box: null,
      body_ids: update.scene.bodies.map((b) => b.id), operations: [] }];
    cam.active_setup_id = 1; cam.next_setup_id = 2; cam.next_tool_id = 2; cam.next_operation_id = 1;
    await store.setCamDocument(cam);
    store.setSelectedCamSetupId(1); store.setSelectedCamOperationId(null); store.setActiveTab('cam');
  });
  await page.evaluate(() => { window.__cameraApi.fit(); window.__cameraApi.snapToDirection([0,0,1]); });
  await page.waitForFunction(() => { const c = window.__cameraApi.getSnapshot(); return Math.abs(c.position[0]-c.target[0]) < 1e-7 && Math.abs(c.position[1]-c.target[1]) < 1e-7; });
  await page.getByRole('button', { name: 'High Speed Roughing', exact: true }).click();
  const dialog = page.getByTestId('cam-adaptive-dialog');
  await dialog.getByRole('button', { name: 'Heights', exact: true }).click();
  const height = label => dialog.locator('section').filter({ has: page.getByText(label, { exact: true }) });
  const pick = async (label, xyz, expectedKind) => {
    await height(label).getByLabel('From', { exact: true }).focus();
    await height(label).getByLabel('From', { exact: true }).selectOption('geometry');
    assert.equal(await page.evaluate(() => document.activeElement?.tagName === 'SELECT'), false,
      'release the native dropdown before the viewport takes over');
    await page.waitForFunction(() => !!window.__appStore.getState().camPointPick);
    const point = await page.evaluate(xyz => window.__cameraApi.worldToScreen(xyz), xyz);
    // Dispatch the first click without a preparatory hover/move. A real
    // click must acquire current geometry, not prime the next click.
    const picked = await page.evaluate(({ point }) => {
      const surface = document.querySelector('[data-cad-interaction-surface="true"]');
      surface.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true,
        clientX: point.x, clientY: point.y, button: 0, buttons: 1 }));
      surface.dispatchEvent(new PointerEvent('pointerup', { bubbles: true,
        clientX: point.x, clientY: point.y, button: 0, buttons: 0 }));
      return window.__appStore.getState().camPointPick === null;
    }, { point });
    assert.equal(picked, true, `${label}: first click must finish the pick`);
    await dialog.waitFor();
    assert.equal(await height(label).getByLabel('From', { exact: true }).inputValue(), 'geometry');
  };
  await pick('BOTTOM HEIGHT', [8,7,0], 'face');
  await height('BOTTOM HEIGHT').getByLabel('Offset', { exact: true }).fill('-2.8');
  await pick('TOP HEIGHT', [8,5,0], 'sketch_line');
  await height('TOP HEIGHT').getByLabel('Offset', { exact: true }).fill('0');
  await pick('FEED HEIGHT', [6,5,0], 'sketch_point');
  await height('FEED HEIGHT').getByLabel('Offset', { exact: true }).fill('2');
  // Hide sketches to expose the same model edge and solid vertex independently.
  await page.evaluate(() => {
    const s = window.__appStore.getState();
    window.__appStore.setState({ projectVisibility: { ...s.projectVisibility, hidden_sketch_names: s.finishedSketches.map(sk => sk.name) } });
  });
  await pick('RETRACT HEIGHT', [8,5,0], 'edge');
  await height('RETRACT HEIGHT').getByLabel('Offset', { exact: true }).fill('5');
  await pick('CLEARANCE HEIGHT', [6,5,0], 'vertex');
  await height('CLEARANCE HEIGHT').getByLabel('Offset', { exact: true }).fill('10');
  await height('BOTTOM HEIGHT').getByRole('button', { name: 'Reselect' }).click();
  await page.keyboard.press('Escape');
  await dialog.waitFor();
  assert.equal(await height('BOTTOM HEIGHT').getByLabel('From', { exact: true }).inputValue(), 'geometry');
  assert.equal(await height('BOTTOM HEIGHT').getByLabel('Offset', { exact: true }).inputValue(), '-2.8');
  await page.screenshot({ path: '/tmp/nbcad-height-geometry.png' });
  await dialog.getByRole('button', { name: 'Save & generate', exact: true }).click();
  try { await dialog.waitFor({ state: 'detached', timeout: 60000 }); }
  catch (error) { throw new Error(String(error) + '\n' + await dialog.innerText()); }
  const result = await page.evaluate(async () => {
    const cam = await window.__engine.camDocument();
    return { expressions: cam.height_expressions[0], op: cam.setups[0].operations[0], status: (await window.__engine.camToolpathStatuses())[0].state };
  });
  assert.equal(result.expressions.bottom.geometry.kind, 'face');
  assert.equal(result.expressions.top.geometry.kind, 'sketch_line');
  assert.equal(result.expressions.feed.geometry.kind, 'sketch_point');
  assert.equal(result.expressions.retract.geometry.kind, 'edge');
  assert.equal(result.expressions.clearance.geometry.kind, 'vertex');
  assert.equal(result.op.bottom_z, -2.8);
  assert.equal(result.op.top_z, 0);
  assert.equal(result.op.feed_height_z, 2);
  assert.equal(result.op.retract_z, 5);
  assert.equal(result.op.clearance_z, 10);
  assert.equal(result.status, 'current');
  await page.evaluate(() => window.__appStore.getState().setCamDialog({ type: 'operation', kind: 'adaptive3d', editId: 1 }));
  await dialog.getByRole('button', { name: 'Heights', exact: true }).click();
  assert.equal(await height('BOTTOM HEIGHT').getByLabel('From', { exact: true }).inputValue(), 'geometry');
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  // Export/import retains independent identities, and regeneration resolves them.
  await page.evaluate(async () => {
    const engine = window.__engine;
    const saved = await engine.exportProjectModel();
    await engine.loadProjectModel(saved);
    await engine.camRegenerateSetup(1);
  });
  const roundTrip = await page.evaluate(() => window.__engine.camDocument());
  assert.deepEqual(roundTrip.height_expressions[0], result.expressions);
  // Unit changes affect only the displayed offset, never reference coordinates.
  await page.evaluate(async () => {
    const cam = await window.__engine.camDocument(); cam.units = 'inches';
    await window.__appStore.getState().setCamDocument(cam);
    window.__appStore.getState().setCamDialog({ type: 'operation', kind: 'adaptive3d', editId: 1 });
  });
  await dialog.getByRole('button', { name: 'Heights', exact: true }).click();
  assert.ok(Math.abs(Number(await height('BOTTOM HEIGHT').getByLabel('Offset', { exact: true }).inputValue()) * 25.4 + 2.8) < 1e-6);
  await dialog.getByRole('button', { name: 'Save & generate', exact: true }).click();
  await dialog.waitFor({ state: 'detached', timeout: 60000 });
  const inch = await page.evaluate(() => window.__engine.camDocument());
  assert.deepEqual(inch.height_expressions[0].bottom.geometry, result.expressions.bottom.geometry);
  assert.ok(Math.abs(inch.setups[0].operations[0].bottom_z + 2.8) < 1e-6);
  // The common editor uses the same picker and persisted reference contract.
  await page.evaluate(async () => {
    const cam = await window.__engine.camDocument(); cam.units = 'millimeters';
    await window.__appStore.getState().setCamDocument(cam);
    window.__appStore.getState().setCamDialog({ type: 'operation', kind: 'face' });
  });
  const faceDialog = page.getByTestId('cam-operation-dialog');
  await faceDialog.getByRole('button', { name: 'Heights', exact: true }).click();
  const bottom = faceDialog.locator('section').filter({ has: page.getByText('BOTTOM HEIGHT', { exact: true }) });
  await bottom.getByLabel('From', { exact: true }).selectOption('geometry');
  const screen = await page.evaluate(() => window.__cameraApi.worldToScreen([8,7,0]));
  await page.mouse.click(screen.x, screen.y);
  await faceDialog.waitFor();
  await bottom.getByLabel('Offset', { exact: true }).fill('-0.2');
  await faceDialog.locator('section').filter({ has: page.getByText('TOP HEIGHT', { exact: true }) }).getByLabel('Offset', { exact: true }).fill('0');
  await faceDialog.locator('button[type=submit]').click();
  try { await faceDialog.waitFor({ state: 'detached', timeout: 10000 }); }
  catch (error) { throw new Error(String(error) + '\n' + await page.locator('body').innerText()); }
  const common = await page.evaluate(() => window.__engine.camDocument());
  const faceOp = common.setups[0].operations.find(op => op.kind === 'face');
  assert.equal(common.height_expressions.find(e => e.operation_id === faceOp.id).bottom.geometry.kind, 'face');
  assert.equal(faceOp.target_z, -0.2);
  assert.deepEqual(errors, []);
  console.log('PASS: actual face, edge, vertex, sketch line/point clicks; cancellation; save, reopen, project round-trip and regeneration');
} catch (error) {
  await page.screenshot({ path: '/tmp/nbcad-height-geometry-error.png' });
  throw error;
} finally { await browser.close(); }
