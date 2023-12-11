use crate::{display::number, model::Day, units::Units};

pub fn sparkline(values: &[Option<f64>], width: usize) -> String {
    let width = width.min(120).min(values.len());
    if width == 0 { return String::new(); }
    let valid: Vec<f64> = values.iter().flatten().copied().filter(|n| n.is_finite()).collect();
    let min = valid.iter().copied().fold(f64::INFINITY, f64::min);
    let max = valid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let glyphs = ['▁','▂','▃','▄','▅','▆','▇','█'];
    (0..width).map(|i| {
        let bucket = &values[i * values.len() / width..(i + 1) * values.len() / width];
        let valid: Vec<f64> = bucket.iter().flatten().copied().filter(|n| n.is_finite()).collect();
        if valid.is_empty() { return '·'; }
        if (max - min).abs() < f64::EPSILON { return '▄'; }
        let mean = valid.iter().sum::<f64>() / valid.len() as f64;
        glyphs[(((mean-min)/(max-min)*7.0).round() as usize).min(7)]
    }).collect()
}
pub fn temperature(days: &[Day], units: Units) -> String {
    let highs: Vec<_> = days.iter().map(|d| d.high).collect();
    let min = highs.iter().flatten().copied().filter(|v| v.is_finite()).reduce(f64::min);
    let max = highs.iter().flatten().copied().filter(|v| v.is_finite()).reduce(f64::max);
    format!("\nDaily high temperatures ({}): {}\n{} … {} {} · · = missing\n", units.temperature_label(), sparkline(&highs, 64), number(min), number(max), units.temperature_label())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparkline_handles_negatives_constants_and_missing_values() {
        assert_eq!(sparkline(&[Some(-10.0), None, Some(0.0), Some(10.0)], 60), "▁·▅█");
        assert_eq!(sparkline(&[Some(2.0), Some(2.0)], 60), "▄▄");
        assert_eq!(sparkline(&[None, Some(f64::NAN)], 60), "··");
        assert_eq!(sparkline(&[], 60), "");
        assert_eq!(sparkline(&[Some(1.0)], 0), "");
    }
    #[test]
    fn downsampling_honors_terminal_width() {
        let values = vec![Some(1.0); 500];
        assert_eq!(sparkline(&values, 30).chars().count(), 30);
        assert_eq!(sparkline(&values, 9999).chars().count(), 120);
    }
}
