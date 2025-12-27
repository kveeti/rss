use chrono::DateTime;
use tracing::{debug, warn};

use crate::db::NewEntry;

pub fn parse_feed(bytes: &[u8], feed_url: &str) -> anyhow::Result<(ParsedFeed, Vec<NewEntry>)> {
    debug!("parsing feed as RSS");
    let feed = parse_rss(bytes).or_else(|_| {
        debug!("failed to parse as RSS, parsing as Atom");
        parse_atom(bytes, feed_url).map_err(|_| anyhow::anyhow!("failed to parse as Atom"))
    })?;
    debug!("parsed feed");

    // not using skipped for anything yet
    Ok((feed.0, feed.1))
}

pub struct ParsedFeed {
    pub title: String,
    pub site_url: Option<String>,
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
