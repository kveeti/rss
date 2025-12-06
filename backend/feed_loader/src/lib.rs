use anyhow::{Context, Ok, Result};
use chrono::{DateTime, Utc};
use html5ever::{ParseOpts, parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts};
use markup5ever_rcdom::{NodeData, RcDom};
use once_cell::sync::Lazy;
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::redirect;
use texting_robots::{Robot, get_robots_url};
use tracing::debug;

#[derive(Debug, serde::Serialize)]
pub enum NewFeedResult {
    DiscoveredMultiple(Vec<String>),
    Feed(Feed),
    NotFound,
}

pub async fn new_feed(url: &str) -> Result<NewFeedResult> {
    debug!("new feed: {}", url);

    let feed = fetch_feed(url).await?;
    match feed {
        FeedFetchResult::Feed(bytes) => Ok(NewFeedResult::Feed(parse_feed(&bytes)?)),
        FeedFetchResult::Html(bytes) => {
            let feed_urls = discover_feed_urls(&bytes, url)?;
            if feed_urls.len() == 1 {
                let feed = &fetch_feed(&feed_urls[0]).await?;
                match feed {
                    FeedFetchResult::Feed(bytes) => Ok(NewFeedResult::Feed(parse_feed(&bytes)?)),
                    _ => Err(anyhow::anyhow!("expected feed, got {feed:?}")),
                }
            } else {
                Ok(NewFeedResult::DiscoveredMultiple(feed_urls))
            }
        }
        FeedFetchResult::NotFound => Err(anyhow::anyhow!("not found")),
        FeedFetchResult::NotAllowed => Err(anyhow::anyhow!("not allowed")),
        FeedFetchResult::Unknown { status, body } => {
            Err(anyhow::anyhow!("unknown: {status}: {body}"))
        }
    }
}

#[derive(Debug)]
enum FeedFetchResult {
    Feed(Vec<u8>),
    Html(Vec<u8>),
    NotFound,
    Unknown { status: StatusCode, body: String },
    NotAllowed,
}

const USER_AGENT: &str = "rss reader";

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(redirect::Policy::limited(10))
        .build()
        .expect("client should be valid")
});

async fn fetch_feed(url: &str) -> Result<FeedFetchResult> {
    debug!("fetch requested for {url}");

    let robots_url = get_robots_url(url).context("error getting robots url")?;
    debug!("checking robots at {robots_url}");

    let robots = CLIENT
        .get(robots_url)
        .send()
        .await
        .context("error fetching robots")?
        .bytes()
        .await
        .context("error reading robots")?;
    let robots = Robot::new(USER_AGENT, &robots).context("error parsing robots")?;

    let allowed = robots.allowed(url);
    if !allowed {
        debug!("not allowed to fetch {url}");
        return Ok(FeedFetchResult::NotAllowed);
    }

    debug!("fetching {url}");

    let response = CLIENT
        .get(url)
        .send()
        .await
        .context("error executing request")?;
    let status = response.status();

    match status {
        StatusCode::NOT_FOUND => return Ok(FeedFetchResult::NotFound),
        StatusCode::OK => {
            let headers = response.headers().clone();

            let bytes = response.bytes().await.context("error reading response")?;

            let content_type = headers
                .get("Content-Type")
                .context("no content type found")?
                .to_str()
                .context("invalid content type")?;
            debug!(
                "got {n} bytes with content type {content_type}",
                n = bytes.len()
            );
            if content_type.starts_with("text/html") {
                return Ok(FeedFetchResult::Html(bytes.to_vec()));
            }

            if content_type.starts_with("text/xml")
                || content_type.starts_with("application/rss+xml")
                || content_type.starts_with("application/atom+xml")
                || content_type.starts_with("application/xml")
            {
                return Ok(FeedFetchResult::Feed(bytes.to_vec()));
            }

            return Ok(FeedFetchResult::Unknown {
                body: String::from_utf8_lossy(&bytes).to_string(),
                status,
            });
        }
        _ => {
            return Ok(FeedFetchResult::Unknown {
                body: response.text().await.context("error reading response")?,
                status,
            });
        }
    }
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

fn discover_feed_urls(mut bytes: &[u8], url: &str) -> Result<Vec<String>> {
    let dom = parse_document(
        RcDom::default(),
        ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .from_utf8()
    .read_from(&mut bytes)
    .context("error parsing HTML")?;

    let feed_links = dom
        .document
        .children
        .take()
        .iter()
        .find(|child| match &child.data {
            NodeData::Element { name, .. } => name.local.as_ref() == "html",
            _ => false,
        })
        .ok_or_else(|| anyhow::anyhow!("no html element found"))?
        .children
        .take()
        .iter()
        .find(|child| match &child.data {
            NodeData::Element { name, .. } => name.local.as_ref() == "head",
            _ => false,
        })
        .ok_or_else(|| anyhow::anyhow!("no head element found"))?
        .children
        .take()
        .iter()
        .filter_map(|child| match &child.data {
            NodeData::Element { name, attrs, .. } => {
                if name.local.as_ref() == "link" {
                    attrs
                        .take()
                        .iter()
                        .find(|attr| {
                            attr.name.local.as_ref() == "href"
                                && (attr.value.as_ref().contains("rss")
                                    || attr.value.as_ref().contains("atom"))
                        })
                        .map(|attr| attr.value.to_string())
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(|href| {
            // if href is a relative URL, make it absolute
            if !href.starts_with("http") {
                if url.ends_with("/") {
                    format!("{}{}", url, href)
                } else {
                    format!("{}/{}", url, href)
                }
            } else {
                href
            }
        })
        .collect::<Vec<String>>();

    Ok(feed_links)
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
