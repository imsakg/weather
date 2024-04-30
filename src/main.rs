use std::io::Write;
use structopt::StructOpt;
use weather::{
    cli::Options,
    display,
    report::{Report, ReportKind},
    Result,
};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Options::from_args()).await {
        eprintln!("weather: {}", weather::text::terminal(&error));
        std::process::exit(1);
    }
}
async fn run(mut args: Options) -> Result<()> {
    if let Some(path) = &args.config {
        weather::config::SavedLocation::read(path)?.apply(&mut args)?;
    }
    let report = weather::app::load(&args).await?;
    // Complete validation and rendering before emitting any report stdout.
    let output = render(&args, &report)?;
    std::io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(|e| format!("cannot write output: {e}"))
}
fn render(args: &Options, report: &Report) -> Result<String> {
    report.validate()?;
    if args.json {
        return Ok(format!("{}\n", report.json()?));
    }
    if args.csv {
        return weather::export::daily_csv(report);
    }
    let location = &report.location;
    let data = &report.data;
    let units = report.units;
    let days = data
        .daily
        .as_ref()
        .ok_or("report has no daily data")?
        .rows()?;
    let mut output = if report.kind == ReportKind::History {
        display::history(location, data, units)?
    } else {
        format!(
            "{}{}",
            display::current(location, data, units),
            display::daily(&days, units)
        )
    };
    output.push_str(&weather::summary::summarize(&days).render(units));
    if args.hourly > 0 {
        output.push_str(&display::hourly(data, units, args.hourly)?);
    }
    if args.chart {
        output.push_str(&weather::chart::temperature(&days, units));
        output.push_str(&weather::chart::rainfall(&days, units, 32));
    }
    Ok(output)
}
