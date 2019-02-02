use std::env;

fn main() {
    let paths = env::args().skip(1);

    if paths.len() == 0 {
        eprintln!("Usage: {} file ...", env::args().next().unwrap());
        std::process::exit(1);
    }

    let mut chain = markov::Chain::of_order(1);

    for path in paths {
        let subs = srtparse::read_from_file(path).expect("failed to parse file");

        for sub in subs {
            let tokens = tinysegmenter::tokenize(&sub.text);
            if !tokens.is_empty() {
                chain.feed(tokens);
            }
        }
    }

    for _ in 0..300 {
        println!("{}", chain.generate().concat());
        println!("\n=====\n");
    }
}
