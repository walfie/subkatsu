pub mod generate;
pub mod opts;
pub mod train;

use opts::Opts;
use structopt::StructOpt;

fn main() -> Result<(), Box<std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match Opts::from_args() {
        Opts::Train(args) => train::train(args)?,
        Opts::Generate(args) => generate::generate(args)?,
    }

    Ok(())
}
