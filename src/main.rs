pub mod error;
pub mod generate;
pub mod opts;
pub mod train;

use error::*;
use opts::Opts;
use structopt::StructOpt;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Err(err) = run() {
        log::error!("Error: {}", err);

        for cause in err.iter().skip(1) {
            log::error!("caused by: {}", cause);
        }

        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Opts::from_args() {
        Opts::Train(args) => train::train(args)?,
        Opts::Generate(args) => generate::generate(args).context("failed to generate chains")?,
    }

    Ok(())
}
