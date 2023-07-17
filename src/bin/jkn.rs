use clap::Parser;
use cli::{exec::execute, Opts};
use env_logger;
use error::Result;
use jkn::config::{self, Config};
use jkn::db;
use jkn::*;

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let cfg = config::load().expect("could not load configuration");
    let db = db::from_config(&cfg).expect("unable to open database");
    cfg.save().expect("failed to save config");
    Ok(execute(&opts, &cfg, &db)?)
}
