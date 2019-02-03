use crate::error::*;
use crate::opts;
use lazy_static::lazy_static;
use markov::Chain;
use slog::Logger;
use std::collections::HashMap;

pub fn generate(log: &Logger, args: opts::Generate) -> Result<()> {
    slog::info!(log, "Loading model from file"; "path" => &args.model);

    let chain: Chain<String> = Chain::load(&args.model).context("failed to load model file")?;

    let start = args.start.as_ref().map(|s| s.as_ref());

    for _ in 0..args.count {
        let mut output = generate_single(log, &chain, start)?;

        if let Some(min_length) = args.min_length {
            while output.chars().count() < min_length {
                output.push(' ');
                output.push_str(&generate_single(log, &chain, None)?);
            }
        }

        println!("{}", output);
    }

    Ok(())
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

    write_tokens(tokens_iter, &mut output).context("failed to write tokens to output")?;

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