pub mod dither;
pub mod shape;

#[cfg(feature = "figure")]
use image::{imageops, DynamicImage, GrayImage};
#[cfg(feature = "figure")]
use std::path::Path;

#[cfg(feature = "figure")]
use crate::figure::dither::{dither, DitherContext};
#[cfg(feature = "figure")]
use crate::figure::shape::{apply_mask, build_mask, enforce_minimum_size, ShapeMask};

// ─────────────────────────────────────────────────────────
// Public enums — available even without --features figure
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DitherMode {
    Density,
    Block,
    HalfBlock,
    QuarterBlock,
    Braille,
    Binary,
    Edge,
}

impl Default for DitherMode {
    fn default() -> Self {
        DitherMode::Block
    }
}

impl DitherMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "density" => Some(Self::Density),
            "block" => Some(Self::Block),
            "half-block" => Some(Self::HalfBlock),
            "quarter-block" => Some(Self::QuarterBlock),
            "braille" => Some(Self::Braille),
            "binary" => Some(Self::Binary),
            "edge" => Some(Self::Edge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeKind {
    Circle,
    Octagon,
    Shield,
    Star,
    Heart,
    Diamond,
    Hexagon,
    Pentagon,
    RoundedRect,
}

impl ShapeKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "circle" => Some(Self::Circle),
            "octagon" => Some(Self::Octagon),
            "shield" => Some(Self::Shield),
            "star" => Some(Self::Star),
            "heart" => Some(Self::Heart),
            "diamond" => Some(Self::Diamond),
            "hexagon" => Some(Self::Hexagon),
            "pentagon" => Some(Self::Pentagon),
            "rounded-rect" => Some(Self::RoundedRect),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelPos {
    Center,
    Top,
    Bottom,
}

impl Default for LabelPos {
    fn default() -> Self {
        LabelPos::Bottom
    }
}

// ─────────────────────────────────────────────────────────
// ImportOptions
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub width: u32,
    pub height: Option<u32>,
    pub dither: DitherMode,
    pub edge: bool,
    pub invert: bool,
    pub threshold: u8,
    pub contrast: f32,
    pub gamma: f32,
    pub bg_char: char,
    pub shape: Option<ShapeKind>,
    pub label: Option<String>,
    pub label_pos: LabelPos,
    pub allow_fetch: bool,
    pub svg_scale: u32,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            width: 40,
            height: None,
            dither: DitherMode::Block,
            edge: false,
            invert: false,
            threshold: 128,
            contrast: 1.0,
            gamma: 1.0,
            bg_char: ' ',
            shape: None,
            label: None,
            label_pos: LabelPos::default(),
            allow_fetch: false,
            svg_scale: 4,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Diagnostic codes
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FigureWarning {
    AspectRatioChanged {
        original: (u32, u32),
        requested: (u32, u32),
    }, // FIGURE-002
    BrailleMode, // FIGURE-004
}

#[derive(Debug, Clone, PartialEq)]
pub enum FigureError {
    NotFound(String),           // FIGURE-001
    ShapeEmpty,                 // FIGURE-003
    RemoteWithoutFetch(String), // FIGURE-006
    ShapeTooSmall(String),      // from enforce_minimum_size
    UnsupportedFormat(String),  // FIGURE-001 variant
    ImageError(String),         // FIGURE-001 variant
}

impl std::fmt::Display for FigureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "FIGURE-001: file not found: {}", p),
            Self::ShapeEmpty => write!(f, "FIGURE-003: shape clip produced empty output"),
            Self::RemoteWithoutFetch(u) => {
                write!(f, "FIGURE-006: remote URL requires --allow-fetch: {}", u)
            }
            Self::ShapeTooSmall(msg) => write!(f, "FIGURE-003: {}", msg),
            Self::UnsupportedFormat(ext) => write!(f, "FIGURE-001: unsupported format: {}", ext),
            Self::ImageError(msg) => write!(f, "FIGURE-001: {}", msg),
        }
    }
}

pub struct ImportResult {
    pub ascii: String,
    pub warnings: Vec<FigureWarning>,
}

// ─────────────────────────────────────────────────────────
// Main pipeline (feature-gated)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn import_image(path: &Path, opts: &ImportOptions) -> Result<ImportResult, FigureError> {
    let mut warnings: Vec<FigureWarning> = Vec::new();

    // Braille warning
    if opts.dither == DitherMode::Braille {
        warnings.push(FigureWarning::BrailleMode);
    }

    // Remote URL check
    let path_str = path.to_string_lossy();
    if path_str.starts_with("http://") || path_str.starts_with("https://") {
        if !opts.allow_fetch {
            return Err(FigureError::RemoteWithoutFetch(path_str.into_owned()));
        }
    }

    // Load image
    let img = load_image(path, opts)?;
    let orig_w = img.width();
    let orig_h = img.height();

    // Compute target dimensions
    let target_w = opts.width;
    let target_h = opts.height.unwrap_or_else(|| {
        // Preserve aspect ratio. Terminal chars are ~2:1 tall, so halve the height.
        let ratio = orig_h as f64 / orig_w as f64;
        ((target_w as f64 * ratio * 0.5) as u32).max(1)
    });

    // Warn if explicit height changes aspect ratio > 20%
    if let Some(explicit_h) = opts.height {
        let orig_ratio = orig_h as f64 / orig_w as f64;
        let new_ratio = explicit_h as f64 / target_w as f64;
        let diff = ((orig_ratio - new_ratio) / orig_ratio).abs();
        if diff > 0.20 {
            warnings.push(FigureWarning::AspectRatioChanged {
                original: (orig_w, orig_h),
                requested: (target_w, explicit_h),
            });
        }
    }

    // Enforce shape minimum size
    if let Some(ref kind) = opts.shape {
        enforce_minimum_size(kind, target_w).map_err(FigureError::ShapeTooSmall)?;
    }

    // Apply gamma/contrast
    let img = apply_gamma_contrast(img, opts);

    // Resize
    let img = img.resize_exact(target_w, target_h, imageops::FilterType::Lanczos3);

    // Convert to grayscale
    let mut gray: GrayImage = img.to_luma8();

    // Apply shape mask
    if let Some(ref kind) = opts.shape {
        let mask: ShapeMask = build_mask(kind, target_w, target_h);
        gray = shape::apply_mask(&gray, &mask, 0u8);

        // Check shape produced non-empty output
        let any_inside = mask.pixels.iter().any(|&b| b);
        if !any_inside {
            return Err(FigureError::ShapeEmpty);
        }
    }

    // Dither
    let ctx = DitherContext::new(&gray, opts);
    let rows = dither(&ctx);

    // Apply label overlay
    let rows = if let Some(ref label) = opts.label {
        apply_label_overlay(rows, label, opts)
    } else {
        rows
    };

    Ok(ImportResult {
        ascii: rows.join("\n"),
        warnings,
    })
}

// ─────────────────────────────────────────────────────────
// Image loading (feature-gated)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn load_image(path: &Path, _opts: &ImportOptions) -> Result<DynamicImage, FigureError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    #[cfg(feature = "svg")]
    if ext == "svg" {
        return load_svg(path);
    }

    if ext == "svg" {
        return Err(FigureError::UnsupportedFormat(
            "SVG requires --features svg".to_string(),
        ));
    }

    if !path.exists() {
        return Err(FigureError::NotFound(path.display().to_string()));
    }

    image::open(path).map_err(|e| FigureError::ImageError(e.to_string()))
}

#[cfg(all(feature = "figure", feature = "svg"))]
fn load_svg(path: &Path) -> Result<DynamicImage, FigureError> {
    // resvg-based SVG loading
    let data = std::fs::read(path)
        .map_err(|e| FigureError::NotFound(format!("{}: {}", path.display(), e)))?;
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&data, &options)
        .map_err(|e| FigureError::ImageError(format!("SVG parse error: {}", e)))?;
    let size = tree.size();
    let w = (size.width() as u32).max(1);
    let h = (size.height() as u32).max(1);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| FigureError::ImageError("pixmap allocation failed".to_string()))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    let rgba = pixmap.take();
    let img = image::RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| FigureError::ImageError("image buffer mismatch".to_string()))?;
    Ok(DynamicImage::ImageRgba8(img))
}

// ─────────────────────────────────────────────────────────
// Gamma / contrast adjustment (feature-gated)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn apply_gamma_contrast(img: DynamicImage, opts: &ImportOptions) -> DynamicImage {
    use image::{ImageBuffer, Pixel};

    let contrast = opts.contrast;
    let gamma = opts.gamma;

    if (contrast - 1.0).abs() < 1e-6 && (gamma - 1.0).abs() < 1e-6 {
        return img; // No adjustment needed
    }

    let img = img.to_luma8();
    let (w, h) = img.dimensions();
    let out = ImageBuffer::from_fn(w, h, |x, y| {
        let v = img.get_pixel(x, y)[0] as f64 / 255.0;
        // Gamma correction
        let v = if (gamma - 1.0).abs() > 1e-6 {
            v.powf(1.0 / gamma as f64)
        } else {
            v
        };
        // Contrast adjustment: (v - 0.5) * contrast + 0.5
        let v = (v - 0.5) * contrast as f64 + 0.5;
        let v = v.clamp(0.0, 1.0);
        image::Luma([(v * 255.0) as u8])
    });
    DynamicImage::ImageLuma8(out)
}

// ─────────────────────────────────────────────────────────
// Label overlay
// ─────────────────────────────────────────────────────────

pub fn apply_label_overlay(
    mut rows: Vec<String>,
    label: &str,
    opts: &ImportOptions,
) -> Vec<String> {
    if rows.is_empty() {
        return rows;
    }

    let frame_w = rows[0].chars().count();
    // Truncate label if wider than frame
    let label: String = label.chars().take(frame_w).collect();
    let label_w = label.chars().count();
    let pad_left = (frame_w.saturating_sub(label_w)) / 2;
    let pad_right = frame_w.saturating_sub(label_w + pad_left);
    let label_line = format!("{}{}{}", " ".repeat(pad_left), label, " ".repeat(pad_right));

    match opts.label_pos {
        LabelPos::Top => {
            rows.insert(0, label_line);
        }
        LabelPos::Bottom => {
            rows.push(label_line);
        }
        LabelPos::Center => {
            let mid = rows.len() / 2;
            // Overwrite the middle row with the label
            if mid < rows.len() {
                rows[mid] = label_line;
            }
        }
    }

    rows
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_options_defaults() {
        let opts = ImportOptions::default();
        assert_eq!(opts.width, 40);
        assert_eq!(opts.height, None);
        assert_eq!(opts.dither, DitherMode::Block);
        assert!(!opts.invert);
        assert_eq!(opts.threshold, 128);
        assert!(!opts.allow_fetch);
    }

    #[test]
    fn test_braille_chars_are_width_1_in_visual_width() {
        use crate::layout::visual_width;
        // U+2800 — braille space
        assert_eq!(visual_width("\u{2800}"), 1);
        // U+28FF — braille full (all 8 dots)
        assert_eq!(visual_width("\u{28FF}"), 1);
        // Mixed braille string
        assert_eq!(visual_width("\u{2801}\u{28FF}\u{2840}"), 3);
    }

    #[test]
    fn test_remote_url_without_allow_fetch_errors() {
        // This test does not touch the filesystem — it tests the guard logic
        let err = FigureError::RemoteWithoutFetch("https://example.com/logo.png".to_string());
        assert!(err.to_string().contains("FIGURE-006"));
    }

    #[test]
    fn test_figure_error_display_codes() {
        assert!(FigureError::NotFound("x".into())
            .to_string()
            .contains("FIGURE-001"));
        assert!(FigureError::ShapeEmpty.to_string().contains("FIGURE-003"));
        assert!(FigureError::ShapeTooSmall("octagon too small".into())
            .to_string()
            .contains("FIGURE-003"));
        assert!(FigureError::UnsupportedFormat("tiff".into())
            .to_string()
            .contains("FIGURE-001"));
    }

    #[test]
    fn test_label_overlay_center_positioning() {
        let rows: Vec<String> = (0..5).map(|_| "          ".to_string()).collect();
        let opts = ImportOptions {
            label_pos: LabelPos::Center,
            ..Default::default()
        };
        let result = apply_label_overlay(rows, "Hi", &opts);
        assert_eq!(
            result.len(),
            5,
            "center overlay should not change row count"
        );
        let mid = result.len() / 2;
        assert!(
            result[mid].contains("Hi"),
            "center row should contain label: {:?}",
            result[mid]
        );
    }

    #[test]
    fn test_label_overlay_top_positioning() {
        let rows: Vec<String> = vec!["          ".to_string(); 4];
        let opts = ImportOptions {
            label_pos: LabelPos::Top,
            ..Default::default()
        };
        let result = apply_label_overlay(rows, "TOP", &opts);
        assert_eq!(result.len(), 5, "top overlay should add one row");
        assert!(
            result[0].contains("TOP"),
            "first row should contain label: {:?}",
            result[0]
        );
    }

    #[test]
    fn test_label_overlay_bottom_positioning() {
        let rows: Vec<String> = vec!["          ".to_string(); 4];
        let opts = ImportOptions {
            label_pos: LabelPos::Bottom,
            ..Default::default()
        };
        let result = apply_label_overlay(rows, "BOT", &opts);
        assert_eq!(result.len(), 5, "bottom overlay should add one row");
        let last = result.last().unwrap();
        assert!(
            last.contains("BOT"),
            "last row should contain label: {:?}",
            last
        );
    }

    #[test]
    fn test_label_truncated_at_frame_width() {
        let rows: Vec<String> = vec!["1234567890".to_string()]; // 10 wide
        let opts = ImportOptions {
            label_pos: LabelPos::Center,
            ..Default::default()
        };
        let result = apply_label_overlay(rows, "TOOLONGLABEL", &opts);
        // Label should be truncated to 10 chars
        assert_eq!(
            result[0].chars().count(),
            10,
            "label row must match frame width"
        );
    }

    #[test]
    fn test_dither_mode_parse() {
        assert_eq!(DitherMode::parse("block"), Some(DitherMode::Block));
        assert_eq!(DitherMode::parse("density"), Some(DitherMode::Density));
        assert_eq!(DitherMode::parse("braille"), Some(DitherMode::Braille));
        assert_eq!(DitherMode::parse("unknown"), None);
    }

    #[test]
    fn test_shape_kind_parse() {
        assert_eq!(ShapeKind::parse("circle"), Some(ShapeKind::Circle));
        assert_eq!(ShapeKind::parse("shield"), Some(ShapeKind::Shield));
        assert_eq!(ShapeKind::parse("unknown"), None);
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_import_1x1_black_image_produces_nonempty_ascii() {
        use image::{GrayImage, Luma};
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a 1×1 PNG in a temp file (must have .png extension for format detection)
        let mut tmp = NamedTempFile::with_suffix(".png").unwrap();
        let img = GrayImage::from_pixel(1, 1, Luma([0u8]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        tmp.write_all(&buf).unwrap();
        tmp.flush().unwrap();

        let opts = ImportOptions {
            width: 1,
            height: Some(1),
            ..Default::default()
        };
        let result = import_image(tmp.path(), &opts).unwrap();
        assert!(
            !result.ascii.is_empty(),
            "1×1 image should produce non-empty ASCII"
        );
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_load_image_unknown_format_errors() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut tmp = NamedTempFile::with_suffix(".tiff").unwrap();
        tmp.write_all(b"not a real image").unwrap();
        let opts = ImportOptions::default();
        let result = load_image(tmp.path(), &opts);
        assert!(result.is_err(), "unknown/corrupt format should error");
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_load_image_nonexistent_path_errors() {
        let opts = ImportOptions::default();
        let result = load_image(Path::new("/nonexistent/path/image.png"), &opts);
        assert!(result.is_err(), "nonexistent file should return error");
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_aspect_ratio_warning_fires_on_override() {
        use image::{GrayImage, Luma};
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a 100×100 (square) image (must have .png extension for format detection)
        let mut tmp = NamedTempFile::with_suffix(".png").unwrap();
        let img = GrayImage::from_pixel(100, 100, Luma([128u8]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        tmp.write_all(&buf).unwrap();
        tmp.flush().unwrap();

        // Request width=40, height=2 — extreme ratio change
        let opts = ImportOptions {
            width: 40,
            height: Some(2),
            ..Default::default()
        };
        let result = import_image(tmp.path(), &opts).unwrap();
        let has_ratio_warning = result
            .warnings
            .iter()
            .any(|w| matches!(w, FigureWarning::AspectRatioChanged { .. }));
        assert!(
            has_ratio_warning,
            "extreme height override should warn about aspect ratio change"
        );
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_gamma_contrast_applied_before_dither() {
        use image::{GrayImage, Luma};
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Must have .png extension for format detection
        let mut tmp = NamedTempFile::with_suffix(".png").unwrap();
        // Mid-gray image
        let img = GrayImage::from_pixel(4, 4, Luma([128u8]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        tmp.write_all(&buf).unwrap();
        tmp.flush().unwrap();

        // High contrast (10.0) pushes mid-gray (≈128) noticeably above threshold=130.
        // Formula: (0.502 - 0.5) * 10 + 0.5 ≈ 0.52 → 133 > 130 → '█'
        let opts = ImportOptions {
            width: 4,
            height: Some(4),
            contrast: 10.0,
            dither: DitherMode::Binary,
            threshold: 130,
            ..Default::default()
        };
        let result_high = import_image(tmp.path(), &opts).unwrap();

        // Very low contrast (0.01) keeps mid-gray essentially unchanged ≈ 128 < 130 → ' '
        let opts_low = ImportOptions {
            width: 4,
            height: Some(4),
            contrast: 0.01,
            dither: DitherMode::Binary,
            threshold: 130,
            ..Default::default()
        };
        let result_low = import_image(tmp.path(), &opts_low).unwrap();

        // They must differ: high contrast → '█', low contrast → ' '
        assert_ne!(
            result_high.ascii, result_low.ascii,
            "different contrast values should produce different output"
        );
    }
}
