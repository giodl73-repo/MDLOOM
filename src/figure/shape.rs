#[cfg(feature = "figure")]
use image::GrayImage;

use crate::figure::ShapeKind;

// ─────────────────────────────────────────────────────────
// Shape mask
// ─────────────────────────────────────────────────────────

pub struct ShapeMask {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>, // true = inside shape (render), false = background
}

impl ShapeMask {
    pub fn get(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.pixels[(y * self.width + x) as usize]
    }
}

// ─────────────────────────────────────────────────────────
// Minimum size enforcement
// ─────────────────────────────────────────────────────────

pub fn enforce_minimum_size(kind: &ShapeKind, width: u32) -> Result<(), String> {
    let (min, name) = match kind {
        ShapeKind::Octagon     => (14, "octagon"),
        ShapeKind::Circle      => (10, "circle"),
        ShapeKind::Shield      => (12, "shield"),
        ShapeKind::Star        => (8,  "star"),
        ShapeKind::Heart       => (8,  "heart"),
        ShapeKind::Diamond     => (6,  "diamond"),
        ShapeKind::Hexagon     => (10, "hexagon"),
        ShapeKind::Pentagon    => (10, "pentagon"),
        ShapeKind::RoundedRect => (6,  "rounded-rect"),
    };
    if width < min {
        Err(format!(
            "shape {:?} requires width >= {} (got {})",
            name, min, width
        ))
    } else {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────
// Mask builder
// ─────────────────────────────────────────────────────────

pub fn build_mask(kind: &ShapeKind, width: u32, height: u32) -> ShapeMask {
    let mut pixels = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            // Normalized coordinates in [-1, 1]
            let nx = (x as f64 + 0.5) / width as f64 * 2.0 - 1.0;
            let ny = (y as f64 + 0.5) / height as f64 * 2.0 - 1.0;
            let inside = match kind {
                ShapeKind::Circle      => inside_circle(nx, ny),
                ShapeKind::Octagon     => inside_octagon(nx, ny),
                ShapeKind::Shield      => inside_shield(nx, ny),
                ShapeKind::Star        => inside_star(nx, ny, 5),
                ShapeKind::Heart       => inside_heart(nx, ny),
                ShapeKind::Diamond     => inside_diamond(nx, ny),
                ShapeKind::Hexagon     => inside_hexagon(nx, ny),
                ShapeKind::Pentagon    => inside_regular_polygon(nx, ny, 5),
                ShapeKind::RoundedRect => inside_rounded_rect(nx, ny, 0.25),
            };
            pixels[(y * width + x) as usize] = inside;
        }
    }

    ShapeMask { width, height, pixels }
}

// ─────────────────────────────────────────────────────────
// Geometric predicates — normalized coords in [-1, 1]
// ─────────────────────────────────────────────────────────

fn inside_circle(nx: f64, ny: f64) -> bool {
    nx * nx + ny * ny <= 1.0
}

fn inside_octagon(nx: f64, ny: f64) -> bool {
    // Regular octagon: inscribed in unit circle, all 8 half-planes satisfied
    let ax = nx.abs();
    let ay = ny.abs();
    // Half-plane along the 8 edges: horizontal, vertical, and 45° diagonals
    ax <= 1.0 && ay <= 1.0 && ax + ay <= 1.4142135623730951
}

fn inside_shield(nx: f64, ny: f64) -> bool {
    // Upper half: rectangle; lower half: pointed bottom (tent shape)
    if ny < 0.4 {
        // Upper portion — rectangular
        nx.abs() <= 0.9 && ny >= -1.0
    } else {
        // Lower pointed portion — two converging lines meet at (0, 1)
        let frac = (ny - 0.4) / 0.6; // 0 at y=0.4, 1 at y=1.0
        nx.abs() <= 0.9 * (1.0 - frac)
    }
}

fn inside_star(nx: f64, ny: f64, points: u32) -> bool {
    // Star polygon: alternating inner (0.4) and outer (1.0) radii
    let r = (nx * nx + ny * ny).sqrt();
    if r > 1.0 { return false; }
    let angle = ny.atan2(nx); // [-π, π]
    let sector_angle = std::f64::consts::PI / points as f64;
    let sector = ((angle + std::f64::consts::PI) / sector_angle) as u32 % 2;
    let inner_r = 0.42;
    let outer_r = 1.0;
    let sector_frac = ((angle + std::f64::consts::PI) % sector_angle) / sector_angle;
    // Interpolate radius threshold at this angle
    let threshold = if sector == 0 {
        outer_r * (1.0 - sector_frac) + inner_r * sector_frac
    } else {
        inner_r * (1.0 - sector_frac) + outer_r * sector_frac
    };
    r <= threshold
}

fn inside_heart(nx: f64, ny: f64) -> bool {
    // Heart: (x²+y²−1)³ < x²y³
    let x = nx * 0.9;
    let y = -ny * 0.9; // flip so y increases downward in image coords
    let lhs = (x*x + y*y - 1.0).powi(3);
    let rhs = x*x * y*y*y;
    lhs < rhs
}

fn inside_diamond(nx: f64, ny: f64) -> bool {
    nx.abs() + ny.abs() <= 1.0
}

fn inside_hexagon(nx: f64, ny: f64) -> bool {
    // Regular hexagon (flat-top)
    let ax = nx.abs();
    let ay = ny.abs();
    ax <= 1.0 && ay <= 0.866_025_4 && ax * 0.5 + ay * 0.866_025_4 <= 0.866_025_4 * 1.0
}

fn inside_regular_polygon(nx: f64, ny: f64, sides: u32) -> bool {
    // Regular polygon via inscribed-radius test
    let r = (nx * nx + ny * ny).sqrt();
    if r > 1.0 { return false; }
    let angle = ny.atan2(nx);
    let sector = (2.0 * std::f64::consts::PI) / sides as f64;
    let offset = angle.rem_euclid(sector) - sector / 2.0;
    let inradius = offset.cos();
    r <= inradius
}

fn inside_rounded_rect(nx: f64, ny: f64, corner_r: f64) -> bool {
    let ix = nx.abs().max(0.0) - (1.0 - corner_r);
    let iy = ny.abs().max(0.0) - (1.0 - corner_r);
    let dx = ix.max(0.0);
    let dy = iy.max(0.0);
    dx * dx + dy * dy <= corner_r * corner_r
        || (nx.abs() <= 1.0 && ny.abs() <= 1.0 - corner_r)
        || (ny.abs() <= 1.0 && nx.abs() <= 1.0 - corner_r)
}

// ─────────────────────────────────────────────────────────
// Mask application
// ─────────────────────────────────────────────────────────

#[cfg(feature = "figure")]
pub fn apply_mask(img: &GrayImage, mask: &ShapeMask, bg: u8) -> GrayImage {
    use image::Luma;
    let mut out = GrayImage::new(img.width(), img.height());
    for y in 0..img.height() {
        for x in 0..img.width() {
            let pixel = if mask.get(x, y) {
                *img.get_pixel(x, y)
            } else {
                Luma([bg])
            };
            out.put_pixel(x, y, pixel);
        }
    }
    out
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octagon_mask_rejects_width_13() {
        let err = enforce_minimum_size(&ShapeKind::Octagon, 13);
        assert!(err.is_err(), "octagon width 13 should be rejected");
    }

    #[test]
    fn test_circle_mask_rejects_width_9() {
        let err = enforce_minimum_size(&ShapeKind::Circle, 9);
        assert!(err.is_err(), "circle width 9 should be rejected");
    }

    #[test]
    fn test_shield_mask_rejects_width_11() {
        let err = enforce_minimum_size(&ShapeKind::Shield, 11);
        assert!(err.is_err(), "shield width 11 should be rejected");
    }

    #[test]
    fn test_star_mask_rejects_width_7() {
        let err = enforce_minimum_size(&ShapeKind::Star, 7);
        assert!(err.is_err(), "star width 7 should be rejected");
    }

    #[test]
    fn test_octagon_mask_at_minimum_size_14() {
        assert!(enforce_minimum_size(&ShapeKind::Octagon, 14).is_ok());
    }

    #[test]
    fn test_circle_mask_at_minimum_size_10() {
        assert!(enforce_minimum_size(&ShapeKind::Circle, 10).is_ok());
    }

    #[test]
    fn test_circle_mask_center_pixel_inside() {
        let mask = build_mask(&ShapeKind::Circle, 20, 10);
        // Center pixel should be inside
        assert!(mask.get(10, 5), "center pixel should be inside circle");
    }

    #[test]
    fn test_circle_mask_corner_pixel_outside() {
        let mask = build_mask(&ShapeKind::Circle, 20, 10);
        assert!(!mask.get(0, 0), "corner pixel should be outside circle");
        assert!(!mask.get(19, 9), "corner pixel should be outside circle");
    }

    #[test]
    fn test_octagon_mask_has_nonzero_coverage() {
        let mask = build_mask(&ShapeKind::Octagon, 20, 10);
        let count = mask.pixels.iter().filter(|&&b| b).count();
        assert!(count > 0, "octagon mask should have non-zero coverage");
        // Octagon should cover less than full rectangle
        assert!(count < (20 * 10) as usize, "octagon should not fill entire frame");
    }

    #[test]
    fn test_shield_mask_top_center_inside() {
        let mask = build_mask(&ShapeKind::Shield, 20, 20);
        assert!(mask.get(10, 2), "top-center should be inside shield");
    }

    #[test]
    fn test_diamond_mask_center_inside_corners_outside() {
        let mask = build_mask(&ShapeKind::Diamond, 20, 20);
        assert!(mask.get(10, 10), "center inside diamond");
        assert!(!mask.get(0, 0), "corner outside diamond");
        assert!(!mask.get(19, 19), "corner outside diamond");
    }

    #[test]
    fn test_star_mask_center_inside() {
        let mask = build_mask(&ShapeKind::Star, 20, 20);
        assert!(mask.get(10, 10), "star center should be inside");
    }

    #[test]
    fn test_build_mask_correct_dimensions() {
        let mask = build_mask(&ShapeKind::Circle, 16, 8);
        assert_eq!(mask.width, 16);
        assert_eq!(mask.height, 8);
        assert_eq!(mask.pixels.len(), 16 * 8);
    }

    #[cfg(feature = "figure")]
    #[test]
    fn test_apply_mask_sets_background_outside() {
        use image::{GrayImage, Luma};
        let img = GrayImage::from_pixel(20, 20, Luma([200u8]));
        let mask = build_mask(&ShapeKind::Circle, 20, 20);
        let result = apply_mask(&img, &mask, 0u8);
        // Corner pixels outside circle should be 0 (background)
        assert_eq!(result.get_pixel(0, 0)[0], 0, "corner should be background");
        // Center pixel inside circle should retain original value
        assert_eq!(result.get_pixel(10, 10)[0], 200, "center should retain value");
    }
}
