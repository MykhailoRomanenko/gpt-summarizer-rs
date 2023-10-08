#![feature(map_try_insert)]
use chatgpt::{
    prelude::{ChatGPT, ModelConfigurationBuilder},
    types::ResponseChunk,
};
use config::{AppConfig, ChatGptConfig};
use extractive_summary::{calc_ranks, similarity_matrix};
use futures_util::{Stream, StreamExt};
use itertools::Itertools;
use rust_stemmers::Stemmer;
use scraper::{ElementRef, Html, Selector};
use std::{
    collections::{hash_map::RandomState, HashSet},
    io::{stdout, Write},
};
use stop_words::LANGUAGE;
use tracing::{error, info};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use crate::error::AppError;

pub mod config;
pub mod error;
mod extractive_summary;

async fn fetch_page(url: Url) -> Result<Html, reqwest::Error> {
    info!("Fetching {:?}...", url.to_string());
    let page = reqwest::get(url).await?.text().await?;
    info!("Web page loaded successfully");

    Ok(Html::parse_document(&page))
}

async fn summarize_gpt(
    config: ChatGptConfig,
    message: String,
) -> Result<impl Stream<Item = ResponseChunk>, chatgpt::err::Error> {
    let ChatGptConfig { api_url, api_key } = config;

    info!("Sending to ChatGPT...");
    let gpt_client = ChatGPT::new_with_config(
        api_key,
        ModelConfigurationBuilder::default()
            .api_url(api_url)
            .build()
            .unwrap(),
    )?;
    let mut conversation =
        gpt_client.new_conversation_directed("Please summarize the following text:");

    let stream = conversation.send_message_streaming(message).await?;

    info!("Response stream acquired");
    Ok(stream)
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let AppConfig {
        gpt,
        url,
        extract_sentences,
    } = config;

    let html = fetch_page(url).await.map_err(|e| {
        error!("Failed to retrieve page HTML: {:?}", e);
        AppError::FetchPageError(e)
    })?;

    let text = html
        .select(&Selector::parse("p").unwrap())
        .map(scrap_text)
        .join(" ");

    let sentences = text.unicode_sentences().collect_vec();
    dbg!(sentences.len());

    let stop_words = HashSet::<String, RandomState>::from_iter(stop_words::get(LANGUAGE::English));
    let stemmer = Stemmer::create(rust_stemmers::Algorithm::English);
    let tokenized_sentences = sentences
        .iter()
        .map(|s| {
            s.unicode_words()
                .map(|v| v.to_lowercase())
                .filter(|v| !stop_words.contains(v))
                .map(|s| stemmer.stem(&s).into_owned())
                .collect_vec()
        })
        .collect_vec();

    let matrix = similarity_matrix(&tokenized_sentences);
    let mut ranks = calc_ranks(&matrix).into_iter().enumerate().collect_vec();
    ranks.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    let summary = ranks
        .into_iter()
        .take(extract_sentences)
        .map(|(i, _)| sentences[i])
        .join(" ");

    let mut stream = summarize_gpt(gpt, summary).await.map_err(|e| {
        error!("Failed to generate ChatGPT summary: {:?}", e);
        AppError::ChatGptError(e)
    })?;

    let mut output = vec![];
    while let Some(chunk) = stream.next().await {
        if let ResponseChunk::Content { delta, .. } = &chunk {
            print!("{delta}");
            stdout().lock().flush().unwrap();
        }
        output.push(chunk);
    }

    Ok(())
}

fn scrap_text(e: ElementRef<'_>) -> String {
    if e.has_children() {
        let children_text = e
            .children()
            .into_iter()
            .flat_map(ElementRef::wrap)
            .map(|e| scrap_text(e))
            .join(" ");
        if !children_text.is_empty() {
            return children_text;
        }
    }
    e.inner_html()
}
