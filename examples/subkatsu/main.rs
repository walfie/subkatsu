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

fn join_tokens<T: AsRef<str>>(tokens: Vec<T>) -> String {
    let mut iter = tokens.iter();
    let first = match iter.next() {
        Some(first) => first.as_ref(),
        None => return "".to_string(),
    };

    let size = tokens.iter().fold(0, |acc, v| acc + v.as_ref().len());
    let mut result = String::with_capacity(size + tokens.len());
    result.push_str(first);

    let mut need_close_quote = first == "\"";
    let mut skip_space = need_close_quote;

    for next_token in iter {
        let next_token = next_token.as_ref();

        if next_token == "\"" {
            if need_close_quote {
                result.push('"');
                need_close_quote = false;
            } else {
                result.push_str(" \"");
                skip_space = true;
                need_close_quote = true;
            }
            continue;
        }

        if next_token.chars().any(|c| c.is_ascii_alphanumeric()) && !skip_space {
            result.push(' ');
        }

        skip_space = false;
        result.push_str(next_token);
    }

    if need_close_quote {
        if result.ends_with('"') {
            result.pop(); // Remove the quote
            result.pop(); // Remove trailing space
        } else {
            result.push('"');
        }
    }

    result
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

        println!("{}", join_tokens(generated));
    }

    Ok(())
}
