//! Opt-in local fixture generation. Complete profile input is supplied by the operator, never committed.
use super::*;
use limo_cad_core::{PartPrintIntentDto, Rgba8};

#[test]
#[ignore = "requires LIMO_BAMBU_TEMPLATE and a fresh owned LIMO_BAMBU_QUALIFICATION_DIR; then run the installed slicer separately"]
fn write_five_part_four_plate_native_qualification_fixtures() {
    let input = std::env::var_os("LIMO_BAMBU_TEMPLATE")
        .expect("LIMO_BAMBU_TEMPLATE points to a complete saved 2.8.2.61 project");
    let directory = std::path::PathBuf::from(
        std::env::var_os("LIMO_BAMBU_QUALIFICATION_DIR").expect("owned output directory"),
    );
    assert!(directory.is_absolute() && directory.is_dir());
    let original = std::fs::read(&input).unwrap();
    let mut template = parse_template(&original).unwrap();
    assert_eq!(
        template.summary.objects.len(),
        5,
        "five-part reference template required"
    );
    assert_eq!(
        template.summary.plate_count, 4,
        "four-plate reference template required"
    );
    let config_source = text(&template.entries, CONFIG).unwrap().to_string();
    let config = xml(&config_source).unwrap();
    let removals = config
        .descendants()
        .filter(|n| {
            n.has_tag_name("metadata")
                && n.attribute("key")
                    .is_some_and(|k| SETTING_KEYS.contains(&k))
        })
        .map(|n| (n.range(), String::new()))
        .collect();
    template.entries.insert(
        CONFIG.into(),
        apply_edits(&config_source, removals).unwrap().into_bytes(),
    );
    template.entries.remove(MANIFEST);
    let clean_template = write_archive(&template.entries).unwrap();
    let clean = parse_template(&clean_template).unwrap();
    let mut meshes = Vec::new();
    let mut appearances = Vec::new();
    let mut instances = Vec::new();
    let mut bindings = Vec::new();
    let mut configured = Vec::new();
    let root = xml(text(&clean.entries, ROOT).unwrap()).unwrap();
    for (index, object) in clean.summary.objects.iter().enumerate() {
        assert_eq!(
            object.parts.len(),
            1,
            "qualification fixture must explicitly bind one normal volume per object"
        );
        assert_eq!(
            object.instance_count, 1,
            "fixture expects original five intentional instances"
        );
        let part = &object.parts[0];
        let body_id = BodyId(index as u64 + 1);
        let occurrence_id = index as u64 + 1;
        let target = &clean.targets[&(object.object_id, part.part_id)];
        let mesh_doc = xml(text(&clean.entries, &target.path).unwrap()).unwrap();
        let mesh_node = mesh_doc
            .descendants()
            .find(|n| n.has_tag_name((CORE_NS, "mesh")) && n.range() == target.mesh_range)
            .unwrap();
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for vertex in mesh_node
            .descendants()
            .filter(|n| n.has_tag_name((CORE_NS, "vertex")))
        {
            for (axis, key) in ["x", "y", "z"].into_iter().enumerate() {
                let value = vertex.attribute(key).unwrap().parse::<f64>().unwrap();
                lo[axis] = lo[axis].min(value);
                hi[axis] = hi[axis].max(value);
            }
        }
        let range = &clean.build_ranges[&(object.object_id, 0)];
        let build = root.descendants().find(|n| n.range() == *range).unwrap();
        let world = Matrix::parse(build.attribute("transform"))
            .unwrap()
            .compose(target.component_transform);
        let dimensions: [f32; 3] = std::array::from_fn(|axis| {
            if world.0[8 + axis].abs() > 0.001 {
                (hi[axis] - lo[axis]) as f32
            } else {
                30.
            }
        });
        let mut mesh = super::tests::cube(body_id.0);
        for point in mesh.positions.as_chunks_mut::<3>().0 {
            for axis in 0..3 {
                point[axis] = (point[axis] / 10. - 0.5) * dimensions[axis];
            }
        }
        mesh.name = format!("Synthetic reference part {}", body_id.0);
        meshes.push(mesh);
        let filament = part
            .settings
            .get("extruder")
            .or_else(|| object.settings.get("extruder"))
            .unwrap()
            .parse::<usize>()
            .unwrap()
            - 1;
        let mut appearance = BodyAppearance::default_for(body_id);
        appearance.filament_type = clean.summary.filament_types[filament].clone();
        let color = clean.summary.filament_colors[filament].trim_start_matches('#');
        appearance.color = Rgba8::opaque(
            u8::from_str_radix(&color[0..2], 16).unwrap(),
            u8::from_str_radix(&color[2..4], 16).unwrap(),
            u8::from_str_radix(&color[4..6], 16).unwrap(),
        );
        appearances.push(appearance);
        instances.push(MeshInstance {
            body_id,
            occurrence_id,
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.],
            visible: true,
        });
        bindings.push(BambuPartBinding {
            body_id,
            occurrence_id,
            object_id: object.object_id,
            instance_id: 0,
            part_id: part.part_id,
        });
        let name = object.name.to_ascii_lowercase();
        if !name.contains("sleeve") {
            let (density, pattern) = if name.contains("adapter") {
                (100., InfillPatternDto::Rectilinear)
            } else if name.contains("auger") {
                (40., InfillPatternDto::Gyroid)
            } else {
                assert!(name.contains("housing"));
                (30., InfillPatternDto::Gyroid)
            };
            configured.push(PartPrintIntentDto {
                body_id,
                settings: PrintSettingsDto {
                    wall_count: Some(6),
                    infill_density_percent: Some(density),
                    infill_pattern: Some(pattern),
                    top_shell_layers: Some(6),
                    bottom_shell_layers: Some(6),
                },
            });
        }
    }
    let namespace = "c832ce9c-765e-4b68-bf99-4b8130142a1c";
    let request = BambuProjectRequest {
        source_document_id: namespace.into(),
        bindings,
        placement: BambuPlacementMode::Template,
        allow_template_appearance: false,
        ..Default::default()
    };
    let structure = limo_cad_assembly::ComponentStructureDto::default();
    let mut intent = PrintIntentDocumentDto {
        source_document_id: Some(namespace.into()),
        ..Default::default()
    };
    for (name, parts) in [("baseline", Vec::new()), ("configured", configured)] {
        intent.parts = parts;
        let output = write_bambu_project(
            &clean_template,
            &meshes,
            &appearances,
            &instances,
            &structure,
            &intent,
            &request,
        )
        .unwrap();
        if name == "configured" {
            if let Some(native_path) = std::env::var_os("LIMO_BAMBU_ROUNDTRIP_TEMPLATE") {
                let native = std::fs::read(native_path).unwrap();
                let refresh_request = BambuProjectRequest {
                    bindings: Vec::new(),
                    refresh_reference: Some(output.report.refresh_reference.clone()),
                    ..request.clone()
                };
                let refreshed = write_bambu_project(
                    &native,
                    &meshes,
                    &appearances,
                    &instances,
                    &structure,
                    &intent,
                    &refresh_request,
                )
                .unwrap();
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(directory.join("bambu-native-roundtrip-refreshed.3mf"))
                    .unwrap()
                    .write_all(&refreshed.bytes)
                    .unwrap();
                std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(directory.join("bambu-native-roundtrip-refreshed.json"))
                    .unwrap()
                    .write_all(&serde_json::to_vec_pretty(&refreshed.report).unwrap())
                    .unwrap();
            }
        }
        let path = directory.join(format!("bambu-synthetic-{name}.3mf"));
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap()
            .write_all(&output.bytes)
            .unwrap();
        let report = serde_json::json!({"original_template_sha256":hash(&original),"qualification_template_sha256":hash(&clean_template),"project_sha256":hash(&output.bytes),"synthetic_geometry":true,"report":output.report});
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(directory.join(format!("bambu-synthetic-{name}.json")))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(&report).unwrap())
            .unwrap();
    }
    assert_eq!(
        hash(&std::fs::read(input).unwrap()),
        hash(&original),
        "source template must remain unchanged"
    );
}
