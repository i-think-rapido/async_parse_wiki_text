// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use async_recursion::async_recursion;
use tokio::task::yield_now;
use crate::{Node, TableCaption, TableRow, ListItem, Parameter, Warning, DefinitionListItem, configuration::Namespace, WikiText, Text};

pub struct OpenNode {
    pub nodes: Vec<Node>,
    pub start: usize,
    pub type_: OpenNodeType,
}

pub enum OpenNodeType {
    DefinitionList {
        items: Vec<DefinitionListItem>,
    },
    ExternalLink,
    Heading {
        level: u8,
    },
    Link {
        namespace: Option<Namespace>,
        target: Text,
    },
    OrderedList {
        items: Vec<ListItem>,
    },
    Parameter {
        default: Option<Vec<Node>>,
        name: Option<Vec<Node>>,
    },
    Preformatted,
    Table(Table),
    Tag {
        name: Text,
    },
    Template {
        name: Option<Vec<Node>>,
        parameters: Vec<Parameter>,
    },
    UnorderedList {
        items: Vec<ListItem>,
    },
}

pub struct State {
    pub flushed_position: usize,
    pub nodes: Vec<Node>,
    pub scan_position: usize,
    pub stack: Vec<OpenNode>,
    pub warnings: Vec<Warning>,
    pub wiki_text: WikiText,
}

pub struct Table {
    pub attributes: Vec<Node>,
    pub before: Vec<Node>,
    pub captions: Vec<TableCaption>,
    pub child_element_attributes: Option<Vec<Node>>,
    pub rows: Vec<TableRow>,
    pub start: usize,
    pub state: TableState,
}

pub enum TableState {
    Before,
    CaptionFirstLine,
    CaptionRemainder,
    CellFirstLine,
    CellRemainder,
    HeadingFirstLine,
    HeadingRemainder,
    Row,
    TableAttributes,
}

impl State {
    pub async fn flush(&mut self, end_position: usize) {
        flush(
            &mut self.nodes,
            self.flushed_position,
            end_position,
            self.wiki_text.clone(),
        ).await;
    }

    pub async fn get_byte(&self, position: usize) -> Option<u8> {
        self.wiki_text.as_ref().as_bytes().get(position).cloned()
    }

    pub async fn push_open_node(&mut self, type_: OpenNodeType, inner_start_position: usize) {
        let scan_position = self.scan_position;
        self.flush(scan_position).await;
        self.stack.push(OpenNode {
            nodes: std::mem::take(&mut self.nodes),
            start: scan_position,
            type_,
        });
        self.scan_position = inner_start_position;
        self.flushed_position = inner_start_position;
    }

    pub fn rewind(&mut self, nodes: Vec<Node>, position: usize) {
        self.scan_position = position + 1;
        self.nodes = nodes;
        if let Some(position_before_text) = match self.nodes.last() {
            Some(Node::Text { start, .. }) => Some(*start),
            _ => None,
        } {
            self.nodes.pop();
            self.flushed_position = position_before_text;
        } else {
            self.flushed_position = position;
        }
    }

    #[async_recursion]
    pub async fn skip_empty_lines(&mut self) {
        match self.stack.last() {
            Some(OpenNode {
                type_: OpenNodeType::Table { .. },
                ..
            }) => {
                self.scan_position -= 1;
                crate::table::parse_table_end_of_line(self, false).await;
            }
            _ => {
                crate::line::parse_beginning_of_line(self, None).await;
            }
        }
    }

    pub async fn skip_whitespace_backwards(&self, position: usize) -> usize {
        skip_whitespace_backwards(self.wiki_text.clone(), position).await
    }

    pub async fn skip_whitespace_forwards(&self, position: usize) -> usize {
        skip_whitespace_forwards(self.wiki_text.clone(), position).await
    }
}

pub async fn flush<'a>(
    nodes: &mut Vec<Node>,
    flushed_position: usize,
    end_position: usize,
    wiki_text: WikiText,
) {
    if end_position > flushed_position {
        nodes.push(Node::Text {
            end: end_position,
            start: flushed_position,
            value: WikiText::new(&wiki_text.as_ref()[flushed_position..end_position]),
        });
    }
    yield_now().await;
}

pub async fn skip_whitespace_backwards(wiki_text: WikiText, mut position: usize) -> usize {
    while position > 0 && matches!(wiki_text.as_ref().as_bytes()[position - 1], b'\t' | b'\n' | b' ') {
        position -= 1;
    }
    position
}

pub async fn skip_whitespace_forwards(wiki_text: WikiText, mut position: usize) -> usize {
    while matches!(wiki_text.as_ref().as_bytes().get(position).cloned(), Some(b'\t') | Some(b'\n') | Some(b' ')) {
        position += 1;
    }
    position
}
