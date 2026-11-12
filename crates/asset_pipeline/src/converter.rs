use crate::inspector::{PipelineError, inspect_installation};
use gtasarust_engine::GameDataRoot;
use gtasarust_gta_formats::{
    col::parse_col_archive,
    img::ImgArchive,
    rw::{DffClump, TxdArchive},
};
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BatchConvertOptions {
    pub game_dir: PathBuf,
    pub output_dir: PathBuf,
    pub max_entries: Option<usize>,
    pub export_png_textures: bool,
    pub filter_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedMesh {
    pub name: String,
    pub rw_version: u32,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub materials: Vec<String>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub source_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConvertReport {
    pub converted_meshes: usize,
    pub converted_textures: usize,
    pub converted_collisions: usize,
    pub errors: Vec<String>,
    pub manifest_path: PathBuf,
}

/// Convert a parsed `DffClump` into a flattened, GPU-ready `ConvertedMesh` intermediate record.
pub fn clump_to_converted_mesh(name: &str, clump: &DffClump, source_bytes: &[u8]) -> ConvertedMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let mut materials = Vec::new();
    let mut tri_count = 0usize;

    for geom in &clump.geometries {
        let base_vertex = positions.len() as u32;
        positions.extend_from_slice(&geom.positions);
        if geom.normals.len() == geom.positions.len() {
            normals.extend_from_slice(&geom.normals);
        } else {
            normals.extend(std::iter::repeat_n([0.0, 0.0, 1.0], geom.positions.len()));
        }
        if geom.uvs.len() == geom.positions.len() {
            uvs.extend_from_slice(&geom.uvs);
        } else {
            uvs.extend(std::iter::repeat_n([0.0, 0.0], geom.positions.len()));
        }
        for mat in &geom.materials {
            if let Some(tex) = &mat.texture_name {
                materials.push(tex.clone());
            }
        }

        if !geom.splits.is_empty() {
            for split in &geom.splits {
                if geom.is_tristrip {
                    for (step, win) in split.indices.windows(3).enumerate() {
                        let (a, b, c) = if step % 2 == 1 {
                            (win[0], win[2], win[1])
                        } else {
                            (win[0], win[1], win[2])
                        };
                        if a != b && b != c && a != c {
                            indices.push(base_vertex + a);
                            indices.push(base_vertex + b);
                            indices.push(base_vertex + c);
                            tri_count += 1;
                        }
                    }
                } else {
                    for &[a, b, c] in split.indices.as_chunks::<3>().0 {
                        indices.push(base_vertex + a);
                        indices.push(base_vertex + b);
                        indices.push(base_vertex + c);
                        tri_count += 1;
                    }
                }
            }
        } else {
            for tri in &geom.triangles {
                indices.push(base_vertex + u32::from(tri[0]));
                indices.push(base_vertex + u32::from(tri[1]));
                indices.push(base_vertex + u32::from(tri[2]));
                tri_count += 1;
            }
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(source_bytes);
    let digest = hasher.finalize();
    let source_sha256: String = digest.iter().map(|b| format!("{b:02X}")).collect();

    ConvertedMesh {
        name: name.to_string(),
        rw_version: clump.rw_version,
        vertex_count: positions.len(),
        triangle_count: tri_count,
        materials,
        positions,
        normals,
        uvs,
        indices,
        source_sha256,
    }
}

/// Run reproducible batch extraction, conversion, and output validation for an installation.
pub fn convert_batch(opts: &BatchConvertOptions) -> Result<BatchConvertReport, PipelineError> {
    let report = inspect_installation(&opts.game_dir)?;
    std::fs::create_dir_all(&opts.output_dir)?;

    let meshes_dir = opts.output_dir.join("meshes");
    let tex_dir = opts.output_dir.join("textures");
    let col_dir = opts.output_dir.join("collisions");
    std::fs::create_dir_all(&meshes_dir)?;
    std::fs::create_dir_all(&tex_dir)?;
    std::fs::create_dir_all(&col_dir)?;

    let manifest_path = opts.output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&report.manifest)?;
    std::fs::write(&manifest_path, manifest_json)?;

    let vfs = GameDataRoot::open(&opts.game_dir)?;
    let gta3_path = vfs.resolve("models/gta3.img")?;
    let gta3 = ImgArchive::open(&gta3_path)?;

    let entries: Vec<_> = gta3
        .entries()
        .iter()
        .filter(|e| {
            if let Some(prefix) = &opts.filter_prefix {
                e.name
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase())
            } else {
                true
            }
        })
        .take(opts.max_entries.unwrap_or(usize::MAX))
        .collect();

    let results: Vec<(usize, usize, usize, Option<String>)> = entries
        .par_iter()
        .map(|entry| {
            let lower = entry.name.to_ascii_lowercase();
            let raw = match gta3.read_entry_data(entry) {
                Ok(b) => b,
                Err(e) => return (0, 0, 0, Some(format!("{}: {e}", entry.name))),
            };

            if lower.ends_with(".dff") {
                match DffClump::parse(&raw) {
                    Ok(clump) => {
                        let mesh = clump_to_converted_mesh(&entry.name, &clump, &raw);
                        let out_path = meshes_dir.join(format!("{}.json", entry.name));
                        if let Ok(json) = serde_json::to_vec(&mesh)
                            && std::fs::write(&out_path, json).is_ok()
                        {
                            return (1, 0, 0, None);
                        }
                        (
                            0,
                            0,
                            0,
                            Some(format!("failed writing {}", out_path.display())),
                        )
                    }
                    Err(e) => (0, 0, 0, Some(format!("{}: {e}", entry.name))),
                }
            } else if lower.ends_with(".txd") {
                match TxdArchive::parse(&raw) {
                    Ok(txd) => {
                        let mut count = 0usize;
                        for tex in &txd.textures {
                            if opts.export_png_textures
                                && let Some(img) = ImageBuffer::<Rgba<u8>, _>::from_raw(
                                    tex.width,
                                    tex.height,
                                    tex.rgba8.clone(),
                                )
                            {
                                let safe_name = tex.name.replace(['/', '\\', ':', '*', '?'], "_");
                                let sub = tex_dir.join(entry.name.trim_end_matches(".txd"));
                                let _ = std::fs::create_dir_all(&sub);
                                let png_path = sub.join(format!("{safe_name}.png"));
                                if img.save(&png_path).is_ok() {
                                    count += 1;
                                }
                            } else {
                                count += 1;
                            }
                        }
                        (0, count, 0, None)
                    }
                    Err(e) => (0, 0, 0, Some(format!("{}: {e}", entry.name))),
                }
            } else if lower.ends_with(".col") {
                match parse_col_archive(&raw) {
                    Ok(cols) => {
                        let out_path = col_dir.join(format!("{}.json", entry.name));
                        if let Ok(json) = serde_json::to_vec(&cols)
                            && std::fs::write(&out_path, json).is_ok()
                        {
                            return (0, 0, cols.len(), None);
                        }
                        (
                            0,
                            0,
                            0,
                            Some(format!("failed writing {}", out_path.display())),
                        )
                    }
                    Err(e) => (0, 0, 0, Some(format!("{}: {e}", entry.name))),
                }
            } else {
                (0, 0, 0, None)
            }
        })
        .collect();

    let mut converted_meshes = 0usize;
    let mut converted_textures = 0usize;
    let mut converted_collisions = 0usize;
    let mut errors = Vec::new();
    for (m, t, c, err) in results {
        converted_meshes += m;
        converted_textures += t;
        converted_collisions += c;
        if let Some(e) = err {
            errors.push(e);
        }
    }

    Ok(BatchConvertReport {
        converted_meshes,
        converted_textures,
        converted_collisions,
        errors,
        manifest_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_minimal_clump() {
        let clump = DffClump::default();
        let mesh = clump_to_converted_mesh("empty.dff", &clump, b"test");
        assert_eq!(mesh.vertex_count, 0);
        assert_eq!(mesh.triangle_count, 0);
        assert!(!mesh.source_sha256.is_empty());
    }
}
