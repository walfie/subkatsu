use slog::Drain;
use structopt::StructOpt;
use subkatsu::opts::Opts;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, slog::o!());

    let result = match Opts::from_args() {
        Opts::Train(args) => subkatsu::train(&log, args),
        Opts::Generate(args) => subkatsu::generate_from_opts(&log, args, &mut std::io::stdout()),
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
