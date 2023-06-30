// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use text_wrapper::Text;

extern crate async_parse_wiki_text;

mod test;
mod test_cases;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    match args.nth(1) {
        None => return test::run_test(&Default::default()).await,
        Some(command) => match &command as _ {
            "file" => if let Some(path) = args.next() {
                if args.next().is_none() {
                    match std::fs::read_to_string(path) {
                        Err(error) => {
                            eprintln!("Failed to read file: {}", error);
                            std::process::exit(1);
                        }
                        Ok(file_contents) => {
                            println!(
                                "{:#?}",
                                async_parse_wiki_text::Configuration::default().parse(Text::new(&file_contents)).await
                            );
                            return;
                        }
                    }
                }
            },
            "text" => if let Some(wiki_text) = args.next() {
                if args.next().is_none() {
                    println!(
                        "{:#?}",
                        async_parse_wiki_text::Configuration::default()
                            .parse(Text::new(&wiki_text.replace("\\t", "\t").replace("\\n", "\n"))).await
                    );
                    return;
                }
            },
            _ => {}
        },
    }
    eprintln!("invalid use");
    std::process::exit(1);
}
