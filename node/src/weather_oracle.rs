use async_trait;
use node_template_runtime::weather_oracle::INHERENT_IDENTIFIER;
use serde::Deserialize;
use sp_core::Encode;
use sp_inherents::{InherentData, InherentIdentifier};
use std::fmt::Debug;
//use serde::de::DeserializeOwned;
use sc_client_api::HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_arithmetic::{PerThing, Permill, Rounding};
use sp_core::H256;
use std::sync::Arc;
//use sp_runtime::generic::BlockId;
use sp_runtime::traits::{Block as BlockT, PhantomData};
//use std::marker::PhantomData;
use node_template_runtime::runtime_apis::WeatherOrder;

#[derive(Deserialize, Debug)]
struct Weather {
	temperature: f32,
	#[allow(dead_code)]
	windspeed: f32,
	#[allow(dead_code)]
	winddirection: f32,
	#[allow(dead_code)]
	weathercode: u32,
	#[allow(dead_code)]
	is_day: u32,
	#[allow(dead_code)]
	time: String,
}

#[derive(Deserialize, Debug)]
struct WeatherResponseBody {
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	latitude: f32,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	longitude: f32,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	generationtime_ms: f32,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	utc_offset_seconds: u32,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	timezone: String,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	timezone_abbreviation: String,
	#[allow(dead_code)]
	#[serde(skip_deserializing)]
	elevation: f32,
	current_weather: Weather,
}

pub async fn fetch_weather(
	lat: &str,
	long: &str,
) -> Result<Permill, Box<dyn std::error::Error + Send + Sync>> {
	let weather = reqwest::get(format!(
		"https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
		lat, long
	))
	.await?
	.text()
	.await?;

	let weather_response_body: WeatherResponseBody = serde_json::from_str(&weather)?;
	log::info!("weather = {:?}", weather_response_body.current_weather);

	let fixed_point_temp = Permill::from_rational_with_rounding(
		(weather_response_body.current_weather.temperature * 10.0) as u32,
		1000,
		Rounding::Down,
	)
	.map_err(|_| "Failed to create Permill value")?;

	Ok(fixed_point_temp)
}

pub async fn fetch_local_weather() -> Result<Permill, Box<dyn std::error::Error + Send + Sync>> {
	let ip = reqwest::get("http://icanhazip.com").await?.text().await?;

	log::info!("ip provider = {:?}", ip);

	let latlong = reqwest::get(format!("https://ipapi.co/{}/latlong/", ip)).await?.text().await?;

	log::info!("latlong = {:?}", latlong);
	let mut it = latlong.split(",");

	let lat = it.next().ok_or::<String>("Could not fetch latitude".to_string())?;
	let long = it.next().ok_or::<String>("Could not fetch longitude".to_string())?;

	fetch_weather(lat, long).await
}

#[derive(Debug, Clone)]
pub struct InherentProvider<B, C> {
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<B, C> InherentProvider<B, C> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: PhantomData }
	}
}

/// Implementation of sp_inherents::InherentDataProvider trait for ExternalDataInherentProvider
#[async_trait::async_trait]
impl<B: BlockT<Hash = H256>, C> sp_inherents::InherentDataProvider for InherentProvider<B, C>
where
	C: ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: WeatherOrder<B>,
{
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		let best_hash = self.client.info().best_hash;

		let latlong =
			self.client.runtime_api().weather_order(best_hash).ok().unwrap_or_default().map(
				|(lat, long)| ((lat as f32 / 10.0).to_string(), (long as f32 / 10.0).to_string()),
			);

		log::info!("from Runtime latlong: {:?}", latlong);

		if let Some((lat, long)) = latlong {
			match fetch_weather(&lat, &long).await {
				Ok(temperature) =>
					inherent_data.put_data(INHERENT_IDENTIFIER, &temperature.encode()),
				Err(err) => Err(err.into()),
			}
		} else {
			match fetch_local_weather().await {
				Ok(temperature) =>
					inherent_data.put_data(INHERENT_IDENTIFIER, &temperature.encode()),
				Err(err) => Err(err.into()),
			}
		}
	}

	async fn try_handle_error(
		&self,
		identifier: &InherentIdentifier,
		error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		// handle only data identified by INHERENT_IDENTIFIER key, ignore the rest
		if *identifier == INHERENT_IDENTIFIER {
			Some(Err(sp_inherents::Error::Application(Box::from(std::format!(
				"Error processing inherent: {:?}",
				error
			)))))
		} else {
			None
		}
	}
}
