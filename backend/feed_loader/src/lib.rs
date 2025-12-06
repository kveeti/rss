use anyhow::{Context, Ok, Result};
use tracing::debug;

pub async fn new_feed(url: &str) -> Result<rss::Channel> {
    debug!("new feed: {}", url);

    let response = reqwest::get(url).await.context("error executing request")?;
    let bytes = response.bytes().await.context("error reading response")?;
    debug!("got {n} bytes", n = bytes.len());

    let parsed = rss::Channel::read_from(&bytes[..]).context("error parsing feed")?;

    debug!("parsed feed: {:?}", parsed.title);

    Ok(parsed)
}
