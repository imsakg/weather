use crate::{display::number, model::Day, units::Units};

#[derive(Debug)]
pub struct Summary {
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub rain_total: Option<f64>,
    pub rain_days: usize,
    pub total_days: usize,
}
pub fn summarize(days: &[Day]) -> Summary {
    let rain: Vec<f64> = days
        .iter()
        .filter_map(|d| d.rain)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .collect();
    Summary {
        high: days
            .iter()
            .filter_map(|d| d.high)
            .filter(|v| v.is_finite())
            .reduce(f64::max),
        low: days
            .iter()
            .filter_map(|d| d.low)
            .filter(|v| v.is_finite())
            .reduce(f64::min),
        rain_total: if rain.is_empty() {
            None
        } else {
            Some(rain.iter().sum())
        },
        rain_days: rain.len(),
        total_days: days.len(),
    }
}
impl Summary {
    pub fn render(&self, units: Units) -> String {
        format!(
            "\nPeriod: low {} {}, high {} {} · precipitation {} {} (available for {}/{} days)\n",
            number(self.low),
            units.temperature_label(),
            number(self.high),
            units.temperature_label(),
            number(self.rain_total),
            units.precipitation_label(),
            self.rain_days,
            self.total_days
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summary_reports_extremes_and_partial_rain_coverage() {
        let days = vec![
            Day {
                date: "2024-03-01".into(),
                high: Some(20.0),
                low: Some(5.0),
                rain: Some(2.0),
                code: None,
            },
            Day {
                date: "2024-03-02".into(),
                high: Some(22.0),
                low: None,
                rain: None,
                code: None,
            },
        ];
        let result = summarize(&days);
        assert_eq!(result.high, Some(22.0));
        assert_eq!(result.low, Some(5.0));
        assert_eq!(result.rain_total, Some(2.0));
        assert_eq!(result.rain_days, 1);
        assert!(result.render(Units::Metric).contains("1/2 days"));
    }
    #[test]
    fn empty_or_all_missing_rain_is_not_zero_rain() {
        assert_eq!(summarize(&[]).rain_total, None);
        let day = Day {
            date: "2024-03-01".into(),
            high: None,
            low: None,
            rain: None,
            code: None,
        };
        assert_eq!(summarize(&[day]).rain_total, None);
    }
}
