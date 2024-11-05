use anyhow::{anyhow, Result};
use clap::Parser;
use colored::Colorize;
use mime::Mime;
use reqwest::Client;
use std::collections::HashMap;
use std::str::FromStr;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
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

    #[arg(short, value_parser = parse_kv_pair, value_delimiter = ',')]
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

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let client = Client::new();
    let result = match opts.subcommands {
        SubCommands::Get(ref args) => get(client, args).await?,
        SubCommands::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}

async fn get(client: Client, get: &Get) -> Result<()> {
    let response = client.get(&get.url).send().await?;
    print_all(response).await?;
    Ok(())
}

async fn post(client: Client, post: &Post) -> Result<()> {
    let mut map = HashMap::new();

    for kv_pair in post.body.iter() {
        let key = &kv_pair.k;
        let value = &kv_pair.v;
        map.insert(key, value);
    }

    let response = client.post(&post.url).json(&map).send().await?;
    print_all(response).await?;

    Ok(())
}

fn get_content_type(response: &reqwest::Response) -> Option<Mime> {
    response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_all(response: reqwest::Response) -> Result<()> {
    print_status(&response);
    print_header(&response);

    let mime = get_content_type(&response);
    let body = response.text().await?;

    print_body(mime, &body);

    Ok(())
}

fn print_status(response: &reqwest::Response) {
    let version = response.version();
    let status_code = response.status();

    let formated_version = format!("{:?}", version).to_string().white();
    let formated_status_code = format!("{:?}", status_code).to_string().black();

    println!("{} {}", formated_version, formated_status_code);
}

fn print_header(response: &reqwest::Response) {
    let headers = response.headers();

    for (name, value) in headers.iter() {
        let formated_name = format!("{}: ", name).yellow();
        let formated_value = format!("{:?}", value).blue();
        println!("{}{}", formated_name, formated_value);
    }
}

fn print_body(option_mime: Option<Mime>, body: &String) {
    match option_mime {
        Some(v) if v == mime::TEXT_HTML => print_with_syntect(body, "html"),
        Some(v) if v == mime::APPLICATION_JSON => print_with_syntect(body, "json"),
        _ => {
            println!("{}", body);
        }
    }
}

fn print_with_syntect(string: &str, code_type: &str) {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set.find_syntax_by_extension(code_type).unwrap();
    let mut highlight_lines = HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]);

    for line in LinesWithEndings::from(string) {
        let ranges: Vec<(Style, &str)> = highlight_lines.highlight_line(line, &syntax_set).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}
