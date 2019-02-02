use std::env;

fn main() -> Result<(), Box<std::error::Error>> {
    let paths = env::args().skip(1);

    if paths.len() == 0 {
        eprintln!("Usage: {} file ...", env::args().next().unwrap());
        std::process::exit(1);
    }

    let mut chain = markov::Chain::of_order(1);

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
    )
    .unwrap();

    let newlines = regex::Regex::new(r"\\N|\\n").unwrap();

    for path in paths {
        let format = subparse::get_subtitle_format_by_ending(&path)
            .unwrap_or_else(|| panic!("could not determine file format for `{}`", path));

        let file = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("failed to read file `{}`", path));

        let subs = subparse::parse_str(format, &file, 24.0)
            .unwrap_or_else(|_| panic!("failed to parse file `{}`", path))
            .get_subtitle_entries()
            .unwrap_or_else(|_| panic!("failed to get subtitle entries for file `{}`", path));

        eprintln!("Processing `{}`", path);

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

    for _ in 0..500 {
        println!("{}", chain.generate().join(" "));
        println!("\n=====\n");
    }

    Ok(())
}
