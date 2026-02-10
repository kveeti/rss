use axum::{extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;

use crate::api::{AppState, error::ApiError};

pub async fn export_opml(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let feeds = state
        .data
        .get_feeds_with_entry_counts()
        .await
        .map_err(|err| ApiError::UnexpectedError(err.into()))?;

    let opml = generate_opml(&feeds).map_err(|err| ApiError::UnexpectedError(err.into()))?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "text/xml; charset=utf-8")],
        opml,
    ))
}

fn generate_opml(feeds: &[crate::db::FeedWithEntryCounts]) -> anyhow::Result<String> {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::writer::Writer;
    use std::io::Cursor;

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // OPML root element
    let mut opml_start = BytesStart::new("opml");
    opml_start.push_attribute(("version", "2.0"));
    writer.write_event(Event::Start(opml_start))?;

    // Head section
    writer.write_event(Event::Start(BytesStart::new("head")))?;

    // Title
    writer.write_event(Event::Start(BytesStart::new("title")))?;
    writer.write_event(Event::Text(BytesText::new("Exported Feeds")))?;
    writer.write_event(Event::End(BytesEnd::new("title")))?;

    // Date created (RFC 822 format)
    let date_str = Utc::now().to_rfc2822();
    writer.write_event(Event::Start(BytesStart::new("dateCreated")))?;
    writer.write_event(Event::Text(BytesText::new(&date_str)))?;
    writer.write_event(Event::End(BytesEnd::new("dateCreated")))?;

    writer.write_event(Event::End(BytesEnd::new("head")))?;

    // Body section
    writer.write_event(Event::Start(BytesStart::new("body")))?;

    // Feed outlines
    for feed in feeds {
        let mut outline = BytesStart::new("outline");
        outline.push_attribute(("type", "rss"));
        outline.push_attribute(("text", feed.title.as_str()));
        outline.push_attribute(("xmlUrl", feed.feed_url.as_str()));
        if let Some(ref site_url) = feed.site_url {
            outline.push_attribute(("htmlUrl", site_url.as_str()));
        }

        writer.write_event(Event::Empty(outline))?;
    }

    writer.write_event(Event::End(BytesEnd::new("body")))?;

    writer.write_event(Event::End(BytesEnd::new("opml")))?;

    let result = writer.into_inner().into_inner();

    String::from_utf8(result).map_err(|e| anyhow::anyhow!(e))
}
