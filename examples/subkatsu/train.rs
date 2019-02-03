use crate::opts;
use lazy_static::lazy_static;
use log::*;
use regex::Regex;

fn tokenize(text: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"([^\s\w]+)?([a-zA-Z'-]+)([^\s\w]+?)?(")?"#).unwrap();
    }

    // TODO: Detect language, case-sensitivity, etc
    let mut tokens = Vec::new();

    for capture in RE.captures_iter(text) {
        for matched in capture.iter().skip(1).flatten() {
            tokens.push(matched.as_str().to_owned());
        }
    }

    tokens
}

pub fn train(args: opts::Train) -> Result<(), Box<std::error::Error>> {
    let mut chain = markov::Chain::of_order(args.order);

    // {\c&H........&} changes color. If the alpha starts with F, the text
    // is transparent, so we should exclude any text afterward, until the
    // next color change, or end of line.
    // TODO: Should probably stop trying to clean the subs in code, and
    // just assume the subtitle files themselves are dialogue-only.
    let escapes = Regex::new(
        r"(?x)                          # Whitespace-insignificant mode, for comments
        ( \{.*\\c&H(F|f).(.{6})?&.*}.*(\{.*\\(c|r).+?}|$)
        | \{.+?}                        # Anything between {}'s is a comment
        )
        ",
    )?;

    let spaces = Regex::new(r"\\N|\\n|\\h")?;

    for path in args.input {
        // TODO: Emit warning on read/parse failure, rather than exiting
        let format = subparse::get_subtitle_format_by_ending_err(&path)?;

        // TODO: Remove `Comment: ` lines
        let file = std::fs::read_to_string(&path)?;

        let subs = subparse::parse_str(format, &file, 24.0)?.get_subtitle_entries()?;

        info!("Processing `{}`", path);

        for entry in subs {
            if let Some(line) = entry.line {
                let line = escapes.replace_all(&line, "");
                let line = spaces.replace_all(&line, " ");

                let tokens = tokenize(&line);
                chain.feed(tokens);
            }
        }
    }

    chain.save(&args.output)?;
    info!("Model saved to `{}`", args.output);

    Ok(())
}
