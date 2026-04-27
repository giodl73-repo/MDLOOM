use super::ElementConfig;

const SPARK_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Map a normalized value [0.0, 1.0] to one of 8 block characters.
/// 0.0 → '▁', 1.0 → '█'.
pub fn level_char(normalized: f64) -> char {
    let idx = (normalized * 7.0).round() as usize;
    SPARK_CHARS[idx.min(7)]
}

/// Aggregate a series into `width` buckets by taking the mean of each bucket.
fn bucket_series(series: &[f64], width: usize) -> Vec<f64> {
    if series.is_empty() || width == 0 {
        return vec![0.0; width];
    }
    let n = series.len();
    (0..width).map(|i| {
        // Evenly-spaced buckets: bucket i covers [i*n/width, (i+1)*n/width)
        let start = i * n / width;
        let end = ((i + 1) * n / width).min(n);
        if start >= end {
            series[start.min(n - 1)]
        } else {
            let sum: f64 = series[start..end].iter().sum();
            sum / (end - start) as f64
        }
    }).collect()
}

/// Repeat-fill a series to exactly `width` values.
fn repeat_fill(series: &[f64], width: usize) -> Vec<f64> {
    if series.is_empty() {
        return vec![0.0; width];
    }
    (0..width).map(|i| series[i % series.len()]).collect()
}

/// Render a series of values as a sparkline string of exactly cfg.width characters.
/// - width < series.len(): bucket-aggregate (mean per bucket)
/// - width > series.len(): repeat-fill
/// - all equal: all '▁' (min level)
pub fn render_sparkline(series: &[f64], cfg: &ElementConfig) -> String {
    let width = cfg.width;

    let working: Vec<f64> = if series.is_empty() {
        vec![0.0; width]
    } else if series.len() == width {
        series.to_vec()
    } else if series.len() > width {
        bucket_series(series, width)
    } else {
        repeat_fill(series, width)
    };

    let min = working.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = working.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    working.iter().map(|&v| {
        if range == 0.0 {
            '▁' // all equal → lowest level
        } else {
            level_char((v - min) / range)
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementConfig, ElementKind};

    fn spark_cfg(width: usize) -> ElementConfig {
        ElementConfig { kind: ElementKind::Sparkline, width, ..Default::default() }
    }

    #[test]
    fn test_level_char_zero() {
        assert_eq!(level_char(0.0), '▁');
    }

    #[test]
    fn test_level_char_one() {
        assert_eq!(level_char(1.0), '█');
    }

    #[test]
    fn test_level_char_half() {
        let c = level_char(0.5);
        assert!(SPARK_CHARS.contains(&c));
        // 0.5 * 7 = 3.5 → rounds to 4 → '▅'
        assert_eq!(c, '▅');
    }

    #[test]
    fn test_render_sparkline_min_to_lowest() {
        let series = vec![0.0, 5.0, 10.0];
        let out = render_sparkline(&series, &spark_cfg(3));
        assert_eq!(out.chars().next().unwrap(), '▁');
    }

    #[test]
    fn test_render_sparkline_max_to_highest() {
        let series = vec![0.0, 5.0, 10.0];
        let out = render_sparkline(&series, &spark_cfg(3));
        assert_eq!(out.chars().last().unwrap(), '█');
    }

    #[test]
    fn test_render_sparkline_all_equal() {
        let series = vec![5.0; 5];
        let out = render_sparkline(&series, &spark_cfg(5));
        assert!(out.chars().all(|c| c == '▁'), "all-equal: {:?}", out);
    }

    #[test]
    fn test_render_sparkline_exact_length() {
        let series = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let out = render_sparkline(&series, &spark_cfg(5));
        assert_eq!(out.chars().count(), 5);
    }

    #[test]
    fn test_render_sparkline_bucket_longer_series() {
        let series: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let out = render_sparkline(&series, &spark_cfg(5));
        assert_eq!(out.chars().count(), 5);
        // First bucket (0,1) should be lower than last bucket (8,9)
        let chars: Vec<char> = out.chars().collect();
        let first_idx = SPARK_CHARS.iter().position(|&c| c == chars[0]).unwrap();
        let last_idx = SPARK_CHARS.iter().position(|&c| c == *chars.last().unwrap()).unwrap();
        assert!(first_idx < last_idx, "first bucket should be lower than last: {:?}", out);
    }

    #[test]
    fn test_render_sparkline_repeat_fill() {
        let series = vec![1.0, 5.0, 3.0];
        let out = render_sparkline(&series, &spark_cfg(6));
        assert_eq!(out.chars().count(), 6);
    }

    #[test]
    fn test_render_sparkline_width_1() {
        let series = vec![1.0, 5.0, 3.0];
        let out = render_sparkline(&series, &spark_cfg(1));
        assert_eq!(out.chars().count(), 1);
        assert!(SPARK_CHARS.contains(&out.chars().next().unwrap()));
    }

    #[test]
    fn test_bucket_series_even() {
        // 4 values into 2 buckets: means of [0,1] and [2,3]
        let series = vec![0.0, 2.0, 6.0, 8.0];
        let bucketed = bucket_series(&series, 2);
        assert_eq!(bucketed.len(), 2);
        assert!((bucketed[0] - 1.0).abs() < 0.01, "first bucket mean: {}", bucketed[0]);
        assert!((bucketed[1] - 7.0).abs() < 0.01, "second bucket mean: {}", bucketed[1]);
    }

    #[test]
    fn test_repeat_fill() {
        let series = vec![1.0, 2.0];
        let filled = repeat_fill(&series, 5);
        assert_eq!(filled, vec![1.0, 2.0, 1.0, 2.0, 1.0]);
    }
}
