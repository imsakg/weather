use structopt::StructOpt;
use weather::{api, cli::Options, display, geocoding, http::Http, Result};

#[tokio::main]
async fn main() {
    if let Err(error) = run(Options::from_args()).await {
        eprintln!("weather: {error}");
        std::process::exit(1);
    }
}
async fn run(args: Options) -> Result<()> {
    args.validate()?;
    let http = Http::new()?;
    let location = if args.locate {
        eprintln!("IP location is approximate; contacting ipapi.co at your request.");
        weather::detection::detect(&http).await?
    } else if let Some(coordinates) = args.coordinates()? {
        weather::location::Location::at(coordinates)
    } else {
        geocoding::resolve(&http, args.city.as_deref().unwrap_or_default(), args.country_code.as_deref()).await?
    };
    let data = if let Some(range) = args.history_range()? {
        api::archive(&http, location.coordinates, args.units, range).await?
    } else { api::forecast(&http, location.coordinates, args.units, args.days).await? };
    if args.start.is_some() { print!("{}", display::history(&location, &data, args.units)?); }
    else {
        print!("{}", display::current(&location, &data, args.units));
        if let Some(daily) = &data.daily { print!("{}", display::daily(&daily.rows()?, args.units)); }
    }
    Ok(())
}
