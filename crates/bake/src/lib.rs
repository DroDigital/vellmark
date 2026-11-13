//! The offline content tool. It turns authored data into pack files.
//!
//! Baking is deterministic: the same inputs give the same bytes, whatever order the
//! file system lists them in and whatever the clock says.

pub mod glyphs;
pub mod grades;
pub mod manifest;
pub mod materials;
pub mod pages;
pub mod paths;
pub mod scene;
pub mod sha256;
pub mod strings;
pub mod terrain;
pub mod trees;

use std::fmt;
use std::path::{Path, PathBuf};
use vellmark_content::{AssetId, AssetKind, PackWriter};

#[derive(Debug)]
pub struct BakeError(pub String);

impl fmt::Display for BakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BakeError {}

pub fn err<T>(msg: impl Into<String>) -> Result<T, BakeError> {
    Err(BakeError(msg.into()))
}

pub struct BakeInputs {
    /// The `assets` folder: authored text and data.
    pub assets: PathBuf,
    /// The `third_party` folder, with its manifest.
    pub third_party: PathBuf,
    /// A folder for results that are slow to bake, keyed by the hash of their inputs.
    /// A cached result is byte-identical to a fresh one. `None` bakes everything afresh.
    pub cache: Option<PathBuf>,
}

/// The asset ID of the glyph atlas.
pub const GLYPH_ATLAS_KEY: &str = "glyphs.atlas";

/// The packs a bake produces, by file name.
pub struct BakeOutput {
    pub packs: Vec<(String, Vec<u8>)>,
    /// One line for each thing baked, for the log.
    pub report: Vec<String>,
}

/// The language the Showcase ships in. Translation is left for the full game, but the
/// tables are already laid out one folder to a language.
pub const BASE_LANGUAGE: &str = "en";

/// The asset ID of a language's string table.
pub fn string_table_id(language: &str) -> AssetId {
    AssetId::from_key(&format!("strings.{language}"))
}

/// Files in a folder, sorted by name, so that the order never depends on the file system.
pub fn sorted_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>, BakeError> {
    let mut out = Vec::new();
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return err(format!("cannot read {}: {e}", dir.display())),
    };
    for entry in read {
        let path = entry.map_err(|e| BakeError(e.to_string()))?.path();
        if path.is_dir() {
            out.extend(sorted_files(&path, extension)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

pub fn bake(inputs: &BakeInputs) -> Result<BakeOutput, BakeError> {
    let mut report = Vec::new();

    let checked = manifest::validate(&inputs.third_party)?;
    report.push(format!("third-party files checked against the manifest: {checked}"));

    let mut base = PackWriter::new();

    let text_root = inputs.assets.join("text");
    let table = strings::bake_language(&text_root.join(BASE_LANGUAGE))?;
    report.push(format!("strings.{BASE_LANGUAGE}: {} entries", table.count));
    base.add(AssetKind::StringTable, string_table_id(BASE_LANGUAGE), table.bytes)
        .map_err(|e| BakeError(e.to_string()))?;

    let atlas = glyphs::bake_atlas(&inputs.third_party, inputs.cache.as_deref())?;
    report.push(format!("{GLYPH_ATLAS_KEY}: {} bytes", atlas.len()));
    base.add(AssetKind::GlyphAtlas, AssetId::from_key(GLYPH_ATLAS_KEY), atlas).map_err(|e| BakeError(e.to_string()))?;

    let set = materials::bake_set(&inputs.assets.join("materials/materials.ron"))?;
    report.push(format!("{}: {} materials at {} px", materials::MATERIAL_SET_KEY, set.materials.len(), set.size));
    // The trees: every level of every kind.
    let kinds = trees::read_kinds(&inputs.assets.join("trees/kinds.ron"))?;
    let index = |key: &str| set.index_of(materials::material_id(key)).ok_or_else(|| BakeError(format!("the trees need the material `{key}`")));
    let mut chains = vellmark_content::MeshLevels::default();
    for kind in &kinds {
        chains.chains.push(trees::LEVELS.iter().map(|l| trees::mesh_id(&kind.key, l)).collect());
        let levels = trees::meshes(kind, index("bark")?, index("leaves")?, index(&trees::impostor_key(&kind.key))?);
        let triangles: Vec<usize> = levels.iter().map(|(_, m)| m.indices.len() / 3).collect();
        report.push(format!("tree.{}: {triangles:?} triangles by level", kind.key));
        for (id, mesh) in levels {
            base.add(AssetKind::Mesh, id, mesh.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
        }
    }
    base.add(AssetKind::Table, AssetId::from_key(vellmark_content::MESH_LEVELS_KEY), chains.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
    let tiles = terrain::bake_tiles(&inputs.assets.join("terrain/valley.ron"), inputs.cache.as_deref())?;
    report.push(format!("terrain.valley: {} tiles", tiles.len()));
    // The ground's height anywhere, from the finest tile that holds the place, for
    // standing the trees on it.
    let mut decoded: Vec<vellmark_content::TerrainTile> =
        tiles.iter().map(|(_, bytes)| vellmark_content::TerrainTile::from_bytes(bytes)).collect::<Result<_, _>>().map_err(|e| BakeError(e.to_string()))?;
    decoded.sort_by_key(|t| t.coord.level);
    let ground = |x: f32, z: f32| -> Option<f32> {
        decoded.iter().find(|t| {
            let (x0, z0) = t.coord.origin();
            x >= x0 && z >= z0 && x < x0 + t.coord.side() && z < z0 + t.coord.side()
        }).map(|t| t.height_at(x, z))
    };
    let tree_places = terrain::tree_places(&inputs.assets.join("terrain/valley.ron"))?;
    for (id, bytes) in tiles {
        base.add(AssetKind::Tile, id, bytes).map_err(|e| BakeError(e.to_string()))?;
    }

    let mut records = vellmark_content::TreeRecords::default();
    for mut baked in scene::bake_all(&inputs.assets.join("scenes"), &set, &table.text)? {
        // The valley's trees, each with an ID from where it stands, which never changes.
        if baked.scene.terrain == AssetId::from_key("terrain.valley") {
            for p in &tree_places {
                let Some(y) = ground(p.x, p.z) else { continue };
                let (s, c) = p.yaw.sin_cos();
                let k = p.scale;
                let key = AssetId::from_key(&format!("{}.tree.{}.{}.{}", baked.key, p.kind, p.x.round() as i32, p.z.round() as i32));
                // What its bark records.
                let kind = kinds.iter().find(|k| k.key == p.kind).ok_or_else(|| BakeError(format!("no tree kind `{}`", p.kind)))?;
                records.trees.push(vellmark_content::TreeRecord { tree: key, kind: kind.key.clone(), years: trees::years(kind, k, p.x, p.z), girth: trees::girth(kind) });
                baked.scene.objects.push(vellmark_content::Object {
                    key,
                    mesh: trees::mesh_id(p.kind, trees::LEVELS[0]),
                    transform: [c * k, 0.0, s * k, p.x, 0.0, k, 0.0, y - 0.1, -s * k, 0.0, c * k, p.z],
                });
            }
            report.push(format!("{}: {} trees", baked.key, tree_places.len()));
        }
        for (id, mesh) in &baked.meshes {
            base.add(AssetKind::Mesh, *id, mesh.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
        }
        let triangles: usize = baked.meshes.iter().map(|(_, m)| m.indices.len() / 3).sum();
        report.push(format!("{}: {} meshes, {triangles} triangles, {} lights", baked.key, baked.meshes.len(), baked.scene.lights.len()));
        if let Some(pages) = &baked.pages {
            report.push(format!("{}.pages: {} pages", baked.key, pages.entries.len()));
            base.add(AssetKind::TextPage, baked.scene.pages, pages.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
        }
        if !baked.readables.items.is_empty() {
            report.push(format!("{}: {} texts to read", baked.key, baked.readables.items.len()));
            base.add(AssetKind::Table, AssetId::from_key(&vellmark_content::readables_key(&baked.key)), baked.readables.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
        }
        base.add(AssetKind::Scene, AssetId::from_key(&baked.key), baked.scene.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
    }
    report.push(format!("{}: {} trees", vellmark_content::TREE_RECORDS_KEY, records.trees.len()));
    base.add(AssetKind::Table, AssetId::from_key(vellmark_content::TREE_RECORDS_KEY), records.to_bytes()).map_err(|e| BakeError(e.to_string()))?;
    base.add(AssetKind::MaterialSet, AssetId::from_key(materials::MATERIAL_SET_KEY), set.to_bytes())
        .map_err(|e| BakeError(e.to_string()))?;


    for (key, bytes) in grades::bake_all(&inputs.assets.join("grades"))? {
        report.push(format!("{key}: {} bytes", bytes.len()));
        base.add(AssetKind::Table, AssetId::from_key(&key), bytes).map_err(|e| BakeError(e.to_string()))?;
    }

    for (key, bytes) in paths::bake_all(&inputs.assets.join("paths"))? {
        report.push(format!("{key}: {} bytes", bytes.len()));
        base.add(AssetKind::CameraPath, AssetId::from_key(&key), bytes).map_err(|e| BakeError(e.to_string()))?;
    }

    Ok(BakeOutput { packs: vec![("base.vpk".to_owned(), base.finish())], report })
}

/// Bakes and writes the packs into `out`.
pub fn bake_to_dir(inputs: &BakeInputs, out: &Path) -> Result<BakeOutput, BakeError> {
    let output = bake(inputs)?;
    std::fs::create_dir_all(out).map_err(|e| BakeError(format!("cannot create {}: {e}", out.display())))?;
    for (name, bytes) in &output.packs {
        let path = out.join(name);
        // Leave an unchanged pack untouched so that nothing downstream is rebuilt.
        if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(&path, bytes).map_err(|e| BakeError(format!("cannot write {}: {e}", path.display())))?;
        }
    }
    Ok(output)
}
