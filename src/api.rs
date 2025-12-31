use std::env;
use reqwest::Client;
use serde::Deserialize;

const API_URL: &str = "https://api.airvisual.com/v2/nearest_city";

#[inline]
pub(crate) async fn fetch_air_quality(
    client: &Client,
    api_key: &ApiKey,
    (lat, lon): &(f64, f64),
) -> anyhow::Result<AirResponse> {
    let url = format!(
        "{API_URL}?lat={lat}&lon={lon}&key={}",
        api_key.key
    );

    let resp = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<AirResponse>()
        .await?;

    Ok(resp)
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirResponse {
    pub(crate) data: AirData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirData {
    pub(crate) city: String,
    pub(crate) current: AirCurrent,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AirCurrent {
    pub(crate) pollution: Pollution,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Pollution {
    pub(crate) aqius: i32,
}

pub(crate) struct ApiKey {
    pub(crate) key: String,
}

impl ApiKey {
    #[inline]
    pub(crate) fn from_env() -> Self {
        let key = env::var("AIR_API_KEY")
            .expect("AIR_API_KEY env var is not set");
        Self { key }
    }
}