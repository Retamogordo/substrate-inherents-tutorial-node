sp_api::decl_runtime_apis! {
	pub trait WeatherOrder {
		fn weather_order() -> Option<(i16, i16)>;
	}
}
