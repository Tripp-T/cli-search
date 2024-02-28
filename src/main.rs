#![allow(unused_imports)]
use ::{
    anyhow::{anyhow, bail, ensure, Context, Result},
    chrono::{DateTime, Utc},
    clap::Parser,
    enum_iterator::Sequence,
    inquire::Select,
    std::{fmt::Display, str::FromStr},
    tracing::{debug, error, info, trace, warn},
};

#[derive(Parser, Debug)]
pub struct Opts {
    pub provider: Option<SearchProvider>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Sequence, PartialEq, clap::ValueEnum)]
pub enum SearchProvider {
    #[clap(alias = "ddg")]
    DuckDuckGo,
    #[clap(alias = "yt")]
    YouTube,
    #[clap(alias = "wiki")]
    Wikipedia,
    Cargo,
    #[clap(alias = "nix")]
    NixWiki,
    Nixpkgs,
    NixOptions,
}
impl SearchProvider {
    pub fn get_url(&self, query: &str) -> Result<String> {
        let query = urlencoding::encode(query);
        match self {
            Self::DuckDuckGo => Ok(format!("https://duckduckgo.com/?q={}", query)),
            Self::Wikipedia => Ok(format!(
                "https://en.wikipedia.org/w/index.php?fulltext=1&search={}",
                query
            )),
            Self::YouTube => Ok(format!(
                "https://www.youtube.com/results?search_query={}",
                query
            )),
            Self::Cargo => Ok(format!("https://crates.io/search?q={}", query)),
            Self::Nixpkgs => Ok(format!("https://search.nixos.org/packages?query={}", query)),
            Self::NixWiki => Ok(format!("https://nixos.wiki/index.php?search={}", query)),
            Self::NixOptions => Ok(format!("https://search.nixos.org/options?query={}", query)),
        }
    }
}
impl Display for SearchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuckDuckGo => write!(f, "DuckDuckGo"),
            Self::Wikipedia => write!(f, "Wikipedia"),
            Self::YouTube => write!(f, "YouTube"),
            Self::Cargo => write!(f, "Cargo"),
            Self::Nixpkgs => write!(f, "Nixpkgs"),
            Self::NixWiki => write!(f, "Nix Wiki"),
            Self::NixOptions => write!(f, "Nix Options"),
        }
    }
}
fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let opts = Opts::parse();

    let provider = if let Some(provider) = opts.provider {
        provider
    } else {
        Select::new(
            "What platform are we searching?",
            enum_iterator::all::<SearchProvider>().collect::<Vec<_>>(),
        )
        .prompt()?
    };

    let query = if let Some(query) = opts.query {
        query
    } else {
        inquire::Text::new("What is the search query?").prompt()?
    };

    let url = provider.get_url(&query)?;
    println!("Opening {}", url);
    open::that(&url).context("Failed to open URL")
}
