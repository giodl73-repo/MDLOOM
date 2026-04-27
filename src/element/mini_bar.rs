use super::ElementConfig;

/// Render a proportional bar of exactly cfg.width characters.
/// fill_count = round(val / max * width), clamped to [0, width].
/// fill_count chars of cfg.fill_char, then (width - fill_count) of cfg.empty_char.
pub fn render_mini_bar(val: f64, max: f64, cfg: &ElementConfig) -> String {
    let width = cfg.width;
    let proportion = if max == 0.0 { 0.0 } else { (val / max).clamp(0.0, 1.0) };
    let fill_count = (proportion * width as f64).round() as usize;
    let fill_count = fill_count.min(width);
    let empty_count = width - fill_count;

    let mut s = String::with_capacity(width);
    for _ in 0..fill_count {
        s.push(cfg.fill_char);
    }
    for _ in 0..empty_count {
        s.push(cfg.empty_char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementConfig, ElementKind};

    fn bar_cfg(width: usize, max: f64) -> ElementConfig {
        ElementConfig { kind: ElementKind::MiniBar, width, max: Some(max), ..Default::default() }
    }

    #[test]
    fn test_half_fill() {
        let cfg = bar_cfg(10, 100.0);
        let out = render_mini_bar(50.0, 100.0, &cfg);
        assert_eq!(out, "█████░░░░░");
    }

    #[test]
    fn test_full_fill() {
        let cfg = bar_cfg(8, 200.0);
        let out = render_mini_bar(200.0, 200.0, &cfg);
        assert_eq!(out, "████████");
    }

    #[test]
    fn test_zero_fill() {
        let cfg = bar_cfg(8, 100.0);
        let out = render_mini_bar(0.0, 100.0, &cfg);
        assert_eq!(out, "░░░░░░░░");
    }

    #[test]
    fn test_overflow_clamped_to_full() {
        let cfg = bar_cfg(8, 100.0);
        let out = render_mini_bar(300.0, 100.0, &cfg);
        assert_eq!(out, "████████");
    }

    #[test]
    fn test_exact_length() {
        let cfg = bar_cfg(20, 200.0);
        let out = render_mini_bar(138.0, 200.0, &cfg);
        assert_eq!(out.chars().count(), 20);
    }

    #[test]
    fn test_custom_fill_chars() {
        let cfg = ElementConfig {
            kind: ElementKind::MiniBar, width: 5, max: Some(100.0),
            fill_char: '#', empty_char: '-',
            ..Default::default()
        };
        let out = render_mini_bar(60.0, 100.0, &cfg);
        // 60% of 5 = 3 filled, 2 empty
        assert_eq!(out, "###--");
    }

    #[test]
    fn test_zero_max_returns_empty() {
        let cfg = bar_cfg(5, 0.0);
        let out = render_mini_bar(50.0, 0.0, &cfg);
        assert_eq!(out.chars().count(), 5);
        // All empty when max = 0
        assert!(out.chars().all(|c| c == '░'), "output: {:?}", out);
    }

    #[test]
    fn test_width_1() {
        let cfg = bar_cfg(1, 100.0);
        let out_full = render_mini_bar(100.0, 100.0, &cfg);
        let out_empty = render_mini_bar(0.0, 100.0, &cfg);
        assert_eq!(out_full, "█");
        assert_eq!(out_empty, "░");
    }
}
