use crate::{display::number, model::condition, report::Report, Result};

pub fn csv_cell(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .filter(|c| {
            matches!(c, '\n' | '\r')
                || (!c.is_control()
                    && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
        })
        .collect();
    let value = cleaned.as_str();
    let formula = value.trim_start().starts_with(['=', '+', '-', '@']);
    let value = if formula {
        format!("'{value}")
    } else {
        value.to_string()
    };
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}
fn numeric(value: Option<f64>) -> String {
    value
        .filter(|v| v.is_finite())
        .map(number_some)
        .unwrap_or_default()
}
fn number_some(value: f64) -> String {
    number(Some(value))
}
pub fn daily_csv(report: &Report) -> Result<String> {
    report.validate()?;
    let days = report
        .data
        .daily
        .as_ref()
        .ok_or("report has no daily data for CSV export")?
        .rows()?;
    let mut out =
        "date,location,temperature_unit,precipitation_unit,high,low,precipitation,conditions\r\n"
            .to_string();
    for day in days {
        let fields = [
            csv_cell(&day.date),
            csv_cell(&report.location.name),
            csv_cell(report.units.temperature_label()),
            csv_cell(report.units.precipitation_label()),
            numeric(day.high),
            numeric(day.low),
            numeric(day.rain),
            csv_cell(condition(day.code)),
        ];
        out.push_str(&fields.join(","));
        out.push_str("\r\n");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_has_units_and_empty_cells_for_missing_values() {
        let report: Report =
            serde_json::from_str(include_str!("../tests/fixtures/forecast.json")).unwrap();
        let text = daily_csv(&report).unwrap();
        assert!(text.starts_with("date,location,temperature_unit,precipitation_unit,high,low,precipitation,conditions\r\n"));
        assert!(text.contains("2024-03-03,Istanbul,°C,mm,14.0,,,Unknown"));
    }
    #[test]
    fn escapes_commas_quotes_newlines_and_spreadsheet_formulas() {
        assert_eq!(csv_cell("a,\"b\""), "\"a,\"\"b\"\"\"");
        assert_eq!(csv_cell("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(csv_cell("safe"), "safe");
    }
}
