use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::error::Error;
use std::fs::File;

pub fn setup_environment() -> Result<(), Box<dyn Error>> {
    env::set_var("CARGO_INCREMENTAL", "0");
    env::set_var("RUSTFLAGS", "-Awarnings");
    env::set_var("CARGO_TERM_COLOR", "always");

    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log")?,
    )?;

    Ok(())
}
