// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use crate::state::{State};
use crate::{Node, Warning, WarningMessage};
use crate::state::OpenNodeType;
use crate::state::OpenNode;

pub async fn parse_beginning_of_line(state: &mut State<'_>, line_start_position: Option<usize>) {
    let mut has_line_break = false;
    'a: loop {
        match state.get_byte(state.scan_position).await {
            None => {
                if line_start_position.is_none() {
                    state.flushed_position = state.scan_position;
                }
                return;
            }
            Some(b'\t') => {
                state.scan_position += 1;
                loop {
                    match state.get_byte(state.scan_position).await {
                        None | Some(b'\n') => continue 'a,
                        Some(b'\t') | Some(b' ') => state.scan_position += 1,
                        Some(_) => break 'a,
                    }
                }
            }
            Some(b'\n') => {
                if has_line_break {
                    state.warnings.push(Warning {
                        end: state.scan_position + 1,
                        message: WarningMessage::RepeatedEmptyLine,
                        start: state.scan_position,
                    });
                }
                has_line_break = true;
                state.scan_position += 1;
            }
            Some(b' ') => {
                state.scan_position += 1;
                let start_position = state.scan_position;
                loop {
                    match state.get_byte(state.scan_position).await {
                        None => return,
                        Some(b'\n') => break,
                        Some(b'\t') | Some(b' ') => state.scan_position += 1,
                        Some(b'{') if state.get_byte(state.scan_position + 1).await == Some(b'|') => {
                            crate::table::start_table(state, line_start_position).await;
                            return;
                        }
                        Some(_) => {
                            if let Some(position) = line_start_position {
                                let position = state.skip_whitespace_backwards(position).await;
                                state.flush(position).await;
                            }
                            state.flushed_position = state.scan_position;
                            state.push_open_node(OpenNodeType::Preformatted, start_position).await;
                            return;
                        }
                    }
                }
            }
            Some(b'#') | Some(b'*') | Some(b':') | Some(b';') => {
                if let Some(position) = line_start_position {
                    let position = state.skip_whitespace_backwards(position).await;
                    state.flush(position).await;
                }
                state.flushed_position = state.scan_position;
                while crate::list::parse_list_item_start(state).await {}
                crate::list::skip_spaces(state).await;
                return;
            }
            Some(b'-') => {
                if state.get_byte(state.scan_position + 1).await == Some(b'-')
                    && state.get_byte(state.scan_position + 2).await == Some(b'-')
                    && state.get_byte(state.scan_position + 3).await == Some(b'-')
                {
                    if let Some(position) = line_start_position {
                        let position = state.skip_whitespace_backwards(position).await;
                        state.flush(position).await;
                    }
                    let start = state.scan_position;
                    state.scan_position += 4;
                    while state.get_byte(state.scan_position).await == Some(b'-') {
                        state.scan_position += 1;
                    }
                    state.nodes.push(Node::HorizontalDivider {
                        end: state.scan_position,
                        start,
                    });
                    while let Some(character) = state.get_byte(state.scan_position).await {
                        match character {
                            b'\t' | b' ' => state.scan_position += 1,
                            b'\n' => {
                                state.scan_position += 1;
                                state.skip_empty_lines().await;
                            }
                            _ => break,
                        }
                    }
                    state.flushed_position = state.scan_position;
                    return;
                }
                break;
            }
            Some(b'=') => {
                if let Some(position) = line_start_position {
                    let position = state.skip_whitespace_backwards(position).await;
                    state.flush(position).await;
                }
                crate::heading::parse_heading_start(state).await;
                return;
            }
            Some(b'{') => {
                if state.get_byte(state.scan_position + 1).await == Some(b'|') {
                    crate::table::start_table(state, line_start_position).await;
                    return;
                }
                break;
            }
            Some(_) => break,
        }
    }
    match line_start_position {
        None => state.flushed_position = state.scan_position,
        Some(position) => if has_line_break {
            let flush_position = state.skip_whitespace_backwards(position).await;
            state.flush(flush_position).await;
            state.nodes.push(Node::ParagraphBreak {
                end: state.scan_position,
                start: position,
            });
            state.flushed_position = state.scan_position;
        },
    }
}

pub async fn parse_end_of_line(state: &mut State<'_>) {
    match state.stack.last() {
        None => {
            let position = state.scan_position;
            state.scan_position += 1;
            parse_beginning_of_line(state, Some(position)).await;
        }
        Some(OpenNode {
            type_: OpenNodeType::DefinitionList { .. },
            ..
        })
        | Some(OpenNode {
            type_: OpenNodeType::OrderedList { .. },
            ..
        })
        | Some(OpenNode {
            type_: OpenNodeType::UnorderedList { .. },
            ..
        }) => {
            crate::list::parse_list_end_of_line(state).await;
        }
        Some(OpenNode {
            type_: OpenNodeType::ExternalLink { .. },
            ..
        }) => {
            crate::external_link::parse_external_link_end_of_line(state).await;
        }
        Some(OpenNode {
            type_: OpenNodeType::Heading { .. },
            ..
        }) => {
            crate::heading::parse_heading_end(state).await;
        }
        Some(OpenNode {
            type_: OpenNodeType::Link { .. },
            ..
        })
        | Some(OpenNode {
            type_: OpenNodeType::Parameter { .. },
            ..
        })
        | Some(OpenNode {
            type_: OpenNodeType::Tag { .. },
            ..
        })
        | Some(OpenNode {
            type_: OpenNodeType::Template { .. },
            ..
        }) => {
            state.scan_position += 1;
        }
        Some(OpenNode {
            type_: OpenNodeType::Preformatted,
            ..
        }) => {
            parse_preformatted_end_of_line(state).await;
        }
        Some(OpenNode {
            type_: OpenNodeType::Table { .. },
            ..
        }) => {
            crate::table::parse_table_end_of_line(state, true).await;
        }
    }
}

async fn parse_preformatted_end_of_line(state: &mut State<'_>) {
    if state.get_byte(state.scan_position + 1).await == Some(b' ') {
        let mut position = state.scan_position + 2;
        loop {
            match state.get_byte(position).await {
                None => break,
                Some(b'\t') | Some(b' ') => position += 1,
                Some(b'{') if state.get_byte(position + 1).await == Some(b'|') => {
                    break;
                }
                Some(b'|')
                    if state.get_byte(position + 1).await == Some(b'}') && state.stack.len() > 1
                        && match state.stack.get(state.stack.len() - 2) {
                            Some(OpenNode {
                                type_: OpenNodeType::Table { .. },
                                ..
                            }) => true,
                            _ => false,
                        } =>
                {
                    break;
                }
                Some(_) => {
                    let position = state.scan_position + 1;
                    state.flush(position).await;
                    state.scan_position += 2;
                    state.flushed_position = state.scan_position;
                    return;
                }
            }
        }
    }
    let open_node = state.stack.pop().unwrap();
    let position = state.skip_whitespace_backwards(state.scan_position).await;
    state.flush(position).await;
    state.scan_position += 1;
    let nodes = std::mem::replace(&mut state.nodes, open_node.nodes);
    state.nodes.push(Node::Preformatted {
        end: state.scan_position,
        nodes,
        start: open_node.start,
    });
    state.skip_empty_lines().await;
}
