use crate::opts;
use log::*;

pub fn train(args: opts::Train) -> Result<(), Box<std::error::Error>> {
    let mut chain = markov::Chain::of_order(args.order);

    // {\c&H........&} changes color. If the alpha starts with F, the text
    // is transparent, so we should exclude any text afterward, until the
    // next color change, or end of line.
    let escapes = regex::Regex::new(
        r"(?x)                          # Whitespace-insignificant mode, for comments
        ( \{\\c&HF.{7}&}.*(\{.+?}|$)
        | \{.+?}                        # Anything between {}'s is a comment
        | \\h                           # Non-breaking space
        )
        ",
    )?;

    let newlines = regex::Regex::new(r"\\N|\\n")?;

    for path in args.input {
        let format = subparse::get_subtitle_format_by_ending_err(&path)?;

        let file = std::fs::read_to_string(&path)?;

        let subs = subparse::parse_str(format, &file, 24.0)?.get_subtitle_entries()?;

        info!("Processing `{}`", path);

        for entry in subs {
            if let Some(line) = entry.line {
                let line = escapes.replace_all(&line, "");
                let line = newlines.replace_all(&line, " ");

                let tokens = line
                    .split(' ')
                    .filter_map(|string| match string.trim() {
                        "" => None,
                        s => Some(s.to_owned()),
                    })
                    .collect();
                chain.feed(tokens);
            }
        }
    }

    chain.save(args.output)?;
    Ok(())
}
