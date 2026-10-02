/** Remaining-stock roughing, independent setup angles and source freshness. */
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
      default_step_down: null, default_step_over: null }];
    cam.setups = [{ id: 1, name: 'Roughing fixture', wcs: { origin: { x: 0, y: 0, z: 0 },
      x_axis: [1, 0, 0], y_axis: [0, 1, 0], z_axis: [0, 0, 1] }, wcs_origin: { mode: 'explicit' },
      work_offset: 'g54', work_offset_count: 1, stock_spec: { mode: 'legacy_box' }, resolved_stock: { shape: 'box' },
      stock: { min: { x: 0, y: 0, z: -3 }, max: { x: 16, y: 14, z: 0 } }, stock_model_box: null,
      body_ids: update.scene.bodies.map((b) => b.id), operations: [] }];
    cam.active_setup_id = 1; cam.next_setup_id = 2; cam.next_tool_id = 2; cam.next_operation_id = 1;
    await store.setCamDocument(cam);
    store.applySolidUpdate(await engine.loadProjectModel(await engine.exportProjectModel()));
    store.setSelectedCamSetupId(1); store.setSelectedCamOperationId(null); store.setActiveTab('cam');
  });
  await page.getByRole('button', {name:'High Speed Roughing',exact:true}).click();
  const operation=page.getByTestId('cam-adaptive-dialog');
  await operation.getByRole('button',{name:'Heights',exact:true}).click();
  const bottom=operation.locator('section').filter({has:page.getByText('BOTTOM HEIGHT',{exact:true})});
  await bottom.getByLabel('From',{exact:true}).selectOption('model_bottom');
  await bottom.getByLabel('Offset',{exact:true}).fill('1');
  await operation.getByRole('button',{name:'Save & generate',exact:true}).click();
  await operation.waitFor({state:'detached',timeout:60000});
  const result=await page.evaluate(async()=>{
    const engine=window.__engine, store=window.__appStore.getState();
    const {restStockToSetup}=await import('/src/cam/geometry.ts');
    const cam=await engine.camDocument(), source=cam.setups[0], rest=structuredClone(source);
    rest.id=2;rest.name='Flipped stock';rest.work_offset='g55';
    rest.wcs.y_axis=[0,-1,0];rest.wcs.z_axis=[0,0,-1];
    rest.stock_spec={mode:'rest_from_setup',setup_id:1};rest.resolved_stock={shape:'rest',source_setup_id:1};
    rest.stock=restStockToSetup(source,rest.wcs);
    const op=rest.operations[0];op.id=2;op.top_z=3;op.bottom_z=1;op.clearance_z=8;op.retract_z=6;op.feed_height_z=4;
    cam.setups.push(rest);cam.next_setup_id=3;cam.next_operation_id=3;
    await store.setCamDocument(cam);await engine.camRegenerateSetup(2);
    const a=await engine.camSimulate({setup_id:1,voxel_size:0.5});
    const b=await engine.camSimulate({setup_id:2,voxel_size:0.5});
    store.applySolidUpdate(await engine.loadProjectModel(await engine.exportProjectModel()));
    return {initial:b.initial_voxels,sourceRemaining:a.remaining_voxels,removed:b.removed_voxels,collisions:b.collisions,statuses:await engine.camToolpathStatuses()};
  });
  assert.equal(result.initial,result.sourceRemaining);
  assert.ok(result.removed>0);assert.deepEqual(result.collisions,[]);
  assert.ok(result.statuses.every(s=>s.state==='current'),JSON.stringify(result.statuses));
  await page.evaluate(()=>{ const store=window.__appStore.getState(); store.setActiveTab('cam'); store.setCamDialog({type:'setup'}); });
  const setup=page.getByTestId('cam-setup-dialog');
  await setup.getByRole('combobox',{name:/Definition/i}).selectOption('rest_from_setup');
  await setup.getByRole('combobox',{name:/Continue from/i}).selectOption('2');
  await setup.getByLabel('Custom orientation angles',{exact:true}).check();
  for(const [axis,value] of [['X','23'],['Y','-41'],['Z','79']]) await setup.getByRole('spinbutton',{name:new RegExp(`Rotation ${axis}`)}).fill(value);
  await setup.getByRole('button',{name:'Create empty setup',exact:true}).click();
  await setup.waitFor({state:'detached'});
  const before=await page.evaluate(async()=> (await window.__engine.camDocument()).setups[2]);
  await page.evaluate(()=>window.__appStore.getState().setCamDialog({type:'setup',editId:3}));
  await setup.waitFor();
  assert.equal(await setup.getByLabel('Custom orientation angles',{exact:true}).isChecked(),true);
  for(const [axis,value] of [['X',23],['Y',-41],['Z',79]]) assert.ok(Math.abs(Number(await setup.getByRole('spinbutton',{name:new RegExp(`Rotation ${axis}`)}).inputValue())-value)<1e-8);
  await setup.getByRole('button',{name:'Save changes',exact:true}).click();
  await setup.waitFor({state:'detached'});
  const after=await page.evaluate(async()=> (await window.__engine.camDocument()).setups[2]);
  for(const axis of ['x_axis','y_axis','z_axis']) for(let i=0;i<3;i++) assert.ok(Math.abs(before.wcs[axis][i]-after.wcs[axis][i])<1e-8);
  await page.evaluate(async()=>{
    const engine=window.__engine, store=window.__appStore.getState(),cam=await engine.camDocument();
    cam.setups[0].operations[0].cutting.feed_xy+=10;await store.setCamDocument(cam);
    let rejected=false;
    try {await engine.camRegenerateSetup(2);} catch(e) {rejected=String(e).includes('Regenerate the source setup');}
    if(!rejected) throw new Error('Stale source stock was accepted');
    await engine.camRegenerateSetup(1);await engine.camRegenerateSetup(2);
  });
  assert.deepEqual(errors,[]);
  console.log('PASS: flipped rest roughing/removal, arbitrary-angle setup editor round trip, and source freshness enforcement');
} catch(e) { console.log('STATE',await page.evaluate(()=>({tab:window.__appStore.getState().activeTab,mode:window.__appStore.getState().mode,dialog:window.__appStore.getState().camDialog})),errors,await page.locator('body').innerText());throw e;} finally {await browser.close();}
