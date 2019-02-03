use crate::error::*;
use crate::opts;
use lazy_static::lazy_static;
use log::*;
use markov::Chain;
use std::collections::HashMap;

pub fn generate(args: opts::Generate) -> Result<()> {
    info!("Loading model from file `{}`", args.model);
    let chain: Chain<String> = Chain::load(&args.model).context("failed to load model file")?;

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

        println!("{}", output);
    }

    Ok(())
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
