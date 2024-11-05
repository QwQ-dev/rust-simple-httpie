use anyhow::{anyhow, Result};
use clap::Parser;
use std::str::FromStr;
use url::Url;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Tyr Chen <tyr@chen.com>")]
struct Opts {
    #[clap(subcommand)]
    subcommands: SubCommands,
}

#[derive(Parser, Debug)]
enum SubCommands {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    #[arg(short, value_parser = parse_url)]
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    #[arg(short, value_parser = parse_url)]
    url: String,

    #[arg(short, value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

#[derive(Debug, Clone)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut split = str.split("=");

        let k = split.next().ok_or_else(|| anyhow!("Failed to parse: no key found"))?.to_string();
        let v = split.next().ok_or_else(|| anyhow!("Failed to parse: no value found"))?.to_string();

        Ok(Self { k, v })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    s.parse()
}

fn parse_url(url: &str) -> Result<String> {
    Ok(String::from(Url::parse(url)?))
}

fn main() {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);
}