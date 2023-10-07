use clap::{arg, command, Parser};
use config::{Config, ConfigError, Environment, Source};
use serde_derive::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub gpt: ChatGptConfig,
    pub url: Url,
    #[serde(default = "default_sentence_count")]
    pub extract_sentences: usize,
}

fn default_sentence_count() -> usize {
    30
}

#[derive(Debug, Deserialize)]
pub struct ChatGptConfig {
    pub api_url: Url,
    pub api_key: String,
}

/// Summarizes web page text using the combination of ChatGPT and extractive summarization.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Web page to summarize
    #[arg(short, long)]
    url: Url,
    /// Amount of sentences to extract from source text (important sentences are prioritized)
    #[arg(short, long)]
    extract_sentences: Option<usize>,
}

impl Source for Args {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, ConfigError> {
        let mut map = config::Map::new();
        map.insert("url".into(), self.url.to_string().into());
        Ok(map)
    }
}

impl AppConfig {
    pub fn new(args: Args) -> Self {
        let config = Config::builder()
            .add_source(Environment::with_prefix("CONF").separator("__"))
            .add_source(args)
            .build()
            .unwrap();

        config.try_deserialize().unwrap()
    }
}
