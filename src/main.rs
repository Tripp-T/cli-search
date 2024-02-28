#![allow(unused_imports)]
use ::{
    anyhow::{anyhow, bail, ensure, Context, Result},
    chrono::{DateTime, Utc},
    clap::Parser,
    enum_iterator::Sequence,
    inquire::Select,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::{env, fmt::Display, os, path::PathBuf, str::FromStr},
    tracing::{debug, error, info, trace, warn},
};

// TODO: setup default config path with clap
static _DEFAULT_CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    home::home_dir()
        .expect("Failed to get home directory")
        .join(".config")
        .join("nsearch.toml")
});

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(short, long, env = "SEARCH_CONFIG_PATH")]
    config_path: Option<PathBuf>,
    provider: Option<String>,
    query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProvider {
    name: String,
    /// The URL to use for searching,
    /// with only one instance of `{}` as a placeholder for the query
    search_url: String,
    #[serde(default)]
    aliases: Vec<String>,
}
impl SearchProvider {
    const QUERY_PLACEHOLDER: &'static str = "{}";
    pub fn new(name: &str, query_url: &str) -> Self {
        Self {
            name: name.to_string(),
            search_url: query_url.to_string(),
            aliases: vec![],
        }
    }
    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases.extend(aliases.iter().map(|s| s.to_string()));
        self
    }
    pub fn validate(&self) -> Result<()> {
        if self
            .search_url
            .match_indices(Self::QUERY_PLACEHOLDER)
            .count()
            .ne(&1)
        {
            bail!(
                "Query URL must contain exactly one instance of '{}'",
                Self::QUERY_PLACEHOLDER
            );
        }

        Ok(())
    }
    pub fn get_url(&self, query: &str) -> Result<String> {
        let query = urlencoding::encode(query);
        self.validate()?;
        Ok(self.search_url.replace(Self::QUERY_PLACEHOLDER, &query))
    }
}
impl Display for SearchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

static DEFAULT_PROVIDERS: Lazy<Vec<SearchProvider>> = Lazy::new(|| {
    vec![
        SearchProvider::new("DuckDuckGo", "https://duckduckgo.com/?q={}").with_aliases(&["ddg"]),
        SearchProvider::new(
            "Wikipedia",
            "https://en.wikipedia.org/w/index.php?search={}",
        )
        .with_aliases(&["wiki"]),
        SearchProvider::new("YouTube", "https://www.youtube.com/results?search_query={}")
            .with_aliases(&["yt"]),
        SearchProvider::new("Cargo", "https://crates.io/search?q={}"),
        SearchProvider::new("Nixpkgs", "https://search.nixos.org/packages?query={}"),
        SearchProvider::new("Nix Wiki", "https://nixos.wiki/index.php?search={}")
            .with_aliases(&["nix"]),
        SearchProvider::new("Nix Options", "https://search.nixos.org/options?query={}"),
    ]
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationFile {
    pub providers: Vec<SearchProvider>,
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let opts = Opts::parse();

    let config = opts
        .config_path
        .map(|config_path| {
            let config =
                std::fs::read_to_string(config_path).context("Failed to read config file")?;
            toml::from_str::<ConfigurationFile>(&config).context("Failed to parse config file")
        })
        .transpose()?;

    let providers: Vec<SearchProvider> = if let Some(config) = &config {
        let mut p = config.providers.to_vec();
        p.extend(DEFAULT_PROVIDERS.iter().cloned());
        p
    } else {
        DEFAULT_PROVIDERS.to_vec()
    };

    let provider = if let Some(provider) = opts.provider {
        providers
            .iter()
            .find(|p| p.name == provider || p.aliases.contains(&provider))
            .with_context(|| format!("No provider found with name or alias '{}'", provider))?
    } else {
        Select::new(
            "What platform are we searching?",
            providers.iter().collect::<Vec<_>>(),
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn validate_default_providers() -> Result<()> {
        for provider in DEFAULT_PROVIDERS.iter() {
            provider
                .validate()
                .with_context(|| format!("invalid default provider '{}'", provider.name))?;
        }
        Ok(())
    }
}
