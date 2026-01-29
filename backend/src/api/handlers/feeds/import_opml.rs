use std::{collections::HashSet, convert::Infallible, time::Duration};

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use futures::{Stream, StreamExt, stream};
use quick_xml::{Reader, events::Event as XmlEvent};
use serde::Serialize;
use tracing::error;
use url::Url;

use crate::{
    api::{AppState, error::ApiError},
    feed_loader::{self, FeedResult},
};

const MAX_OPML_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct ImportStartResponse {
    status: &'static str,
    job_id: String,
    total: i64,
    skipped: i64,
}

#[derive(Debug, Serialize)]
struct ImportProgressItem {
    feed_url: String,
    status: String,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImportProgressEvent {
    job_id: String,
    status: String,
    total: i64,
    imported: i64,
    skipped: i64,
    failed: i64,
    done: bool,
    recent: Vec<ImportProgressItem>,
}

pub async fn import_opml(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let opml_bytes = read_opml_file(&mut multipart).await?;
    let urls = extract_opml_feed_urls(&opml_bytes)?;

    if urls.is_empty() {
        return Err(ApiError::BadRequest(
            "no feed urls found in opml".to_string(),
        ));
    }

    let existing_urls = state.data.get_existing_feed_urls(&urls).await?;
    let job = state
        .data
        .create_opml_import_job(&urls, &existing_urls)
        .await?;

    let urls_to_process: Vec<String> = urls
        .into_iter()
        .filter(|url| !existing_urls.contains(url))
        .collect();

    state.data.insert_stub_feeds(&urls_to_process).await?;

    let data = state.data.clone();
    let job_id = job.job_id.clone();
    tokio::spawn(async move {
        run_import_job(data, job_id, urls_to_process).await;
    });

    Ok((
        StatusCode::OK,
        Json(ImportStartResponse {
            status: "import_started",
            job_id: job.job_id,
            total: job.total,
            skipped: job.skipped,
        }),
    ))
}

pub async fn import_opml_events(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    state
        .data
        .get_opml_import_job(&job_id)
        .await?
        .ok_or(ApiError::NotFound("import job not found".to_string()))?;

    let data = state.data.clone();
    let stream = stream::unfold(
        (tokio::time::interval(Duration::from_millis(800)), false),
        move |(mut interval, done)| {
            let data = data.clone();
            let job_id = job_id.clone();
            async move {
                if done {
                    return None;
                }

                interval.tick().await;

                let payload = match build_progress_event(&data, &job_id).await {
                    Ok(event) => event,
                    Err(err) => ImportProgressEvent {
                        job_id,
                        status: "failed".to_string(),
                        total: 0,
                        imported: 0,
                        skipped: 0,
                        failed: 0,
                        done: true,
                        recent: vec![ImportProgressItem {
                            feed_url: "".to_string(),
                            status: "failed".to_string(),
                            error: Some(err.to_string()),
                        }],
                    },
                };

                let done_next = payload.done;
                let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
                let event = Event::default().data(json);

                Some((Ok(event), (interval, done_next)))
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    ))
}

async fn run_import_job(data: crate::db::Data, job_id: String, feed_urls: Vec<String>) {
    if feed_urls.is_empty() {
        let _ = data
            .update_opml_import_job_status(&job_id, "imported")
            .await;
        return;
    }

    let job_id_clone = job_id.clone();
    stream::iter(feed_urls)
        .for_each_concurrent(5, |url| {
            let data = data.clone();
            let job_id = job_id_clone.clone();
            async move {
                if let Err(err) = data
                    .update_opml_import_item(&job_id, &url, "running", None)
                    .await
                {
                    error!("error updating opml import item: {err:#}");
                }

                match feed_loader::load_feed(&url).await {
                    Ok(FeedResult::Loaded(loaded_feed)) => {
                        let upsert_res = data
                            .upsert_feed_and_entries_and_icon(
                                &loaded_feed.feed,
                                loaded_feed.entries,
                                loaded_feed.icon,
                            )
                            .await;

                        match upsert_res {
                            Ok(()) => {
                                if let Err(err) = data
                                    .update_opml_import_item(&job_id, &url, "imported", None)
                                    .await
                                {
                                    error!("error updating opml import item: {err:#}");
                                }
                                if let Err(err) = data
                                    .increment_opml_import_job_counts(&job_id, 1, 0, 0)
                                    .await
                                {
                                    error!("error updating opml import job counts: {err:#}");
                                }
                            }
                            Err(err) => {
                                mark_import_failure(&data, &job_id, &url, err.to_string()).await;
                            }
                        }
                    }
                    Ok(FeedResult::NeedsChoice(options)) => {
                        mark_import_failure(
                            &data,
                            &job_id,
                            &url,
                            format!("discovered_multiple ({})", options.len()),
                        )
                        .await;
                    }
                    Ok(FeedResult::NotFound) => {
                        mark_import_failure(&data, &job_id, &url, "not_found".to_string()).await;
                    }
                    Ok(FeedResult::Disallowed) => {
                        mark_import_failure(&data, &job_id, &url, "not_allowed".to_string()).await;
                    }
                    Err(err) => {
                        mark_import_failure(&data, &job_id, &url, err.to_string()).await;
                    }
                }
            }
        })
        .await;

    if let Err(err) = data
        .update_opml_import_job_status(&job_id, "imported")
        .await
    {
        error!("error updating opml import job status: {err:#}");
    }
}

async fn mark_import_failure(data: &crate::db::Data, job_id: &str, url: &str, reason: String) {
    if let Err(err) = data
        .update_opml_import_item(job_id, url, "failed", Some(&reason))
        .await
    {
        error!("error updating opml import item: {err:#}");
    }
    if let Err(err) = data.increment_opml_import_job_counts(job_id, 0, 0, 1).await {
        error!("error updating opml import job counts: {err:#}");
    }
}

async fn build_progress_event(
    data: &crate::db::Data,
    job_id: &str,
) -> Result<ImportProgressEvent, ApiError> {
    let job = data
        .get_opml_import_job(job_id)
        .await?
        .ok_or(ApiError::NotFound("import job not found".to_string()))?;
    let recent = data.get_opml_import_recent_items(job_id, 10).await?;
    let done = job.imported + job.skipped + job.failed >= job.total;

    Ok(ImportProgressEvent {
        job_id: job.id,
        status: job.status,
        total: job.total,
        imported: job.imported,
        skipped: job.skipped,
        failed: job.failed,
        done,
        recent: recent
            .into_iter()
            .map(|item| ImportProgressItem {
                feed_url: item.feed_url,
                status: item.status,
                error: item.error,
            })
            .collect(),
    })
}

async fn read_opml_file(multipart: &mut Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::BadRequest(err.to_string()))?
    {
        let field_name = field.name().unwrap_or("");
        if field_name.is_empty() || field_name == "file" || field_name == "opml" {
            let mut bytes = Vec::new();
            let mut field = field;
            while let Some(chunk) = field
                .chunk()
                .await
                .map_err(|err| ApiError::BadRequest(err.to_string()))?
            {
                if bytes.len() + chunk.len() > MAX_OPML_BYTES {
                    return Err(ApiError::BadRequest("opml file too large".to_string()));
                }
                bytes.extend_from_slice(&chunk);
            }
            return Ok(bytes);
        }
    }

    Err(ApiError::BadRequest("missing opml file".to_string()))
}

fn extract_opml_feed_urls(bytes: &[u8]) -> Result<Vec<String>, ApiError> {
    let mut reader = Reader::from_reader(std::io::Cursor::new(bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut urls = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(event)) | Ok(XmlEvent::Empty(event)) => {
                if event.name().as_ref() == b"outline" {
                    for attr in event.attributes().with_checks(false) {
                        let attr = attr.map_err(|err| ApiError::BadRequest(err.to_string()))?;
                        if attr.key.as_ref() == b"xmlUrl" {
                            let value = attr
                                .unescape_value()
                                .map_err(|err| ApiError::BadRequest(err.to_string()))?;
                            if let Some(url) = normalize_url(value.as_ref()) {
                                urls.push(url);
                            }
                        }
                    }
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(err) => {
                return Err(ApiError::BadRequest(format!("invalid opml: {err}")));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(dedup_urls(urls))
}

fn normalize_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        let parsed = Url::parse(trimmed).ok()?;
        match parsed.scheme() {
            "http" | "https" => Some(parsed.to_string()),
            _ => None,
        }
    }
}

fn dedup_urls(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for url in urls {
        if seen.insert(url.clone()) {
            deduped.push(url);
        }
    }

    deduped
}
