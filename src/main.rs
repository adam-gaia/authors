use anyhow::Result;
use authors::Authors;
use clap::Parser;
use log::debug;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug, Parser)]
struct Cli {
    /// Only print names
    #[clap(short, long)]
    #[arg(group = "display")]
    names: bool,

    /// Only print emails
    #[clap(short, long)]
    #[arg(group = "display")]
    emails: bool,

    /// Output json
    #[clap(short, long)]
    json: bool,

    input: Option<Vec<String>>,
}

pub fn print<I>(items: I)
where
    I: IntoIterator,
    I::Item: Display,
{
    let mut items = items.into_iter().peekable();
    while let Some(item) = items.next() {
        print!("{item}");
        if items.peek().is_some() {
            print!(", ")
        }
    }
    println!();
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let input = match args.input {
        Some(input) => input,
        None => {
            let reader = BufReader::new(io::stdin());
            let lines: Vec<String> = reader.lines().collect::<Result<_, io::Error>>()?;
            lines
        }
    };

    let input = input.join(" ");
    let authors = Authors::from_str(&input)?;
    debug!("{authors:?}");

    if args.names {
        let names = authors.into_iter().map(|author| author.name());
        print(names);
    } else if args.emails {
        let empty = String::new();
        let emails = authors
            .into_iter()
            .map(|author| author.email().unwrap_or(&empty));
        print(emails)
    } else {
        print(&authors)
    }

    Ok(())
}
