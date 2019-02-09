#[macro_use]
pub mod train;
pub mod error;
pub mod generate;
pub mod opts;

use opts::Opts;
use slog::Drain;
use structopt::StructOpt;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, slog::o!());

    let result = match Opts::from_args() {
        Opts::Train(args) => train::train(&log, args),
        Opts::Generate(args) => generate::generate(&log, args),
    };

    if let Err(err) = result {
        slog::error!(log, "Encountered error"; "description" => %err);

        for cause in err.iter().skip(1) {
            slog::error!(log, "Underlying error"; "description" => %cause);
        }

        // Drop the logger so the messages get flushed before exiting
        std::mem::drop(log);
        std::process::exit(1);
    }
}
