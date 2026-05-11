use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Unknown,
    Image,
    Font,
    Text,
    Json,
    Binary,
    Localization,
    Audio,
    Video,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    pub bytes: [u8; 32],
}

impl Hash {
    pub fn zero() -> Self {
        Self { bytes: [0u8; 32] }
    }

    pub fn to_hex(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        let chars = b"0123456789abcdef";
        for (i, &byte) in self.bytes.iter().enumerate() {
            out[i * 2] = chars[(byte >> 4) as usize];
            out[i * 2 + 1] = chars[(byte & 0x0f) as usize];
        }
        out
    }

    pub fn parse_hex(input: &[u8]) -> Result<Self, HashError> {
        if input.len() != 64 {
            return Err(HashError::InvalidHashLength);
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = (hex_value(input[i * 2])? << 4) | hex_value(input[i * 2 + 1])?;
        }
        Ok(Self { bytes })
    }

    pub fn eql(a: &Self, b: &Self) -> bool {
        a.bytes == b.bytes
    }
}

fn hex_value(ch: u8) -> Result<u8, HashError> {
    match ch {
        b'0'..=b'9' => Ok(ch - b'0'),
        b'a'..=b'f' => Ok(ch - b'a' + 10),
        b'A'..=b'F' => Ok(ch - b'A' + 10),
        _ => Err(HashError::InvalidHashCharacter),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub id: String,
    pub kind: AssetKind,
    pub source_path: String,
    pub bundle_path: String,
    pub byte_len: u64,
    pub hash: Hash,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub assets: Vec<Asset>,
}

impl Manifest {
    pub fn find_by_id(&self, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }

    pub fn find_by_bundle_path(&self, path: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.bundle_path == path)
    }
}

pub fn sha256_hash(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&result);
    Hash { bytes: digest }
}

pub fn hash_hex(bytes: &[u8]) -> [u8; 64] {
    sha256_hash(bytes).to_hex()
}

pub fn infer_kind(path: &str) -> AssetKind {
    let ext = match path.rsplit('.').next() {
        Some(e) => e.to_ascii_lowercase(),
        None => return AssetKind::Unknown,
    };
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "bmp" => AssetKind::Image,
        "ttf" | "otf" | "woff" | "woff2" => AssetKind::Font,
        "txt" | "md" | "csv" => AssetKind::Text,
        "json" => AssetKind::Json,
        "strings" | "ftl" | "po" | "mo" => AssetKind::Localization,
        "mp3" | "wav" | "ogg" | "flac" | "m4a" => AssetKind::Audio,
        "mp4" | "webm" | "mov" | "mkv" => AssetKind::Video,
        "bin" | "dat" => AssetKind::Binary,
        _ => AssetKind::Unknown,
    }
}

pub fn infer_media_type(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "json" => "application/json",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashError {
    InvalidHashLength,
    InvalidHashCharacter,
}

pub fn normalize_path<'a>(output: &'a mut [u8], input: &[u8]) -> Result<&'a [u8], PathError> {
    if input.is_empty() {
        return Err(PathError::EmptyPath);
    }
    if input[0] == b'/' || input[0] == b'\\' {
        return Err(PathError::AbsolutePath);
    }
    let mut out_len = 0usize;
    let mut seg_start = 0usize;
    for (i, &raw) in input.iter().enumerate() {
        if raw == 0 {
            return Err(PathError::NullByte);
        }
        let ch = if raw == b'\\' { b'/' } else { raw };
        if ch == b'/' {
            validate_path_segment(&input[seg_start..i])?;
            if out_len >= output.len() {
                return Err(PathError::NoSpaceLeft);
            }
            output[out_len] = b'/';
            out_len += 1;
            seg_start = out_len;
            continue;
        }
        if out_len >= output.len() {
            return Err(PathError::NoSpaceLeft);
        }
        output[out_len] = ch;
        out_len += 1;
    }
    validate_path_segment(&input[seg_start..input.len()])?;
    Ok(&output[..out_len])
}

fn validate_path_segment(segment: &[u8]) -> Result<(), PathError> {
    if segment.is_empty() {
        return Err(PathError::EmptySegment);
    }
    if segment == b"." {
        return Err(PathError::CurrentSegment);
    }
    if segment == b".." {
        return Err(PathError::ParentSegment);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    EmptyPath,
    AbsolutePath,
    EmptySegment,
    CurrentSegment,
    ParentSegment,
    NullByte,
    NoSpaceLeft,
}

pub struct RuntimeAssets {
    pub manifest: Manifest,
}

impl RuntimeAssets {
    pub fn init(manifest: Manifest) -> Self { Self { manifest } }
    pub fn find(&self, id: &str) -> Option<&Asset> { self.manifest.find_by_id(id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vectors() {
        let h = sha256_hash(b"");
        let hex = h.to_hex();
        let expected = b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(&hex[..], &expected[..]);

        let h2 = sha256_hash(b"abc");
        let hex2 = h2.to_hex();
        let expected2 = b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(&hex2[..], &expected2[..]);
    }

    #[test]
    fn infer_kind_common() {
        assert_eq!(infer_kind("icons/app.PNG"), AssetKind::Image);
        assert_eq!(infer_kind("fonts/inter.woff2"), AssetKind::Font);
        assert_eq!(infer_kind("copy/readme.md"), AssetKind::Text);
        assert_eq!(infer_kind("data/app.json"), AssetKind::Json);
        assert_eq!(infer_kind("locales/en/messages.ftl"), AssetKind::Localization);
        assert_eq!(infer_kind("sounds/click.wav"), AssetKind::Audio);
        assert_eq!(infer_kind("video/intro.webm"), AssetKind::Video);
        assert_eq!(infer_kind("data/blob.bin"), AssetKind::Binary);
        assert_eq!(infer_kind("data/blob.unknown"), AssetKind::Unknown);
    }

    #[test]
    fn infer_media_type_common() {
        assert_eq!(Some("image/png"), infer_media_type("icons/app.png"));
        assert_eq!(Some("font/woff2"), infer_media_type("fonts/inter.woff2"));
        assert_eq!(Some("application/json"), infer_media_type("data/app.json"));
        assert_eq!(Some("audio/wav"), infer_media_type("sounds/click.wav"));
        assert_eq!(Some("video/webm"), infer_media_type("video/intro.webm"));
        assert_eq!(None, infer_media_type("data/blob.unknown"));
    }

    #[test]
    fn hash_hex_parsing_round_trips() {
        let hash = sha256_hash(b"abc");
        let hex = hash.to_hex();
        let restored = Hash::parse_hex(&hex).unwrap();
        assert!(Hash::eql(&hash, &restored));

        assert!(Hash::parse_hex(b"abc").is_err()); // too short
        let mut bad = hex;
        bad[0] = b'z';
        assert!(Hash::parse_hex(&bad).is_err());
    }

    #[test]
    fn path_normalization() {
        let mut buf = [0u8; 64];
        let result = normalize_path(&mut buf, b"images\\icons/app.png").unwrap();
        assert_eq!(b"images/icons/app.png", result);

        assert!(normalize_path(&mut buf, b"").is_err());
        assert!(normalize_path(&mut buf, b"/assets/icon.png").is_err());
        assert!(normalize_path(&mut buf, b"assets//icon.png").is_err());
        assert!(normalize_path(&mut buf, b"assets/./icon.png").is_err());
        assert!(normalize_path(&mut buf, b"assets/../icon.png").is_err());
    }

    #[test]
    fn manifest_lookup() {
        let manifest = Manifest {
            assets: vec![
                Asset {
                    id: "fonts/inter".into(),
                    kind: AssetKind::Font,
                    source_path: "assets/fonts/inter.woff2".into(),
                    bundle_path: "fonts/inter.woff2".into(),
                    byte_len: 42,
                    hash: Hash::zero(),
                    media_type: Some("font/woff2".into()),
                },
                Asset {
                    id: "icons/app".into(),
                    kind: AssetKind::Image,
                    source_path: "assets/icons/app.png".into(),
                    bundle_path: "icons/app.png".into(),
                    byte_len: 64,
                    hash: Hash::zero(),
                    media_type: Some("image/png".into()),
                },
            ],
        };
        assert_eq!("icons/app", manifest.find_by_id("icons/app").unwrap().id);
        assert_eq!("fonts/inter", manifest.find_by_bundle_path("fonts/inter.woff2").unwrap().id);
        assert!(manifest.find_by_id("missing").is_none());
        assert!(manifest.find_by_bundle_path("missing.png").is_none());
    }

    #[test]
    fn path_normalization_rejects_invalid() {
        let mut buf = [0u8; 64];
        assert!(normalize_path(&mut buf, b"assets//icon.png").is_err());
        assert!(normalize_path(&mut buf, b"assets/./icon.png").is_err());
        assert!(normalize_path(&mut buf, b"assets/../icon.png").is_err());
    }

    #[test]
    fn runtime_assets_init_and_find() {
        let assets = vec![
            Asset {
                id: "index.html".into(),
                kind: AssetKind::Text,
                source_path: "assets/index.html".into(),
                bundle_path: "index.html".into(),
                byte_len: 0,
                hash: Hash::zero(),
                media_type: None,
            },
        ];
        let manifest = Manifest { assets };
        let runtime = RuntimeAssets::init(manifest);
        assert!(runtime.find("index.html").is_some());
        assert!(runtime.find("missing").is_none());
    }
}
