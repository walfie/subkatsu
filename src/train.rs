use crate::error::*;
use crate::opts;
use lazy_static::lazy_static;
use regex::Regex;
use slog::Logger;
use std::path::PathBuf;

lazy_static! {
    static ref IS_CJK: Regex = Regex::new(r"[\p{Hiragana}\p{Katakana}\p{Han}]").unwrap();
    static ref ENGLISH: Regex = Regex::new(r#"([^\s\w]+)?([a-zA-Z'-]+)([^\s\w]+?)?(")?"#).unwrap();
}

pub fn is_cjk(text: &str) -> bool {
    IS_CJK.is_match(text)
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    if is_cjk(text) {
        for token in tinysegmenter::tokenize(text) {
            match token.trim() {
                "" => tokens.push(token),
                s => tokens.push(s.to_owned()),
            }
        }
    } else {
        // TODO: Case sensitivity?
        for capture in ENGLISH.captures_iter(text) {
            for matched in capture.iter().skip(1).flatten() {
                tokens.push(matched.as_str().to_owned());
            }
        }
    }

    tokens
}

// This has to be a macro for now because `subparse::parse_str` returns a private
// type in its public interface: https://github.com/kaegi/subparse/issues/3
macro_rules! get_subtitles {
    ( $path:expr ) => {{
        let format = subparse::get_subtitle_format_by_ending_err($path)
            .context("failed to determine subtitle format")?;

        // TODO: Remove `Comment: ` lines
        let content = &std::fs::read_to_string($path).context("failed to read file")?;

        subparse::parse_str(format, &content, 24.0).context("failed to parse subtitle file")
    }};
}

fn iterate_files(
    log: &Logger,
    path: String,
    recursive: bool,
) -> impl Iterator<Item = PathBuf> + '_ {
    let paths = walkdir::WalkDir::new(&path);
    let paths = if recursive { paths } else { paths.max_depth(0) };

    paths.into_iter().filter_map(move |entry| {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                let path = err
                    .path()
                    .map_or("<UNKNOWN>".into(), |p| p.to_string_lossy());
                let reason = err
                    .io_error()
                    .map_or("unknown".to_string(), |e| e.to_string());
                slog::warn!(
                    log, "Failed to handle file";
                    "path" => %path,
                    "reason" => reason
                );
                return None;
            }
        };

        if entry.file_type().is_dir() {
            if !recursive {
                slog::warn!(
                    log, "Ignoring directory";
                    "reason" => "--recursive is not specified",
                    "path" => %entry.path().to_string_lossy()
                );
            }
            return None;
        } else {
            return Some(entry.into_path());
        }
    })
}

pub fn train(log: &Logger, args: opts::Train) -> Result<()> {
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
    )
    .unwrap();

    let spaces = Regex::new(r"\\N|\\n|\\h|\n").unwrap();
    let recursive = args.recursive;

    let paths = args
        .input
        .into_iter()
        .flat_map(|path| iterate_files(log, path, recursive));

    let mut processed_files = 0;
    let mut skipped_files = 0;
    for path_buf in paths {
        let path = match path_buf.to_str() {
            Some(p) => p,
            None => {
                slog::warn!(log, "failed to parse path"; "path" => %path_buf.to_string_lossy());
                continue;
            }
        };

        // TODO: Don't error, just continue
        let subs: Result<Vec<subparse::SubtitleEntry>> = (|| {
            Ok(get_subtitles!(path)?
                .get_subtitle_entries()
                .context("failed to get subtitle entries")?)
        })();

        let subs = match subs {
            Ok(s) => s,
            Err(s) => {
                slog::warn!(
                    log, "Skipping file";
                    "reason" => %s, "path" => path
                );
                skipped_files = skipped_files + 1;
                continue;
            }
        };

        for entry in subs {
            if let Some(line) = entry.line {
                let line = escapes.replace_all(&line, "");
                let line = spaces.replace_all(&line, " ");

                let tokens = tokenize(&line);
                chain.feed(tokens);
            }
        }

        slog::info!(log, "Processed file"; "path" => path);

        processed_files = processed_files + 1;
    }

    if processed_files == 0 {
        return Err(Error::context("No files processed"));
    }

    slog::info!(
        log, "Processed input files";
        "skipped" => skipped_files, "count" => processed_files
    );
    slog::info!(log, "Saving model to file"; "path" => &args.output);
    chain
        .save(&args.output)
        .context("failed to save model file")?;

    Ok(())
}
