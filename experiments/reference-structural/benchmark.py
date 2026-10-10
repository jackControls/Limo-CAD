import gmsh, pathlib, subprocess, os, json, time, re, hashlib
root=pathlib.Path(__file__).parent
solver=root/'calculix/calculix_2.23_4win/ccx_static.exe'
env=os.environ.copy();env.update(OMP_NUM_THREADS='1',CCX_NPROC_RESULTS='1',CCX_NPROC_EQUATION_SOLVER='1')
E=2000.;nu=.35;force=.1;L=40.;width=6.;thickness=2.
reference=force*L**3/(3*E*(width*thickness**3/12))
results=[]
gmsh.initialize();gmsh.option.setNumber('General.Terminal',0)
gmsh.model.add('beam');gmsh.model.occ.addBox(0,0,0,L,width,thickness);gmsh.model.occ.synchronize()
gmsh.option.setString('Geometry.OCCTargetUnit','MM');gmsh.write(str(root/'beam.step'))
for size in [1.5,1.,.7]:
    gmsh.clear();gmsh.model.add('imported beam')
    volumes=gmsh.model.occ.importShapes(str(root/'beam.step'));gmsh.model.occ.synchronize()
    assert len(volumes)==1
    faces=gmsh.model.getEntities(2)
    fixed=[tag for _,tag in faces if abs(gmsh.model.occ.getCenterOfMass(2,tag)[0])<1e-8]
    tip=[tag for _,tag in faces if abs(gmsh.model.occ.getCenterOfMass(2,tag)[0]-L)<1e-8]
    assert len(fixed)==len(tip)==1
    gf=gmsh.model.addPhysicalGroup(2,fixed);gt=gmsh.model.addPhysicalGroup(2,tip)
    gmsh.model.setPhysicalName(2,gf,'clamp');gmsh.model.setPhysicalName(2,gt,'load')
    gmsh.model.addPhysicalGroup(3,[v[1] for v in volumes],name='solid')
    gmsh.option.setNumber('Mesh.MeshSizeMin',size);gmsh.option.setNumber('Mesh.MeshSizeMax',size)
    gmsh.option.setNumber('Mesh.MaxNumThreads3D',1)
    start=time.perf_counter();gmsh.model.mesh.generate(3);gmsh.model.mesh.setOrder(2)
    tags,xyz,_=gmsh.model.mesh.getNodes();points={int(t):tuple(xyz[i*3:i*3+3]) for i,t in enumerate(tags)}
    types,ids,connect=gmsh.model.mesh.getElements(3);assert list(types)==[11]
    fixed_nodes,_=gmsh.model.mesh.getNodesForPhysicalGroup(2,gf)
    tip_nodes,_=gmsh.model.mesh.getNodesForPhysicalGroup(2,gt)
    fixed_nodes=list(map(int,fixed_nodes));tip_nodes=list(map(int,tip_nodes))
    quality=list(gmsh.model.mesh.getElementQualities(ids[0],'minSICN'))
    gmsh.write(str(root/f'beam-{size}.msh'))
    text=['*NODE']
    for tag,p in points.items():text.append(f'{tag},'+','.join(f'{x:.12g}' for x in p))
    text+=['*ELEMENT,TYPE=C3D10,ELSET=SOLID']
    edges=[(0,1),(1,2),(2,0),(0,3),(1,3),(2,3)]
    for i,tag in enumerate(ids[0]):
        nodes=list(map(int,connect[0][i*10:(i+1)*10]));out=nodes[:4]
        for a,b in edges:
            midpoint=tuple((points[nodes[a]][j]+points[nodes[b]][j])/2 for j in range(3))
            n=min(nodes[4:],key=lambda n:sum((points[n][j]-midpoint[j])**2 for j in range(3)))
            assert sum((points[n][j]-midpoint[j])**2 for j in range(3))<1e-12
            out.append(n)
        assert len(set(out))==10
        text.append(f'{int(tag)},'+','.join(map(str,out)))
    for name,nodes in [('FIXED',fixed_nodes),('TIP',tip_nodes)]:
        text.append('*NSET,NSET='+name)
        for i in range(0,len(nodes),12):text.append(','.join(map(str,nodes[i:i+12])))
    text+=['*MATERIAL,NAME=REFERENCE','*ELASTIC',f'{E},{nu}',
        '*SOLID SECTION,ELSET=SOLID,MATERIAL=REFERENCE','*BOUNDARY','FIXED,1,3',
        '*STEP','*STATIC','*CLOAD']
    text += [f'{n},3,{-force/len(tip_nodes):.14g}' for n in tip_nodes]
    text += ['*NODE PRINT,NSET=TIP','U','*NODE PRINT,NSET=FIXED,TOTALS=YES','RF','*END STEP']
    stem=f'beam_{str(size).replace(".","_")}'
    (root/(stem+'.inp')).write_text('\n'.join(text)+'\n')
    run=subprocess.run([str(solver),'-i',stem],cwd=root,env=env,capture_output=True,text=True,timeout=120,creationflags=subprocess.CREATE_NO_WINDOW)
    (root/(stem+'-solver.log')).write_text(run.stdout+run.stderr)
    assert run.returncode==0,(run.returncode,run.stdout[-1000:])
    dat=(root/(stem+'.dat')).read_text();displacements=[];reaction=[];mode=None
    for line in dat.splitlines():
        if 'displacements' in line:mode='u'
        elif 'forces' in line:mode='rf'
        elif line.strip().lower().startswith('total'):mode=None
        elif re.match(r'\s*\d+\s+[-+0-9.]',line):
            values=line.split()
            if len(values)==4:
                if mode=='u':displacements.append(float(values[3]))
                elif mode=='rf':reaction.append(float(values[3]))
    delta=-sum(displacements)/len(displacements)
    record=dict(size_mm=size,nodes=len(points),tetra10=len(ids[0]),min_sicn=min(quality),
        fixed_nodes=len(fixed_nodes),tip_nodes=len(tip_nodes),tip_mean_mm=delta,
        euler_reference_mm=reference,error_percent=100*(delta/reference-1),reaction_z_N=sum(reaction),seconds=time.perf_counter()-start)
    results.append(record);print(json.dumps(record),flush=True)
gmsh.finalize()
(root/'results.json').write_text(json.dumps(dict(gmsh_version=gmsh.__version__,solver='CalculiX 2.23 ccx_static',
    units='mm N MPa',material='synthetic isotropic benchmark, not printed PETG',
    step_sha256=hashlib.sha256((root/'beam.step').read_bytes()).hexdigest(),
    solver_sha256=hashlib.sha256(solver.read_bytes()).hexdigest(),results=results),indent=2))
