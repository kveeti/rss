use futures::{StreamExt, stream};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::time::Duration;

use anyhow::Context;
use base64::Engine;
use chrono::{DateTime, Utc};
use html5ever::{ParseOpts, parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts};
use markup5ever_rcdom::Node;
use markup5ever_rcdom::{NodeData, RcDom};
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::redirect;
use sha2::{Digest, Sha256};
use texting_robots::{Robot, get_robots_url};
use tracing::debug;
use tracing::warn;
use url::Url;

use crate::db::{Data, NewEntry, NewFeed, NewIcon};

#[derive(Debug)]
pub enum GetFeedResult {
    DiscoveredMultiple(Vec<String>),
    Feed {
        feed: NewFeed,
        entries: Vec<NewEntry>,
        icon: Option<NewIcon>,
    },
    NotFound,
    NotAllowed,
    Unknown {
        status: u16,
        body: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum GetFeedError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("robots url determination error")]
    RobotsDeterminingUrlError,

    #[error("robots fetch error")]
    RobotsFetchError,

    #[error("robots parsing error")]
    RobotsParsingError,

    #[error("unexpected response, expected feed")]
    UnexpectedFeed,

    #[error("error fetching feed")]
    FetchFeedError,

    #[error("error parsing feed")]
    ParseFeedError,
}

pub async fn get_feed(url: &str) -> Result<GetFeedResult, GetFeedError> {
    debug!("feed requested: {}", url);

    let url = if !url.starts_with("http") {
        debug!("url doesn't have scheme, assuming https");
        &format!("https://{}", url)
    } else {
        url
    };

    let robots_url = get_robots_url(url).map_err(|_| GetFeedError::RobotsDeterminingUrlError)?;
    debug!("checking robots at {robots_url}");

    let robots = CLIENT
        .get(robots_url)
        .send()
        .await
        .map_err(|_| GetFeedError::RobotsFetchError)?
        .bytes()
        .await
        .map_err(|_| GetFeedError::RobotsParsingError)?;
    let robots = Robot::new(USER_AGENT, &robots).map_err(|_| GetFeedError::RobotsParsingError)?;

    let allowed = robots.allowed(url);
    if !allowed {
        debug!("not allowed to fetch {url}");
        return Ok(GetFeedResult::NotAllowed);
    }

    let feed = fetch_feed(url).await.context("error fetching feed")?;
    match feed {
        FeedFetchResult::Feed { bytes, location } => {
            let (parsed_feed, entries) =
                parse_feed(&bytes, &url).map_err(|_| GetFeedError::ParseFeedError)?;
            let feed = NewFeed {
                title: parsed_feed.title,
                site_url: parsed_feed.site_url,
                feed_url: url.to_owned(),
            };
            Ok(GetFeedResult::Feed {
                feed,
                entries,
                icon: discover_favicon(&location.origin().ascii_serialization())
                    .await
                    .ok()
                    .flatten(),
            })
        }
        FeedFetchResult::Html { bytes, location } => {
            let (feed_urls, maybe_favicon_url) =
                discover_feed_and_favicon_url(&bytes, &url_to_string(&location))
                    .context("error discovering feed and favicon from html")?;

            if feed_urls.is_empty() {}

            if feed_urls.len() == 1 {
                let feed_url = &feed_urls[0];
                let feed = &fetch_feed(feed_url).await.context("error fetching feed")?;
                match feed {
                    FeedFetchResult::Feed {
                        bytes,
                        location: new_location,
                    } => {
                        let new_origin = new_location.origin().ascii_serialization();
                        let icon = if let Some(favicon_url) = maybe_favicon_url {
                            get_favicon(&favicon_url).await.ok().flatten()
                        } else if location.origin().ascii_serialization() != new_origin {
                            // if we didn't find a favicon and didn't visit origin yet,
                            // lets see if it has a favicon
                            discover_favicon(&new_origin).await.ok().flatten()
                        } else {
                            None
                        };

                        let (parsed_feed, entries) = parse_feed(&bytes, &feed_url)
                            .map_err(|_| GetFeedError::ParseFeedError)?;
                        let feed = NewFeed {
                            title: parsed_feed.title,
                            site_url: parsed_feed.site_url,
                            feed_url: feed_url.to_owned(),
                        };
                        Ok(GetFeedResult::Feed {
                            feed,
                            entries,
                            icon,
                        })
                    }
                    _ => Err(GetFeedError::UnexpectedFeed),
                }
            } else {
                Ok(GetFeedResult::DiscoveredMultiple(feed_urls))
            }
        }
        FeedFetchResult::NotFound => Ok(GetFeedResult::NotFound),
        FeedFetchResult::Unknown { status, body } => Err(GetFeedError::UnexpectedError(
            anyhow::anyhow!("unknown error fetching feed: {status}: {body}"),
        )),
    }
}

fn url_to_string(url: &Url) -> String {
    format!(
        "{origin}{path}",
        origin = url.origin().ascii_serialization(),
        path = url.path()
    )
}

#[derive(Debug)]
enum FeedFetchResult {
    Feed { bytes: Vec<u8>, location: Url },
    Html { bytes: Vec<u8>, location: Url },
    NotFound,
    Unknown { status: u16, body: String },
}

const USER_AGENT: &str = "rss reader";

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(redirect::Policy::limited(10))
        .build()
        .expect("client should be valid")
});

async fn fetch_feed(url: &str) -> anyhow::Result<FeedFetchResult> {
    debug!("fetch requested for {url}");

    let response = CLIENT
        .get(url)
        .send()
        .await
        .context("error executing request")?;
    let status = response.status();
    let location = response.url().to_owned();

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
                return Ok(FeedFetchResult::Html {
                    bytes: bytes.to_vec(),
                    location,
                });
            }

            if content_type.starts_with("text/xml")
                || content_type.starts_with("application/rss+xml")
                || content_type.starts_with("application/atom+xml")
                || content_type.starts_with("application/xml")
            {
                return Ok(FeedFetchResult::Feed {
                    bytes: bytes.to_vec(),
                    location,
                });
            }

            return Ok(FeedFetchResult::Unknown {
                body: String::from_utf8_lossy(&bytes).to_string(),
                status: status.as_u16(),
            });
        }
        _ => {
            return Ok(FeedFetchResult::Unknown {
                body: response.text().await.context("error reading response")?,
                status: status.as_u16(),
            });
        }
    }
}

fn parse_feed(bytes: &[u8], feed_url: &str) -> anyhow::Result<(ParsedFeed, Vec<NewEntry>)> {
    debug!("parsing feed as RSS");
    let feed = parse_rss(bytes).or_else(|_| {
        debug!("failed to parse as RSS, parsing as Atom");
        parse_atom(bytes, feed_url).map_err(|_| anyhow::anyhow!("failed to parse as Atom"))
    })?;
    debug!("parsed feed");

    // not using skipped for anything yet
    Ok((feed.0, feed.1))
}

struct ParsedFeed {
    title: String,
    site_url: Option<String>,
}

fn parse_rss(bytes: &[u8]) -> anyhow::Result<(ParsedFeed, Vec<NewEntry>, usize)> {
    let parsed = rss::Channel::read_from(bytes)?;
    let (entries, skipped) =
        parsed
            .items
            .iter()
            .fold((Vec::new(), 0usize), |(mut entries, mut skipped), item| {
                let title = match &item.title {
                    Some(title) => {
                        if title.trim().is_empty() {
                            warn!("title is empty for item {item:?}, skipping...");
                            skipped += 1;
                            return (entries, skipped);
                        }
                        title.to_string()
                    }
                    None => {
                        warn!("no title found for item {item:?}, skipping...");
                        skipped += 1;
                        return (entries, skipped);
                    }
                };

                let url = match item.link.to_owned() {
                    Some(url) => url,
                    None => {
                        warn!("no link found for item {item:?}, skipping...");
                        skipped += 1;
                        return (entries, skipped);
                    }
                };

                entries.push(NewEntry {
                    title,
                    url,
                    published_at: item
                        .pub_date
                        .to_owned()
                        .map(|date| DateTime::parse_from_rfc2822(&date).unwrap().into()),
                    comments_url: item
                        .comments
                        .to_owned()
                        .map(|comments| comments.to_string()),
                });

                (entries, skipped)
            });

    Ok((
        ParsedFeed {
            title: parsed.title.to_string(),
            site_url: Some(parsed.link.to_owned()),
        },
        entries,
        skipped,
    ))
}

fn parse_atom(bytes: &[u8], feed_url: &str) -> anyhow::Result<(ParsedFeed, Vec<NewEntry>, usize)> {
    let parsed = atom_syndication::Feed::read_from(bytes)?;

    let (entries, skipped) =
        parsed
            .entries
            .iter()
            .fold((Vec::new(), 0usize), |(mut entries, mut skipped), entry| {
                let title = entry.title.to_owned().value.to_string();
                if title.trim().is_empty() {
                    warn!("title is empty for entry {entry:?}, skipping...");
                    skipped += 1;
                    return (entries, skipped);
                }

                let url = match entry.links.first().map(|link| link.href.clone()) {
                    Some(url) => url,
                    None => {
                        warn!("no link found for entry {entry:?}, skipping...");
                        skipped += 1;
                        return (entries, skipped);
                    }
                };

                entries.push(NewEntry {
                    title,
                    url,
                    published_at: entry.published.map(|published| published.to_utc()),
                    comments_url: None,
                });
                (entries, skipped)
            });

    let site_url = parsed
        .links
        .iter()
        .find(|link| link.rel == "alternate")
        .or(parsed.links.iter().find(|link| link.href != feed_url))
        .map(|link| link.href.to_owned());

    Ok((
        ParsedFeed {
            title: parsed.title.to_string(),
            site_url,
        },
        entries,
        skipped,
    ))
}

fn discover_feed_and_favicon_url(
    mut bytes: &[u8],
    url: &str,
) -> anyhow::Result<(Vec<String>, Option<String>)> {
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

    let head_children = get_head_children(&dom)?.into_inner();

    let feed_links = head_children
        .iter()
        .filter_map(|child| match &child.data {
            NodeData::Element { name, attrs, .. } => {
                if name.local.as_ref() == "link" {
                    attrs
                        .borrow()
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
                format!(
                    "{}/{}",
                    url.trim_end_matches("/"),
                    href.trim_start_matches('/')
                )
            } else {
                href
            }
        })
        .collect::<Vec<String>>();
    debug!("found {} feed links", feed_links.len());

    let favicon_url = get_best_favicon_url(&head_children, url);
    debug!("found favicon url {favicon_url:?}");

    Ok((feed_links, favicon_url))
}

async fn discover_favicon(url: &str) -> anyhow::Result<Option<NewIcon>> {
    debug!("discovering favicon from {url}");

    let bytes = CLIENT
        .get(url)
        .send()
        .await
        .context("error executing request")?
        .bytes()
        .await
        .context("error reading response")?;

    let url = discover_favicon_url_from_html(&bytes[..], &url)?;
    if let Some(url) = url {
        return Ok(get_favicon(&url).await?);
    }

    Ok(None)
}

// must be its own non-async function
// because "`Rc<markup5ever_rcdom::Node>` cannot be sent between threads safely"
fn discover_favicon_url_from_html(bytes: &[u8], url: &str) -> anyhow::Result<Option<String>> {
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
    .read_from(&mut &bytes[..])
    .context("error parsing HTML")?;

    let url = get_best_favicon_url(&get_head_children(&dom)?.into_inner(), url);

    Ok(url)
}

async fn get_favicon(url: &str) -> anyhow::Result<Option<NewIcon>> {
    if url.starts_with("data:") {
        let parts = url.split(",").collect::<Vec<&str>>();
        if parts.len() < 2 {
            warn!("invalid data url {url}");
            return Ok(None);
        }

        let header = parts[0];
        let content_type = header.split(':').nth(1).and_then(|mt| mt.split(';').next());

        let content_type = match content_type {
            Some(ct) => ct,
            None => {
                warn!("invalid data url, no content type {url}");
                return Ok(None);
            }
        };

        let is_base64 = header.contains("base64");

        debug!("discovered icon as data url with content type {content_type:?}");

        let content = parts[1];
        let content = if is_base64 {
            base64::engine::general_purpose::STANDARD
                .decode(content)
                .context("error decoding data url as base64")?
        } else {
            percent_decode_str(content)
                .decode_utf8()
                .context("error decoding data url with utf8 percent encoding")?
                .as_bytes()
                .to_vec()
        };

        return Ok(Some(NewIcon {
            hash: hash_bytes(&content),
            data: content,
            content_type: content_type.to_string(),
        }));
    } else if url.starts_with("http") {
        debug!("discovered icon as url {url}");
        let icon = fetch_favicon(&url).await?;
        return Ok(icon);
    }

    Ok(None)
}

fn get_head_children(dom: &RcDom) -> anyhow::Result<RefCell<Vec<Rc<Node>>>> {
    Ok(dom
        .document
        .children
        .borrow()
        .iter()
        .find(|child| match &child.data {
            NodeData::Element { name, .. } => name.local.as_ref() == "html",
            _ => false,
        })
        .ok_or_else(|| anyhow::anyhow!("no html element found"))?
        .children
        .borrow()
        .iter()
        .find(|child| match &child.data {
            NodeData::Element { name, .. } => name.local.as_ref() == "head",
            _ => false,
        })
        .ok_or_else(|| anyhow::anyhow!("no head element found"))?
        .children
        .clone())
}

const ICON_RELS: &[&str] = &["icon", "shortcut icon", "apple-touch-icon"];

fn get_best_favicon_url(head_children: &Vec<Rc<Node>>, url: &str) -> Option<String> {
    head_children
        .iter()
        .filter_map(|child| match &child.data {
            NodeData::Element { name, attrs, .. } => {
                if name.local.as_ref() == "link" {
                    let rel_value = attrs
                        .borrow()
                        .iter()
                        .find(|attr| attr.name.local.as_ref() == "rel")
                        .map(|attr| attr.value.to_string());

                    if let Some(rel_value) = rel_value
                        && ICON_RELS.contains(&rel_value.as_str())
                    {
                        let href = attrs
                            .borrow()
                            .iter()
                            .find(|attr| attr.name.local.as_ref() == "href")
                            .map(|attr| attr.value.to_string());

                        href
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(|href| {
            // if href is a relative URL, make it absolute
            if !href.starts_with("http") && !href.starts_with("data:") {
                format!(
                    "{}/{}",
                    url.trim_end_matches("/"),
                    href.trim_start_matches('/')
                )
            } else {
                href
            }
        })
        .collect::<Vec<String>>()
        .first()
        .cloned()
}

async fn fetch_favicon(url: &str) -> anyhow::Result<Option<NewIcon>> {
    debug!("fetching favicon from {url}");
    let response = CLIENT
        .get(url)
        .send()
        .await
        .context("error executing request")?;
    let status = response.status();

    match status {
        StatusCode::OK => {
            let headers = response.headers().clone();
            let bytes = response.bytes().await.context("error reading response")?;
            let content_type = headers
                .get("Content-Type")
                .context("no content type found")?
                .to_str()
                .context("invalid content type")?
                .to_string();
            debug!("got favicon response with content type {content_type}");
            if content_type.starts_with("image/") {
                Ok(Some(NewIcon {
                    hash: hash_bytes(&bytes),
                    data: bytes.to_vec(),
                    content_type: content_type,
                }))
            } else {
                Err(anyhow::anyhow!("invalid content type: {content_type}"))
            }
        }
        StatusCode::NOT_FOUND => Ok(None),
        _ => Err(anyhow::anyhow!("unknown: {status}")),
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

static MAX_SYNCING_FEEDS: usize = 10;

pub async fn feed_sync_loop(data: Data) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        let feeds = data
            .get_feeds_to_sync(Utc::now() - chrono::Duration::hours(1))
            .await?;

        if feeds.len() == 0 {
            tracing::info!("no feeds to sync");
            continue;
        }

        tracing::info!("syncing {} feeds", feeds.len());

        stream::iter(feeds)
            .for_each_concurrent(MAX_SYNCING_FEEDS, |feed| {
                let data = data.clone();
                async move {
                    sync_feed(&data, feed.feed_url).await;
                }
            })
            .await;
    }
}

async fn sync_feed(data: &Data, url: String) {
    let result = get_feed(&url).await;

    match result {
        Ok(GetFeedResult::Feed {
            feed,
            entries,
            icon,
        }) => {
            let _ = data
                .upsert_feed_and_entries_and_icon(&feed, entries, icon)
                .await
                .map_err(|e| tracing::error!("error upserting feed: {e:#}"));

            tracing::info!("feed synced {:?}", feed);
        }
        Ok(GetFeedResult::DiscoveredMultiple(feed_urls)) => {
            tracing::warn!("discovered multiple feeds: {feed_urls:?}");
        }
        Ok(GetFeedResult::NotFound) => {
            tracing::warn!("feed not found");
        }
        Ok(GetFeedResult::NotAllowed) => {
            tracing::warn!("feed not allowed");
        }
        Ok(GetFeedResult::Unknown { status, body }) => {
            tracing::warn!("unknown error fetching feed: {status}: {body}");
        }
        Err(e) => {
            tracing::error!("error getting feed: {}", e);
        }
    }
}
