// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use crate::state::State;
use crate::{Node, Warning, WarningMessage, Text};
use crate::state::OpenNodeType;

pub async fn parse_comment(state: &mut State) {
    let start_position = state.scan_position;
    let mut position = start_position;
    state.flush(position).await;
    position += 4;
    while let Some(character) = state.get_byte(position).await {
        match character {
            b'-' if state.get_byte(position + 1).await == Some(b'-')
                && state.get_byte(position + 2).await == Some(b'>') =>
            {
                position += 3;
                break;
            }
            b'<' if state.get_byte(position + 1).await == Some(b'/') => {
                if parse_end_tag(state, start_position, position).await {
                    return;
                }
                position += 2;
                continue;
            }
            _ => {}
        }
        position += 1;
    }
    state.flushed_position = position;
    state.scan_position = position;
    state.nodes.push(Node::Comment {
        end: state.scan_position,
        start: start_position,
    });
}

async fn parse_end_tag(
    state: &mut State,
    comment_start_position: usize,
    tag_start_position: usize,
) -> bool {
    let tag_name_start_position = tag_start_position + 2;
    let mut tag_name_end_position = tag_name_start_position;
    let tag_end_position = loop {
        match state.get_byte(tag_name_end_position).await {
            None | Some(b'/') | Some(b'<') => return false,
            Some(b'\t') | Some(b'\n') | Some(b' ') => {
                let tag_end_position = state.skip_whitespace_forwards(tag_name_end_position + 1).await;
                match state.get_byte(tag_end_position).await {
                    Some(b'>') => break tag_end_position,
                    _ => return false,
                }
            }
            Some(b'>') => break tag_name_end_position,
            _ => tag_name_end_position += 1,
        }
    } + 1;
    let tag_name = Text::new(&state.wiki_text.as_ref()[tag_name_start_position..tag_name_end_position].to_ascii_lowercase());
    let mut matched_node_index = None;
    for (open_node_index, open_node) in state.stack.iter().enumerate().rev() {
        if let OpenNodeType::Tag { name, .. } = &open_node.type_ {
            if name == &tag_name {
                matched_node_index = Some(open_node_index);
                break;
            }
        }
    }
    match matched_node_index {
        None => false,
        Some(open_node_index) => {
            if open_node_index < state.stack.len() - 1 {
                state.warnings.push(Warning {
                    end: tag_end_position,
                    message: WarningMessage::MissingEndTagRewinding,
                    start: tag_start_position,
                });
                state.stack.truncate(open_node_index + 2);
                let open_node = state.stack.pop().unwrap();
                state.rewind(open_node.nodes, open_node.start);
            } else {
                state.warnings.push(Warning {
                    end: tag_end_position,
                    message: WarningMessage::EndTagInComment,
                    start: tag_start_position,
                });
                state.nodes.push(Node::Comment {
                    end: tag_start_position,
                    start: comment_start_position,
                });
                let open_node = state.stack.pop().unwrap();
                state.flushed_position = tag_end_position;
                state.scan_position = state.flushed_position;
                let nodes = std::mem::replace(&mut state.nodes, open_node.nodes);
                state.nodes.push(Node::Tag {
                    end: state.scan_position,
                    name: tag_name,
                    nodes,
                    start: open_node.start,
                });
            }
            true
        }
    }
}
