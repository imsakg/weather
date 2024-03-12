use structopt::StructOpt;
use weather::{cli::Options, display, Result};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Options::from_args()).await {
        eprintln!("weather: {error}");
        std::process::exit(1);
    }
}
async fn run(mut args: Options) -> Result<()> {
    if let Some(path) = &args.config { weather::config::SavedLocation::read(path)?.apply(&mut args)?; }
    let report = weather::app::load(&args).await?;
    let location = &report.location;
    let data = &report.data;
    if args.json { println!("{}", report.json()?); return Ok(()); }
    if args.start.is_some() { print!("{}", display::history(&location, &data, args.units)?); }
    else {
        print!("{}", display::current(&location, &data, args.units));
        if let Some(daily) = &data.daily { print!("{}", display::daily(&daily.rows()?, args.units)); }
    }
    if let Some(daily) = &data.daily { print!("{}", weather::summary::summarize(&daily.rows()?).render(args.units)); }
    if args.hourly > 0 { print!("{}", display::hourly(&data, args.units, args.hourly)?); }
    if args.chart {
        if let Some(daily) = &data.daily {
            let days = daily.rows()?;
            print!("{}{}", weather::chart::temperature(&days, args.units), weather::chart::rainfall(&days, args.units, 32));
        }
    }
    Ok(())
}
