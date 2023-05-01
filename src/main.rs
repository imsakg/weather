use exitfailure::ExitFailure;
use structopt::StructOpt;

mod forecast;
use forecast::{Forecast, Temperatures, CLI};

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = CLI::from_args();
    let response = Forecast::get(&args.city, &args.country_code).await?;
    let _my_units = forecast::Units::new(Some(Temperatures::Celsius));
    println!(
        "selected city: {}, country code: {}, temperature: {}, humidity: {}",
        args.city, args.country_code, response.main.temp, response.main.humidity
    );
    // sleep
    Ok(())
}
