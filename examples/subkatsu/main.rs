pub mod opts;
pub mod train;

use log::*;
use markov::Chain;
use opts::Opts;
use structopt::StructOpt;

fn main() -> Result<(), Box<std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match Opts::from_args() {
        Opts::Train(args) => train::train(args)?,
        Opts::Generate(args) => generate(args)?,
    }

    Ok(())
}

fn generate(args: opts::Generate) -> Result<(), Box<std::error::Error>> {
    let chain: Chain<String> = Chain::load(&args.model)?;

    info!("Loaded model from `{}`", args.model);

    for _ in 0..args.count {
        let generated = match args.start.clone() {
            Some(start_token) => chain.generate_from_token(start_token),
            None => chain.generate(),
        };

        if generated.is_empty() && args.start.is_some() {
            error!("Token `{}` was not found in the model", args.start.unwrap());

            // TODO: Maybe return Error instead of exiting here
            std::process::exit(1);
        }

        println!("{}", generated.join(" "));
    }

    Ok(())
}
