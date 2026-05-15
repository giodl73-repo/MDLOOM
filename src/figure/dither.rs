#[cfg(feature = "figure")]
use crate::figure::{DitherMode, ImportOptions};
#[allow(dead_code)]
#[cfg(feature = "figure")]
use image::GrayImage;

// ─────────────────────────────────────────────────────────
// DitherContext (only available with feature flag)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub struct DitherContext<'a> {
    pub gray: &'a GrayImage,
    pub width: u32,
    pub height: u32,
    pub opts: &'a ImportOptions,
}

#[cfg(feature = "figure")]
impl<'a> DitherContext<'a> {
    pub fn new(gray: &'a GrayImage, opts: &'a ImportOptions) -> Self {
        Self {
            width: gray.width(),
            height: gray.height(),
            gray,
            opts,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Dispatch
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn dither(ctx: &DitherContext) -> Vec<String> {
    match ctx.opts.dither {
        DitherMode::Density => dither_density(ctx),
        DitherMode::Block => dither_block(ctx),
        DitherMode::HalfBlock => dither_half_block(ctx),
        DitherMode::QuarterBlock => dither_quarter_block(ctx),
        DitherMode::Braille => dither_braille(ctx),
        DitherMode::Binary => dither_binary(ctx),
        DitherMode::Edge => dither_edge(ctx),
    }
}

// ─────────────────────────────────────────────────────────
// Brightness helper
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
fn luma(ctx: &DitherContext, x: u32, y: u32) -> u8 {
    if x >= ctx.width || y >= ctx.height {
        return 0;
    }
    let v = ctx.gray.get_pixel(x, y)[0];
    if ctx.opts.invert {
        255 - v
    } else {
        v
    }
}

// ─────────────────────────────────────────────────────────
// Density dither — " .:-=+*#%@"
// ─────────────────────────────────────────────────────────

#[allow(dead_code)]
const DENSITY_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

#[cfg(feature = "figure")]
pub fn dither_density(ctx: &DitherContext) -> Vec<String> {
    (0..ctx.height)
        .map(|y| {
            (0..ctx.width)
                .map(|x| {
                    let v = luma(ctx, x, y) as usize;
                    let idx = v * (DENSITY_CHARS.len() - 1) / 255;
                    DENSITY_CHARS[idx]
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Block dither — " ░▒▓█"
// ─────────────────────────────────────────────────────────

#[allow(dead_code)]
const BLOCK_CHARS: &[char] = &[' ', '░', '▒', '▓', '█'];

#[cfg(feature = "figure")]
pub fn dither_block(ctx: &DitherContext) -> Vec<String> {
    (0..ctx.height)
        .map(|y| {
            (0..ctx.width)
                .map(|x| {
                    let v = luma(ctx, x, y) as usize;
                    let idx = v * (BLOCK_CHARS.len() - 1) / 255;
                    BLOCK_CHARS[idx]
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Half-block dither — 2 image rows per output row
// Uses upper/lower half-block characters: ' ', '▀', '▄', '█'
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn dither_half_block(ctx: &DitherContext) -> Vec<String> {
    // Each output row consumes 2 image rows
    let out_height = (ctx.height + 1) / 2;
    (0..out_height)
        .map(|row| {
            let y_top = row * 2;
            let y_bot = row * 2 + 1;
            (0..ctx.width)
                .map(|x| {
                    let top = luma(ctx, x, y_top) >= 128;
                    let bot = if y_bot < ctx.height {
                        luma(ctx, x, y_bot) >= 128
                    } else {
                        false
                    };
                    match (top, bot) {
                        (false, false) => ' ',
                        (true, false) => '▀',
                        (false, true) => '▄',
                        (true, true) => '█',
                    }
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Quarter-block dither — 2×2 image pixels per output char
//
// Each output cell carries one of the 16 block-quadrant glyphs in the 2×2
// pattern space (' '▘▝▀▖▌▞▛▗▚▐▜▄▙▟█). Bit assignment:
//   bit 0 = top-left quadrant, bit 1 = top-right,
//   bit 2 = bottom-left,        bit 3 = bottom-right
// A quadrant is "on" iff its source pixel's luma ≥ threshold. Doubling
// effective resolution in BOTH axes vs full-block at the cost of contrast
// (each quadrant is binary).
// ─────────────────────────────────────────────────────────

#[allow(dead_code)]
const QUARTER_BLOCK_CHARS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];

#[cfg(feature = "figure")]
pub fn dither_quarter_block(ctx: &DitherContext) -> Vec<String> {
    let out_w = (ctx.width + 1) / 2;
    let out_h = (ctx.height + 1) / 2;
    let threshold = ctx.opts.threshold;

    (0..out_h)
        .map(|row| {
            (0..out_w)
                .map(|col| {
                    let px = col * 2;
                    let py = row * 2;
                    let q = |dx: u32, dy: u32| -> u8 {
                        let x = px + dx;
                        let y = py + dy;
                        if x < ctx.width && y < ctx.height && luma(ctx, x, y) >= threshold {
                            1
                        } else {
                            0
                        }
                    };
                    let bits = q(0, 0)         // bit 0 = TL
                     | (q(1, 0) << 1)  // bit 1 = TR
                     | (q(0, 1) << 2)  // bit 2 = BL
                     | (q(1, 1) << 3); // bit 3 = BR
                    QUARTER_BLOCK_CHARS[bits as usize]
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Braille dither — U+2800-U+28FF, 2×4 cells
// Each braille char covers a 2×4 pixel block.
// Dot layout (Unicode braille bit positions):
//   dot 1 (bit 0) = (0,0)    dot 4 (bit 3) = (1,0)
//   dot 2 (bit 1) = (0,1)    dot 5 (bit 4) = (1,1)
//   dot 3 (bit 2) = (0,2)    dot 6 (bit 5) = (1,2)
//   dot 7 (bit 6) = (0,3)    dot 8 (bit 7) = (1,3)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn dither_braille(ctx: &DitherContext) -> Vec<String> {
    // Output dimensions: ceil(width/2) chars wide, ceil(height/4) chars tall
    let out_w = (ctx.width + 1) / 2;
    let out_h = (ctx.height + 3) / 4;
    let threshold = ctx.opts.threshold;

    (0..out_h)
        .map(|row| {
            (0..out_w)
                .map(|col| {
                    let px = col * 2;
                    let py = row * 4;

                    // Dot positions within the 2×4 cell
                    const DOT_X: [u32; 8] = [0, 0, 0, 1, 1, 1, 0, 1];
                    const DOT_Y: [u32; 8] = [0, 1, 2, 0, 1, 2, 3, 3];

                    let mut bits: u8 = 0;
                    for dot in 0..8u8 {
                        let x = px + DOT_X[dot as usize];
                        let y = py + DOT_Y[dot as usize];
                        if x < ctx.width && y < ctx.height && luma(ctx, x, y) >= threshold {
                            bits |= 1 << dot;
                        }
                    }

                    // U+2800 is the base braille pattern (all dots off)
                    char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Binary dither — "█" or " " based on threshold
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn dither_binary(ctx: &DitherContext) -> Vec<String> {
    let threshold = ctx.opts.threshold;
    (0..ctx.height)
        .map(|y| {
            (0..ctx.width)
                .map(|x| {
                    if luma(ctx, x, y) >= threshold {
                        '█'
                    } else {
                        ' '
                    }
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Edge dither — Sobel operator → "─│╱╲ "
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn dither_edge(ctx: &DitherContext) -> Vec<String> {
    let threshold = ctx.opts.threshold as i32;

    (0..ctx.height)
        .map(|y| {
            (0..ctx.width)
                .map(|x| {
                    // 3×3 Sobel kernels
                    let get = |dx: i32, dy: i32| -> i32 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= ctx.width as i32 || ny >= ctx.height as i32 {
                            return 0;
                        }
                        luma(ctx, nx as u32, ny as u32) as i32
                    };

                    let gx = -get(-1, -1) + get(1, -1) - 2 * get(-1, 0) + 2 * get(1, 0)
                        - get(-1, 1)
                        + get(1, 1);
                    let gy = -get(-1, -1) - 2 * get(0, -1) - get(1, -1)
                        + get(-1, 1)
                        + 2 * get(0, 1)
                        + get(1, 1);

                    let mag = ((gx * gx + gy * gy) as f64).sqrt() as i32;
                    if mag < threshold {
                        return ' ';
                    }

                    // Choose character based on angle
                    let angle = (gy as f64).atan2(gx as f64).to_degrees();
                    // Normalize to [0, 180)
                    let angle = ((angle % 180.0) + 180.0) % 180.0;

                    if angle < 22.5 || angle >= 157.5 {
                        '─'
                    }
                    // horizontal
                    else if angle < 67.5 {
                        '╱'
                    }
                    // diagonal /
                    else if angle < 112.5 {
                        '│'
                    }
                    // vertical
                    else {
                        '╲'
                    } // diagonal \
                })
                .collect()
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(all(test, feature = "figure"))]
mod tests {
    use super::*;
    use crate::figure::{DitherMode, ImportOptions};
    use image::{GrayImage, Luma};

    fn opts_with_dither(mode: DitherMode) -> ImportOptions {
        ImportOptions {
            dither: mode,
            ..Default::default()
        }
    }

    fn solid_gray(w: u32, h: u32, val: u8) -> GrayImage {
        GrayImage::from_pixel(w, h, Luma([val]))
    }

    fn gradient_gray(w: u32, h: u32) -> GrayImage {
        let mut img = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let v = (x * 255 / w.max(1)) as u8;
                img.put_pixel(x, y, Luma([v]));
            }
        }
        img
    }

    // ── density ─────────────────────────────────────────

    #[test]
    fn test_density_dither_produces_ascii_chars() {
        let img = gradient_gray(10, 5);
        let opts = opts_with_dither(DitherMode::Density);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_density(&ctx);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].chars().count(), 10);
        // All chars must be in the density palette
        let palette: std::collections::HashSet<char> = DENSITY_CHARS.iter().cloned().collect();
        for row in &rows {
            for c in row.chars() {
                assert!(palette.contains(&c), "unexpected char {:?}", c);
            }
        }
    }

    #[test]
    fn test_density_darkest_is_space() {
        let img = solid_gray(4, 4, 0);
        let opts = opts_with_dither(DitherMode::Density);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_density(&ctx);
        for row in &rows {
            assert!(
                row.chars().all(|c| c == ' '),
                "black image should be spaces: {:?}",
                row
            );
        }
    }

    #[test]
    fn test_density_brightest_is_at() {
        let img = solid_gray(4, 4, 255);
        let opts = opts_with_dither(DitherMode::Density);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_density(&ctx);
        for row in &rows {
            assert!(
                row.chars().all(|c| c == '@'),
                "white image should be @: {:?}",
                row
            );
        }
    }

    // ── block ────────────────────────────────────────────

    #[test]
    fn test_block_dither_uses_block_chars() {
        let img = gradient_gray(10, 5);
        let opts = opts_with_dither(DitherMode::Block);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_block(&ctx);
        let palette: std::collections::HashSet<char> = BLOCK_CHARS.iter().cloned().collect();
        for row in &rows {
            for c in row.chars() {
                assert!(palette.contains(&c), "unexpected char {:?}", c);
            }
        }
    }

    #[test]
    fn test_block_black_is_space() {
        let img = solid_gray(4, 4, 0);
        let opts = opts_with_dither(DitherMode::Block);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_block(&ctx);
        for row in &rows {
            assert!(row.chars().all(|c| c == ' '), "black→space: {:?}", row);
        }
    }

    #[test]
    fn test_block_white_is_full_block() {
        let img = solid_gray(4, 4, 255);
        let opts = opts_with_dither(DitherMode::Block);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_block(&ctx);
        for row in &rows {
            assert!(row.chars().all(|c| c == '█'), "white→█: {:?}", row);
        }
    }

    // ── half-block ───────────────────────────────────────

    #[test]
    fn test_half_block_reduces_height_by_half() {
        let img = solid_gray(10, 8, 200);
        let opts = opts_with_dither(DitherMode::HalfBlock);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_half_block(&ctx);
        assert_eq!(rows.len(), 4, "8 image rows → 4 output rows");
    }

    #[test]
    fn test_half_block_odd_height() {
        let img = solid_gray(10, 7, 200);
        let opts = opts_with_dither(DitherMode::HalfBlock);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_half_block(&ctx);
        assert_eq!(rows.len(), 4, "7 image rows → 4 output rows (ceiling)");
    }

    // ── quarter-block ────────────────────────────────────

    #[test]
    fn test_quarter_block_output_dimensions() {
        // 8×6 input → 4×3 output (ceil halves on both axes).
        let img = solid_gray(8, 6, 200);
        let opts = ImportOptions {
            dither: DitherMode::QuarterBlock,
            threshold: 128,
            ..Default::default()
        };
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_quarter_block(&ctx);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].chars().count(), 4);
    }

    #[test]
    fn test_quarter_block_white_is_full() {
        let img = solid_gray(4, 4, 255);
        let opts = ImportOptions {
            dither: DitherMode::QuarterBlock,
            threshold: 128,
            ..Default::default()
        };
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_quarter_block(&ctx);
        for row in &rows {
            assert!(
                row.chars().all(|c| c == '█'),
                "all white → full block: {:?}",
                row
            );
        }
    }

    #[test]
    fn test_quarter_block_black_is_space() {
        let img = solid_gray(4, 4, 0);
        let opts = ImportOptions {
            dither: DitherMode::QuarterBlock,
            threshold: 128,
            ..Default::default()
        };
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_quarter_block(&ctx);
        for row in &rows {
            assert!(
                row.chars().all(|c| c == ' '),
                "all black → space: {:?}",
                row
            );
        }
    }

    #[test]
    fn test_quarter_block_each_quadrant_distinct() {
        // 2×2 image with each pixel a different on/off pattern → each output cell
        // carries the matching glyph. Use 4 separate 2×2 images to test individual
        // quadrants rather than tiling.
        // TL only: pixel (0,0) bright, others dark.
        for &(pos, expected, name) in &[
            ((0u32, 0u32), '▘', "TL"),
            ((1, 0), '▝', "TR"),
            ((0, 1), '▖', "BL"),
            ((1, 1), '▗', "BR"),
        ] {
            let mut img = GrayImage::new(2, 2);
            for y in 0..2 {
                for x in 0..2 {
                    img.put_pixel(x, y, Luma([0u8]));
                }
            }
            img.put_pixel(pos.0, pos.1, Luma([255u8]));
            let opts = ImportOptions {
                dither: DitherMode::QuarterBlock,
                threshold: 128,
                ..Default::default()
            };
            let ctx = DitherContext::new(&img, &opts);
            let rows = dither_quarter_block(&ctx);
            let got = rows[0].chars().next().unwrap();
            assert_eq!(got, expected, "{} → {:?}, got {:?}", name, expected, got);
        }
    }

    // ── braille ─────────────────────────────────────────

    #[test]
    fn test_braille_dither_uses_braille_range() {
        let img = gradient_gray(20, 16);
        let opts = opts_with_dither(DitherMode::Braille);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_braille(&ctx);
        // All non-space chars must be in U+2800..=U+28FF
        for row in &rows {
            for c in row.chars() {
                let cp = c as u32;
                assert!(
                    c == ' ' || (0x2800..=0x28FF).contains(&cp),
                    "unexpected braille char U+{:04X}",
                    cp
                );
            }
        }
    }

    #[test]
    fn test_braille_output_dimensions() {
        // 20 wide → 10 output cols; 16 tall → 4 output rows
        let img = solid_gray(20, 16, 128);
        let opts = opts_with_dither(DitherMode::Braille);
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_braille(&ctx);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].chars().count(), 10);
    }

    // ── binary ───────────────────────────────────────────

    #[test]
    fn test_binary_dither_threshold_splits_black_white() {
        // All pixels at 200 with threshold 128 → all '█'
        let img = solid_gray(6, 4, 200);
        let opts = ImportOptions {
            dither: DitherMode::Binary,
            threshold: 128,
            ..Default::default()
        };
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_binary(&ctx);
        for row in &rows {
            assert!(
                row.chars().all(|c| c == '█'),
                "above threshold → █: {:?}",
                row
            );
        }
        // All pixels at 50 → all ' '
        let img2 = solid_gray(6, 4, 50);
        let ctx2 = DitherContext::new(&img2, &opts);
        let rows2 = dither_binary(&ctx2);
        for row in &rows2 {
            assert!(
                row.chars().all(|c| c == ' '),
                "below threshold → space: {:?}",
                row
            );
        }
    }

    // ── invert ───────────────────────────────────────────

    #[test]
    fn test_dither_invert_reverses_brightness() {
        let img = solid_gray(4, 4, 255);
        let opts_normal = ImportOptions {
            dither: DitherMode::Block,
            invert: false,
            ..Default::default()
        };
        let opts_invert = ImportOptions {
            dither: DitherMode::Block,
            invert: true,
            ..Default::default()
        };
        let ctx_n = DitherContext::new(&img, &opts_normal);
        let ctx_i = DitherContext::new(&img, &opts_invert);
        let rows_n = dither_block(&ctx_n);
        let rows_i = dither_block(&ctx_i);
        // White image normal → '█'; inverted → ' '
        assert!(
            rows_n[0].chars().all(|c| c == '█'),
            "normal white: {:?}",
            rows_n[0]
        );
        assert!(
            rows_i[0].chars().all(|c| c == ' '),
            "inverted white: {:?}",
            rows_i[0]
        );
    }

    // ── edge ─────────────────────────────────────────────

    #[test]
    fn test_edge_dither_detects_edges() {
        // Create an image with a hard vertical edge in the middle
        let mut img = GrayImage::new(20, 10);
        for y in 0..10 {
            for x in 0..20 {
                let v = if x < 10 { 0u8 } else { 255u8 };
                img.put_pixel(x, y, Luma([v]));
            }
        }
        let opts = ImportOptions {
            dither: DitherMode::Edge,
            threshold: 50,
            ..Default::default()
        };
        let ctx = DitherContext::new(&img, &opts);
        let rows = dither_edge(&ctx);
        assert_eq!(rows.len(), 10);
        // At the edge (x=9 or x=10), expect a non-space char in at least one row
        let has_edge_char = rows.iter().any(|row| {
            row.chars()
                .enumerate()
                .any(|(i, c)| (i == 9 || i == 10) && c != ' ')
        });
        assert!(
            has_edge_char,
            "should detect edge between dark and bright regions"
        );
    }
}

#[cfg(all(test, not(feature = "figure")))]
mod no_feature_tests {
    // Tests that run without --features figure — no image types available
    #[test]
    fn test_dither_mode_variants_exist() {
        use crate::figure::DitherMode;
        // Ensure variants compile without the feature
        let _modes = [
            DitherMode::Density,
            DitherMode::Block,
            DitherMode::HalfBlock,
            DitherMode::QuarterBlock,
            DitherMode::Braille,
            DitherMode::Binary,
            DitherMode::Edge,
        ];
    }
}
