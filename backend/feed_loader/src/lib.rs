use anyhow::{Context, Ok, Result};
use chrono::{DateTime, Utc};
use tracing::debug;

pub async fn new_feed(url: &str) -> Result<Feed> {
    debug!("new feed: {}", url);

    let response = reqwest::get(url).await.context("error executing request")?;
    let bytes = response.bytes().await.context("error reading response")?;
    debug!("got {n} bytes", n = bytes.len());

    let parsed = parse_feed(&bytes)?;

    debug!("parsed feed: {:?}", parsed.title);

    Ok(parsed)
}

fn parse_feed(bytes: &[u8]) -> Result<Feed> {
    debug!("parsing feed as RSS");
    let feed = parse_rss(bytes).or_else(|_| {
        debug!("failed to parse as RSS, parsing as Atom");
        parse_atom(bytes).map_err(|_| anyhow::anyhow!("failed to parse as Atom"))
    })?;
    debug!("parsed feed");
    Ok(feed)
}

fn parse_rss(feed: &[u8]) -> Result<Feed> {
    let parsed = rss::Channel::read_from(feed)?;

    Ok(Feed {
        title: parsed.title.to_string(),
        description: parsed.description.to_string(),
        url: parsed.link.to_string(),
        entries: parsed
            .items
            .iter()
            .map(|item| Entry {
                title: item
                    .title
                    .as_ref()
                    .map(|title| title.to_string())
                    .unwrap_or_default(),
                url: item
                    .link
                    .as_ref()
                    .map(|link| link.to_string())
                    .unwrap_or_default(),
                published_at: item
                    .pub_date
                    .as_ref()
                    .map(|date| DateTime::parse_from_rfc2822(date).unwrap().into())
                    .unwrap_or_default(),
                comments_url: item.comments.as_ref().map(|comments| comments.to_string()),
            })
            .collect(),
    })
}

fn parse_atom(feed: &[u8]) -> Result<Feed> {
    let parsed = atom_syndication::Feed::read_from(feed)?;

    Ok(Feed {
        title: parsed.title.to_string(),
        description: parsed
            .subtitle
            .map(|subtitle| subtitle.value)
            .unwrap_or_default(),
        url: parsed
            .links
            .iter()
            .find(|link| link.rel == "self")
            .map(|link| link.href.clone())
            .ok_or_else(|| anyhow::anyhow!("no self link found"))?,
        entries: parsed
            .entries
            .iter()
            .map(|entry| Entry {
                title: entry.title.to_string(),
                url: entry
                    .links
                    .first()
                    .map(|link| link.href.clone())
                    .ok_or_else(|| anyhow::anyhow!("no link found"))
                    .unwrap_or_default(),
                published_at: entry
                    .published
                    .map(|published| published.to_utc())
                    .unwrap_or_default(),
                comments_url: None,
            })
            .collect(),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct Feed {
    pub title: String,
    pub description: String,
    pub url: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, serde::Serialize)]
pub struct Entry {
    pub title: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub comments_url: Option<String>,
}
