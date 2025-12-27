use std::rc::Rc;

use html5ever::{ParseOpts, parse_document, tendril::TendrilSink, tree_builder::TreeBuilderOpts};
use markup5ever_rcdom::{Node, NodeData, RcDom};

pub struct Html {
    head_children: Vec<Node>,
}

impl Html {
    pub fn from_bytes(mut bytes: &[u8]) -> Self {
        let rc_dom = parse_document(
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
        // TODO
        .unwrap();

        let head_children = {
            let document_children = rc_dom.document.children.borrow();

            let html_elem = document_children
                .iter()
                .find(|child| match &child.data {
                    NodeData::Element { name, .. } => name.local.as_ref() == "html",
                    _ => false,
                })
                // TODO
                .unwrap();

            let html_children = html_elem.children.borrow();

            let head_elem = html_children
                .iter()
                .find(|child| match &child.data {
                    NodeData::Element { name, .. } => name.local.as_ref() == "head",
                    _ => false,
                })
                // TODO
                .unwrap();

            head_elem
                .children
                .take()
                .into_iter()
                .map(|rc| {
                    Rc::try_unwrap(rc)
                        // TODO
                        .unwrap()
                })
                .collect()
        };

        return Self { head_children };
    }

    pub fn favicon_urls(&self) -> Vec<String> {
        self.head_children
            .iter()
            .filter_map(|child| {
                let NodeData::Element { name, attrs, .. } = &child.data else {
                    return None;
                };

                if name.local.as_ref() != "link" {
                    return None;
                }

                let borrowed_attrs = attrs.borrow();

                let rel_value = borrowed_attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "rel")
                    .map(|attr| attr.value.as_ref())?;

                if !ICON_RELS.contains(&rel_value) {
                    return None;
                }

                borrowed_attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "href")
                    .map(|attr| attr.value.to_string())
            })
            .collect()
    }

    pub fn feed_urls(&self) -> Vec<String> {
        self.head_children
            .iter()
            .filter_map(|child| {
                let NodeData::Element { name, attrs, .. } = &child.data else {
                    return None;
                };

                if name.local.as_ref() != "link" {
                    return None;
                }

                let borrowed_attrs = attrs.borrow();

                let is_feed = borrowed_attrs.iter().any(|attr| {
                    let attr_name = attr.name.local.as_ref();
                    let attr_value = attr.value.as_ref();
                    let contains_feed_keyword =
                        attr_value.contains("rss") || attr_value.contains("atom");

                    (attr_name == "href" || attr_name == "type") && contains_feed_keyword
                });

                if !is_feed {
                    return None;
                }

                borrowed_attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "href")
                    .map(|attr| attr.value.to_string())
            })
            .collect()
    }
}

const ICON_RELS: &[&str] = &["icon", "shortcut icon", "apple-touch-icon"];
