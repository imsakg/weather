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
    let location = geocoding::resolve(&http, args.city.as_deref().unwrap_or_default(), args.country_code.as_deref()).await?;
    let data = api::forecast(&http, location.coordinates, args.units, args.days).await?;
    print!("{}", display::current(&location, &data, args.units));
    if let Some(daily) = &data.daily { print!("{}", display::daily(&daily.rows()?, args.units)); }
    Ok(())
}
