pub mod error;
pub mod generate;
pub mod opts;
pub mod train;

use error::*;
use opts::Opts;
use slog::{Drain, Logger};
use structopt::StructOpt;

fn main() {
    let exit_code = run();

    std::process::exit(exit_code)
}

// This separate method is needed for slog_async to flush properly
fn run() -> i32 {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, slog::o!());

    if let Err(err) = try_run(&log) {
        slog::error!(log, "Encountered error"; "description" => %err);

        for cause in err.iter().skip(1) {
            slog::error!(log, "Underlying error"; "description" => %cause);
        }

        return 1;
    }

    return 0;
}

fn try_run(log: &Logger) -> Result<()> {
    match Opts::from_args() {
        Opts::Train(args) => train::train(log, args)?,
        Opts::Generate(args) => generate::generate(log, args)?,
    }

    Ok(())
}
