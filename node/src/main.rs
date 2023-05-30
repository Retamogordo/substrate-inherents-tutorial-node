//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;
mod inherent_data_provider;
mod rpc;
mod weather_oracle;

fn main() -> sc_cli::Result<()> {
	command::run()
}
