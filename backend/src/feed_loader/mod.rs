use once_cell::sync::Lazy;
use reqwest::{Client, Response, StatusCode, redirect};
use std::collections::HashMap;
use std::time::Duration;
use texting_robots::Robot;
use url::Url;

use crate::{
    db::{NewEntry, NewFeed, NewIcon},
    feed_loader::{feed::parse_feed, html::Html},
};

mod feed;
mod html;
mod sync;
pub use sync::*;

pub const SYNC_RESULT_SUCCESS: &str = "success";
pub const SYNC_RESULT_NOT_MODIFIED: &str = "not_modified";
pub const SYNC_RESULT_PARSE_ERROR: &str = "parse_error";
pub const SYNC_RESULT_NOT_FOUND: &str = "not_found";
pub const SYNC_RESULT_DISALLOWED: &str = "disallowed";
pub const SYNC_RESULT_NEEDS_CHOICE: &str = "needs_choice";
pub const SYNC_RESULT_UNEXPECTED_HTML: &str = "unexpected_html";
pub const SYNC_RESULT_INVALID_URL: &str = "invalid_url";
pub const SYNC_RESULT_FETCH_ERROR: &str = "fetch_error";
pub const SYNC_RESULT_UNEXPECTED: &str = "unexpected";
pub const SYNC_RESULT_DB_ERROR: &str = "db_error";

pub fn sync_result_for_feed_result(result: &FeedResult) -> &'static str {
    match result {
        FeedResult::Loaded(_) => SYNC_RESULT_SUCCESS,
        FeedResult::NotModified => SYNC_RESULT_NOT_MODIFIED,
        FeedResult::NeedsChoice(_) => SYNC_RESULT_NEEDS_CHOICE,
        FeedResult::NotFound => SYNC_RESULT_NOT_FOUND,
        FeedResult::Disallowed => SYNC_RESULT_DISALLOWED,
    }
}

pub fn sync_result_for_error(err: &FeedError) -> &'static str {
    match err {
        FeedError::Parse => SYNC_RESULT_PARSE_ERROR,
        FeedError::UnexpectedHtml => SYNC_RESULT_UNEXPECTED_HTML,
        FeedError::InvalidUrl => SYNC_RESULT_INVALID_URL,
        FeedError::NotFound => SYNC_RESULT_NOT_FOUND,
        FeedError::Fetch(fetch_err) => match fetch_err {
            FetchError::InvalidUrl => SYNC_RESULT_INVALID_URL,
            FetchError::Disallowed => SYNC_RESULT_DISALLOWED,
            _ => SYNC_RESULT_FETCH_ERROR,
        },
        _ => SYNC_RESULT_UNEXPECTED,
    }
}

#[tracing::instrument(name = "load_feed", skip(etag, last_modified))]
pub async fn load_feed(
    url: &str,
    etag: Option<String>,
    last_modified: Option<String>,
) -> Result<FeedResult, FeedError> {
    let result = FeedLoader::new(url, etag, last_modified).run().await;
    match &result {
        Ok(FeedResult::Loaded(loaded)) => {
            tracing::info!("loaded feed: {}", loaded.feed.title)
        }
        Ok(FeedResult::NotModified) => tracing::info!("feed not modified"),
        Ok(FeedResult::NeedsChoice(urls)) => {
            tracing::info!("feed discovery found {} options", urls.len())
        }
        Ok(FeedResult::NotFound) => tracing::warn!("feed not found"),
        Ok(FeedResult::Disallowed) => tracing::warn!("feed disallowed by robots.txt"),
        Err(e) => tracing::error!("failed to load feed: {}", e),
    }
    result
}

#[tracing::instrument(name = "load_selected_feed", skip(etag, last_modified))]
pub async fn load_selected_feed(
    url: &str,
    etag: Option<String>,
    last_modified: Option<String>,
) -> Result<LoadedFeed, FeedError> {
    let result = FeedLoader::new_selected(url, etag, last_modified)
        .run()
        .await;
    match &result {
        Ok(loaded) => tracing::info!("loaded selected feed: {}", loaded.feed.title),
        Err(e) => tracing::error!("failed to load selected feed: {}", e),
    }
    result
}

const USER_AGENT: &str = "rss reader";
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .redirect(redirect::Policy::limited(10))
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client should be valid")
});

#[derive(Debug)]
pub enum FeedResult {
    Loaded(LoadedFeed),
    NotModified,
    NeedsChoice(Vec<String>),
    NotFound,
    Disallowed,
}

#[derive(Debug)]
pub struct LoadedFeed {
    pub feed: NewFeed,
    pub entries: Vec<NewEntry>,
    pub icon: Option<NewIcon>,
    pub http_etag: Option<String>,
    pub http_last_modified: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("invalid url")]
    InvalidUrl,

    #[error("failed to fetch: {0}")]
    Fetch(FetchError),

    #[error("got unexpected response: {0}")]
    UnexpectedResponse(#[from] reqwest::Error),

    #[error("failed to parse feed")]
    Parse,

    #[error("expected feed but got html")]
    UnexpectedHtml,

    #[error("not found")]
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("invalid url")]
    InvalidUrl,

    #[error("network error: {0}")]
    Network(reqwest::Error),

    #[error("disallowed by robots.txt")]
    Disallowed,

    #[error("error fetching robots.txt")]
    RobotsFetchFailed,

    #[error("error parsing robots.txt")]
    RobotsParseFailed,
}

impl From<reqwest::Error> for FetchError {
    fn from(e: reqwest::Error) -> Self {
        FetchError::Network(e)
    }
}

enum Content {
    Feed {
        bytes: Vec<u8>,
        final_url: Url,
        etag: Option<String>,
        last_modified: Option<String>,
    },
    Html {
        bytes: Vec<u8>,
        final_url: Url,
    },
    NotFound,
    NotModified,
}

trait HasConditionalHeaders {
    fn etag(&self) -> Option<&str>;
    fn last_modified(&self) -> Option<&str>;
}

struct Initial {
    etag: Option<String>,
    last_modified: Option<String>,
}

impl HasConditionalHeaders for Initial {
    fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }
    fn last_modified(&self) -> Option<&str> {
        self.last_modified.as_deref()
    }
}

struct Selected {
    etag: Option<String>,
    last_modified: Option<String>,
}

impl HasConditionalHeaders for Selected {
    fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }
    fn last_modified(&self) -> Option<&str> {
        self.last_modified.as_deref()
    }
}

struct FetchedHtml {
    bytes: Vec<u8>,
    final_url: Url,
}

struct FetchedFeed {
    bytes: Vec<u8>,
    final_url: Url,
    etag: Option<String>,
    last_modified: Option<String>,
}

struct ParsedFeed {
    meta: FeedMeta,
    entries: Vec<NewEntry>,
    final_url: Url,
    etag: Option<String>,
    last_modified: Option<String>,
}

struct FeedMeta {
    title: String,
    site_url: Option<String>,
}

enum Fetched {
    Feed(FeedLoader<FetchedFeed>),
    Html(FeedLoader<FetchedHtml>),
    NotFound,
    NotModified,
}

enum SelectedFetched {
    Feed(FeedLoader<FetchedFeed>),
    NotFound,
    NotModified,
}

struct FeedLoader<S> {
    robots: HashMap<String, Robot>,
    url: String,
    state: S,
}

impl FeedLoader<Initial> {
    fn new(url: &str, etag: Option<String>, last_modified: Option<String>) -> Self {
        Self {
            robots: HashMap::new(),
            url: ensure_scheme(url),
            state: Initial {
                etag,
                last_modified,
            },
        }
    }

    async fn run(self) -> Result<FeedResult, FeedError> {
        match self.fetch().await? {
            Fetched::NotFound => Ok(FeedResult::NotFound),
            Fetched::NotModified => Ok(FeedResult::NotModified),
            Fetched::Feed(loader) => loader.run().await.map(FeedResult::Loaded),
            Fetched::Html(loader) => loader.run().await,
        }
    }

    async fn fetch(mut self) -> Result<Fetched, FeedError> {
        let url = self.url.clone();
        let response = self.do_fetch(&url).await.map_err(FeedError::Fetch)?;

        Ok(match classify_response(response).await? {
            Content::NotFound => Fetched::NotFound,
            Content::NotModified => Fetched::NotModified,
            Content::Feed {
                bytes,
                final_url,
                etag,
                last_modified,
            } => {
                tracing::debug!(bytes = bytes.len(), %final_url, "fetched feed");
                Fetched::Feed(self.into_state(FetchedFeed {
                    bytes,
                    final_url,
                    etag,
                    last_modified,
                }))
            }
            Content::Html { bytes, final_url } => {
                tracing::debug!(bytes = bytes.len(), %final_url, "fetched html");
                Fetched::Html(self.into_state(FetchedHtml { bytes, final_url }))
            }
        })
    }
}

impl FeedLoader<Selected> {
    fn new_selected(url: &str, etag: Option<String>, last_modified: Option<String>) -> Self {
        Self {
            robots: HashMap::new(),
            url: ensure_scheme(url),
            state: Selected {
                etag,
                last_modified,
            },
        }
    }

    async fn run(self) -> Result<LoadedFeed, FeedError> {
        match self.fetch().await? {
            SelectedFetched::NotFound => Err(FeedError::NotFound),
            SelectedFetched::NotModified => Err(FeedError::NotFound), // Shouldn't happen in this flow
            SelectedFetched::Feed(loader) => loader.run().await,
        }
    }

    async fn fetch(mut self) -> Result<SelectedFetched, FeedError> {
        let url = self.url.clone();
        let response = self.do_fetch(&url).await.map_err(FeedError::Fetch)?;

        match classify_response(response).await? {
            Content::Feed {
                bytes,
                final_url,
                etag,
                last_modified,
            } => {
                tracing::debug!(bytes = bytes.len(), "fetched selected feed");
                Ok(SelectedFetched::Feed(self.into_state(FetchedFeed {
                    bytes,
                    final_url,
                    etag,
                    last_modified,
                })))
            }
            Content::Html { .. } => Err(FeedError::UnexpectedHtml),
            Content::NotFound => Ok(SelectedFetched::NotFound),
            Content::NotModified => Ok(SelectedFetched::NotModified),
        }
    }
}

impl FeedLoader<FetchedHtml> {
    async fn run(self) -> Result<FeedResult, FeedError> {
        let feed_urls = self.discover_feeds();

        match feed_urls.as_slice() {
            [] => Ok(FeedResult::NotFound),
            [single_url] => self
                .select(single_url.to_owned())
                .run()
                .await
                .map(FeedResult::Loaded),
            _ => Ok(FeedResult::NeedsChoice(feed_urls)),
        }
    }

    fn discover_feeds(&self) -> Vec<String> {
        let origin = self.state.final_url.origin().ascii_serialization();
        let html = Html::from_bytes(&self.state.bytes);

        let feed_urls: Vec<String> = html
            .feed_urls()
            .iter()
            .map(|href| absolutize(href, &origin))
            .collect();

        tracing::debug!(
            count = feed_urls.len(),
            ?feed_urls,
            "discovered feeds from html"
        );

        feed_urls
    }

    fn select(self, feed_url: String) -> FeedLoader<Selected> {
        FeedLoader {
            robots: self.robots,
            url: feed_url,
            state: Selected {
                etag: None,
                last_modified: None,
            },
        }
    }
}

impl FeedLoader<FetchedFeed> {
    async fn run(self) -> Result<LoadedFeed, FeedError> {
        self.parse()?.run().await
    }

    fn parse(self) -> Result<FeedLoader<ParsedFeed>, FeedError> {
        let (meta, entries) = parse_feed(&self.state.bytes, &self.url).map_err(|e| {
            tracing::debug!("failed to parse feed: {:?}", e);
            FeedError::Parse
        })?;

        tracing::debug!(title = meta.title, entries = entries.len(), "parsed feed");

        let final_url = self.state.final_url.to_owned();
        let etag = self.state.etag.clone();
        let last_modified = self.state.last_modified.clone();

        Ok(self.into_state(ParsedFeed {
            meta: FeedMeta {
                title: meta.title,
                site_url: meta.site_url,
            },
            entries,
            final_url,
            etag,
            last_modified,
        }))
    }
}

impl FeedLoader<ParsedFeed> {
    async fn run(mut self) -> Result<LoadedFeed, FeedError> {
        let icon = self.load_favicon().await;
        Ok(self.finish(icon))
    }

    fn finish(self, icon: Option<NewIcon>) -> LoadedFeed {
        LoadedFeed {
            feed: NewFeed {
                title: self.state.meta.title,
                site_url: self.state.meta.site_url,
                feed_url: self.url,
            },
            entries: self.state.entries,
            icon,
            http_etag: self.state.etag,
            http_last_modified: self.state.last_modified,
        }
    }

    async fn load_favicon(&mut self) -> Option<NewIcon> {
        let origin = self
            .state
            .meta
            .site_url
            .as_deref()
            .and_then(|site_url| {
                Url::parse(site_url)
                    .ok()
                    .map(|url| url.origin().ascii_serialization())
            })
            .unwrap_or_else(|| self.state.final_url.origin().ascii_serialization());

        let response = self.do_fetch_with_headers(&origin, None, None).await.ok()?;
        let bytes = response.bytes().await.ok()?;
        let favicon_urls = {
            let html = Html::from_bytes(&bytes);

            let mut favicon_urls = html.favicon_urls();
            favicon_urls.push(format!("{origin}/favicon.ico"));
            favicon_urls
        };

        tracing::debug!(candidates = favicon_urls.len(), "looking for favicon");

        for href in favicon_urls {
            let url = absolutize(&href, &origin);
            tracing::trace!("trying favicon url: {}", url);

            if let Some(icon) = parse_data_url(&url) {
                tracing::debug!("found favicon (data url)");
                return Some(icon);
            }

            if let Some(icon) = self.fetch_icon(&url).await {
                tracing::debug!(%url, "found favicon");
                return Some(icon);
            }
        }

        tracing::debug!("no favicon found");
        None
    }

    async fn fetch_icon(&mut self, url: &str) -> Option<NewIcon> {
        if !url.starts_with("http") {
            tracing::trace!("skipping non-http favicon url: {}", url);
            return None;
        }

        let response = self.do_fetch_with_headers(url, None, None).await.ok()?;

        if response.status() != StatusCode::OK {
            tracing::trace!("favicon fetch failed with status: {}", response.status());
            return None;
        }

        let content_type = response
            .headers()
            .get("content-type")?
            .to_str()
            .ok()?
            .to_owned();

        if !content_type.starts_with("image/") {
            tracing::trace!("favicon has non-image content-type: {}", content_type);
            return None;
        }

        let bytes = response.bytes().await.ok()?;
        tracing::trace!(
            "fetched favicon ({} bytes, type: {})",
            bytes.len(),
            content_type
        );

        Some(NewIcon {
            hash: hash_bytes(&bytes),
            data: bytes.to_vec(),
            content_type,
        })
    }
}

impl<S: HasConditionalHeaders> FeedLoader<S> {
    async fn do_fetch(&mut self, url: &str) -> Result<Response, FetchError> {
        let etag = self.state.etag().map(|s| s.to_owned());
        let last_modified = self.state.last_modified().map(|s| s.to_owned());
        self.do_fetch_with_headers(url, etag, last_modified).await
    }
}

impl<S> FeedLoader<S> {
    fn into_state<T>(self, state: T) -> FeedLoader<T> {
        FeedLoader {
            robots: self.robots,
            url: self.url,
            state,
        }
    }

    async fn do_fetch_with_headers(
        &mut self,
        url: &str,
        etag: Option<String>,
        last_modified: Option<String>,
    ) -> Result<Response, FetchError> {
        if !self.is_allowed(url).await? {
            return Err(FetchError::Disallowed);
        }

        let has_conditional = etag.is_some() || last_modified.is_some();
        let mut request = CLIENT.get(url);

        if let Some(ref etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(ref last_modified) = last_modified {
            request = request.header("If-Modified-Since", last_modified);
        }

        let response = request.send().await.map_err(FetchError::Network)?;
        tracing::debug!(
            status = %response.status(),
            conditional = has_conditional,
            "fetched {url}"
        );

        Ok(response)
    }

    async fn is_allowed(&mut self, _url: &str) -> Result<bool, FetchError> {
        Ok(true)
        // let origin = Url::parse(url)
        //     .map_err(|_| FetchError::InvalidUrl)?
        //     .origin()
        //     .ascii_serialization();

        // if let Some(robot) = self.robots.get(&origin) {
        //     let allowed = robot.allowed(url);
        //     tracing::trace!("robots.txt check (cached): {url} -> {allowed}");
        //     return Ok(allowed);
        // }

        // let robots_url = get_robots_url(url).map_err(|_| FetchError::InvalidUrl)?;
        // tracing::trace!("fetching robots.txt from: {robots_url}");

        // let robotstxt = CLIENT
        //     .get(robots_url)
        //     .send()
        //     .await
        //     .map_err(|_| FetchError::RobotsFetchFailed)?
        //     .bytes()
        //     .await
        //     .map_err(|_| FetchError::RobotsParseFailed)?;

        // let robot =
        //     Robot::new(USER_AGENT, &robotstxt).map_err(|_| FetchError::RobotsParseFailed)?;

        // let allowed = robot.allowed(url);
        // tracing::trace!("robots.txt check: {} -> {}", url, allowed);
        // self.robots.insert(origin, robot);

        // Ok(allowed)
    }
}

async fn classify_response(response: Response) -> Result<Content, FeedError> {
    let status = response.status();
    let final_url = response.url().to_owned();

    match status {
        StatusCode::NOT_FOUND => Ok(Content::NotFound),
        StatusCode::NOT_MODIFIED => Ok(Content::NotModified),

        StatusCode::OK => {
            let headers = response.headers();

            let content_type = headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            let etag = headers
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let last_modified = headers
                .get("last-modified")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let bytes = response.bytes().await?.to_vec();

            if content_type.starts_with("text/html") {
                Ok(Content::Html { bytes, final_url })
            } else {
                if !is_feed_content_type(&content_type) {
                    tracing::debug!("unknown content-type '{content_type}', assuming feed");
                }
                Ok(Content::Feed {
                    bytes,
                    final_url,
                    etag,
                    last_modified,
                })
            }
        }

        _ => Err(FeedError::UnexpectedResponse(
            response.error_for_status().unwrap_err(),
        )),
    }
}

fn is_feed_content_type(content_type: &str) -> bool {
    content_type.starts_with("text/xml")
        || content_type.starts_with("application/xml")
        || content_type.starts_with("application/rss+xml")
        || content_type.starts_with("application/atom+xml")
}

fn ensure_scheme(url: &str) -> String {
    if url.starts_with("http") {
        url.to_owned()
    } else {
        format!("https://{url}")
    }
}

fn absolutize(href: &str, origin: &str) -> String {
    if href.starts_with("http") || href.starts_with("data:") {
        href.to_owned()
    } else {
        format!(
            "{}/{}",
            origin.trim_end_matches('/'),
            href.trim_start_matches('/')
        )
    }
}

fn parse_data_url(url: &str) -> Option<NewIcon> {
    use base64::Engine;
    use percent_encoding::percent_decode_str;

    if !url.starts_with("data:") {
        return None;
    }

    let (header, content) = url.split_once(',')?;
    let content_type = header.split(':').nth(1)?.split(';').next()?;

    let data = if header.contains("base64") {
        base64::engine::general_purpose::STANDARD
            .decode(content)
            .ok()?
    } else {
        percent_decode_str(content)
            .decode_utf8()
            .ok()?
            .as_bytes()
            .to_vec()
    };

    Some(NewIcon {
        hash: hash_bytes(&data),
        data,
        content_type: content_type.to_owned(),
    })
}

fn hash_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
