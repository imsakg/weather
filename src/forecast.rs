use exitfailure::ExitFailure;
use reqwest::Url;
use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::API_KEY;

#[derive(Debug, Deserialize, Serialize)]
pub enum Temperatures {
    Kelvin,
    Celsius,
    Fahrenheit,
}
pub struct Units {
    pub temperature: Temperatures,
}

impl Units {
    pub fn new(temperature: Option<Temperatures>) -> Self {
        Self {
            temperature: temperature.unwrap_or(Temperatures::Kelvin),
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct CLI {
    pub city: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Forecast {
    pub coord: Coord,
    pub weather: Weather,
    pub base: String,
    pub main: Temps,
    pub visibility: i32,
    pub wind: Wind,
    pub clouds: Clouds,
    pub dt: i32,
    pub sys: Sys,
    pub timezone: i32,
    pub id: i32,
    pub name: String,
    pub cod: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Coord {
    pub lon: f32,
    pub lat: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Weather {
    pub details: Details,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Details {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Temps {
    pub temp: f32,
    pub feels_like: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub pressure: i32,
    pub humidity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Wind {
    pub speed: f32,
    pub deg: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Clouds {
    pub all: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sys {
    pub r#type: i32,
    pub id: i32,
    pub country: String,
    pub sunrise: i32,
    pub sunset: i32,
}

impl Forecast {
    pub async fn get(city: &String, country_code: &String) -> Result<Self, ExitFailure> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={},{}&appid={}",
            city, country_code, API_KEY
        );
        dbg!(&url);
        let url = Url::parse(&*url)?;
        let resp = reqwest::get(url).await?.json::<Forecast>().await?;
        Ok(resp)
    }
}
