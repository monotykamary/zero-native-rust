use std::fs;
use std::path::Path;

use crate::assets;

#[derive(Debug, Clone)]
pub struct BundleStats {
    pub asset_count: usize,
    pub manifest_path: String,
}

impl Default for BundleStats {
    fn default() -> Self {
        Self {
            asset_count: 0,
            manifest_path: "asset-manifest.zon".into(),
        }
    }
}

pub fn bundle(assets_dir: &str, output_dir: &str) -> Result<BundleStats, String> {
    let _ = fs::create_dir_all(output_dir);

    let assets_path = Path::new(assets_dir);
    if !assets_path.exists() {
        write_manifest(output_dir, &[])?;
        return Ok(BundleStats::default());
    }

    let mut copied: Vec<assets::Asset> = Vec::new();

    collect_assets_recursive(assets_dir, assets_dir, output_dir, &mut copied)?;

    copied.sort_by(|a, b| a.id.cmp(&b.id));

    write_manifest(output_dir, &copied)?;

    Ok(BundleStats {
        asset_count: copied.len(),
        manifest_path: "asset-manifest.zon".into(),
    })
}

fn collect_assets_recursive(
    base_dir: &str,
    current_dir: &str,
    output_dir: &str,
    assets: &mut Vec<assets::Asset>,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if path.is_dir() {
            collect_assets_recursive(
                base_dir,
                &path.to_string_lossy(),
                output_dir,
                assets,
            )?;
        } else {
            let rel_path = path
                .strip_prefix(base_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let bytes = fs::read(&path).map_err(|e| format!("{}: {}", path.display(), e))?;

            let out_path = Path::new(output_dir).join(&rel_path);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&out_path, &bytes).map_err(|e| e.to_string())?;

            let path_str = rel_path.replace('\\', "/");
            assets.push(assets::Asset {
                id: path_str.clone(),
                kind: assets::infer_kind(&path_str),
                source_path: path.to_string_lossy().to_string(),
                bundle_path: path_str,
                byte_len: bytes.len() as u64,
                hash: assets::sha256_hash(&bytes),
                media_type: assets::infer_media_type(&name).map(String::from),
            });
        }
    }
    Ok(())
}

fn write_manifest(output_dir: &str, assets_list: &[assets::Asset]) -> Result<(), String> {
    let manifest_path = Path::new(output_dir).join("asset-manifest.zon");
    let mut out = String::new();
    out.push_str(".{ .assets = .{\n");
    for asset in assets_list {
        let hex = asset.hash.to_hex();
        let hex_str: String = hex.iter().map(|b| *b as char).collect();
        out.push_str(&format!(
            "  .{{ .id = \"{}\", .bundle_path = \"{}\", .source_path = \"{}\", .byte_len = {}, .hash = \"{}\"",
            asset.id, asset.bundle_path, asset.source_path, asset.byte_len, hex_str,
        ));
        if let Some(ref mt) = asset.media_type {
            out.push_str(&format!(", .media_type = \"{}\"", mt));
        }
        out.push_str(" },\n");
    }
    out.push_str("} }\n");
    fs::write(&manifest_path, out).map_err(|e| e.to_string())
}
