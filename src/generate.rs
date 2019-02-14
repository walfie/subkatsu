use crate::error::*;
use crate::opts;
use crate::train::tokenize;
use lazy_static::lazy_static;
use markov::Chain;
use slog::Logger;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Write;
use subparse::{GenericSubtitleFile, SubtitleFile};

pub fn generate_from_opts(
    log: &Logger,
    args: opts::Generate,
    output: &mut impl Write,
) -> Result<()> {
    let subtitle_file = match args.existing_subs {
        None => None,
        Some(path) => {
            slog::info!(log, "Loading subtitles from file"; "path" => &path);
            Some(crate::train::get_subtitles_from_file(&path, true)?)
        }
    };

    slog::info!(log, "Loading model from file"; "path" => &args.model);
    let chain = load_model(&args.model)?;

    let start = args.start.as_ref().map(|s| s.as_ref());

    if let Some(mut file) = subtitle_file {
        generate_subtitle_file(&log, &mut file, chain, start, args.min_length)?;

        let data = file
            .to_data()
            .context(|| "failed to serialize subtitle data")?;

        output
            .write(&data)
            .context(|| "failed to write to output")?;
    } else {
        let lines = generate_lines(&log, chain, start, args.min_length);
        for line in lines.take(args.count) {
            output
                .write(line?.as_ref())
                .context(|| "failed to write to output")?;
        }
    }

    Ok(())
}

pub fn load_model(path: &str) -> Result<Chain<String>> {
    Chain::load(path).context(|| "failed to load model file")
}

pub fn generate_subtitle_file(
    log: &Logger,
    subtitle_file: &mut GenericSubtitleFile,
    chain: Chain<String>,
    start: Option<&str>,
    min_length: Option<usize>,
) -> Result<()> {
    let mut subtitle_entries = subtitle_file
        .get_subtitle_entries()
        .context(|| "failed to parse subtitle entries")?;

    // Lines that have the same tokenized output should get the same generated string
    let mut generated: HashMap<Vec<String>, String> = HashMap::new();

    for mut subtitle in subtitle_entries.iter_mut() {
        if let Some(line) = subtitle.line.take() {
            // `\pos` highly suggests the line was used for typesetting
            // backgrounds/signs rather than dialogue
            if line.trim().is_empty() || line.contains(r"\pos") {
                subtitle.line = Some("".to_owned());
            } else {
                match generated.entry(tokenize(&line)) {
                    Entry::Occupied(e) => {
                        subtitle.line = Some(e.get().to_owned());
                    }
                    Entry::Vacant(e) => {
                        let new_line = e.insert(generate_line(log, &chain, start, min_length)?);
                        subtitle.line = Some(new_line.to_owned());
                    }
                }
            }
        }
    }

    subtitle_file
        .update_subtitle_entries(&subtitle_entries)
        .context(|| "failed to update subtitle lines")
}

pub fn generate_lines<'a>(
    log: &'a Logger,
    chain: Chain<String>,
    start: Option<&'a str>,
    min_length: Option<usize>,
) -> impl Iterator<Item = Result<String>> + 'a {
    std::iter::repeat_with(move || generate_line(log, &chain, start, min_length))
}

pub fn generate_line(
    log: &Logger,
    chain: &Chain<String>,
    start_token: Option<&str>,
    min_length: Option<usize>,
) -> Result<String> {
    let mut line = generate_single(log, &chain, start_token)?;

    if let Some(length) = min_length {
        while line.chars().count() < length {
            line.push(' ');
            line.push_str(&generate_single(log, &chain, None)?);
        }
    }

    Ok(line)
}

fn generate_single(
    log: &Logger,
    chain: &Chain<String>,
    start_token: Option<&str>,
) -> Result<String> {
    let generated = match start_token {
        Some(start_token) => chain.generate_from_token(start_token.to_owned()),
        None => chain.generate(),
    };

    if generated.is_empty() && start_token.is_some() {
        slog::error!(
            log,
            "Token was not found in the model \
            (note that the `--start` param only works for models with order = 1)";
            "token" => &start_token.unwrap()
        );
        return Err(Error::context("failed to generate chain from start token"));
    }

    let (pre, post) = balance_symbols(&generated);

    let mut output = {
        let size = generated.iter().fold(0, |acc, v| acc + v.len() + 1);
        String::with_capacity(size)
    };

    let tokens_iter = pre
        .into_iter()
        .chain(generated.into_iter())
        .chain(post.into_iter().rev().map(String::from));

    write_tokens(tokens_iter, &mut output).context(|| "failed to write tokens to output")?;

    // TODO: This seems kinda hacky
    if output.ends_with(" \"\"") {
        output.truncate(output.len() - 3);
    }

    Ok(output)
}

fn balance_symbols<T: AsRef<str>>(
    tokens: impl IntoIterator<Item = T>,
) -> (Vec<String>, Vec<String>) {
    lazy_static! {
        static ref BRACKETS: &'static [(char, char)] = &[
            ('（', '）'),
            ('｛', '｝'),
            ('［', '］'),
            ('【', '】'),
            ('〖', '〗'),
            ('〔', '〕'),
            ('〘', '〙'),
            ('〈', '〉'),
            ('《', '》'),
            ('「', '」'),
            ('『', '』'),
            ('＜', '＞'),
            ('≪', '≫'),
            ('｢', '｣'),
            ('(', ')'),
            ('[', ']'),
            ('<', '>'),
            ('"', '"')
        ];
        static ref OPEN: HashMap<char, char> = BRACKETS.iter().cloned().collect();
        static ref CLOSE: HashMap<char, char> =
            BRACKETS.iter().cloned().map(|(x, y)| (y, x)).collect();
    };

    let mut pre_stack = Vec::<char>::new();
    let mut post_stack = Vec::<char>::new();

    for token in tokens.into_iter() {
        for c in token.as_ref().chars() {
            if let Some(close) = OPEN.get(&c) {
                if pre_stack.last() == Some(&c) {
                    pre_stack.pop();
                } else {
                    post_stack.push(*close);
                }
            } else if let Some(open) = CLOSE.get(&c) {
                if post_stack.last() == Some(&c) {
                    post_stack.pop();
                } else {
                    pre_stack.push(*open);
                }
            }
        }
    }

    (
        pre_stack.iter().map(|c| c.to_string()).collect(),
        post_stack.iter().map(|c| c.to_string()).collect(),
    )
}

fn write_tokens<T: AsRef<str>>(
    tokens: impl IntoIterator<Item = T>,
    output: &mut impl std::fmt::Write,
) -> std::fmt::Result {
    let mut iter = tokens.into_iter();
    let first = match iter.next() {
        Some(first) => first,
        None => return Ok(()),
    };

    output.write_str(first.as_ref())?;

    // TODO: Generalize for other types of quotes and brackets
    let mut need_close_quote = first.as_ref() == "\"";
    let mut skip_space = need_close_quote;

    for next_token in iter {
        let next_token = next_token.as_ref();

        if next_token == "\"" {
            if need_close_quote {
                output.write_char('"')?;
                need_close_quote = false;
            } else {
                output.write_str(" \"")?;
                skip_space = true;
                need_close_quote = true;
            }
            continue;
        }

        if next_token.chars().any(|c| c.is_ascii_alphanumeric()) && !skip_space {
            output.write_char(' ')?;
        }

        skip_space = false;
        output.write_str(next_token)?;
    }

    Ok(())
}
