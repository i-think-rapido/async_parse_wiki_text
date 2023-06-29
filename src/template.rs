// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use crate::state::{State, OpenNode};
use crate::state::OpenNodeType;
use crate::{Parameter, Warning, WarningMessage, Node};


pub async fn parse_parameter_name_end(state: &mut State<'_>) {
    let stack_length = state.stack.len();
    if stack_length > 0 {
        if let OpenNode {
            type_:
                OpenNodeType::Template {
                    name: Some(_),
                    parameters,
                },
            ..
        } = &mut state.stack[stack_length - 1]
        {
            let parameters_length = parameters.len();
            let name = &mut parameters[parameters_length - 1].name;
            if name.is_none() {
                crate::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    crate::state::skip_whitespace_backwards(state.wiki_text, state.scan_position).await,
                    state.wiki_text,
                ).await;
                state.flushed_position =
                    crate::state::skip_whitespace_forwards(state.wiki_text, state.scan_position + 1).await;
                state.scan_position = state.flushed_position;
                *name = Some(std::mem::take(&mut state.nodes));
                return;
            }
        }
    }
    state.scan_position += 1;
}

pub async fn parse_parameter_separator(state: &mut State<'_>) {
    match state.stack.last_mut() {
        Some(OpenNode {
            type_: OpenNodeType::Parameter { default, name },
            ..
        }) => {
            if name.is_none() {
                let position =
                    crate::state::skip_whitespace_backwards(state.wiki_text, state.scan_position).await;
                crate::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    position,
                    state.wiki_text,
                ).await;
                *name = Some(std::mem::take(&mut state.nodes));
            } else {
                crate::state::flush(
                    &mut state.nodes,
                    state.flushed_position,
                    state.scan_position,
                    state.wiki_text,
                ).await;
                *default = Some(std::mem::take(&mut state.nodes));
                state.warnings.push(Warning {
                    end: state.scan_position + 1,
                    message: WarningMessage::UselessTextInParameter,
                    start: state.scan_position,
                });
            }
            state.scan_position += 1;
            state.flushed_position = state.scan_position;
        }
        _ => unreachable!(),
    }
}

pub async fn parse_template_end(state: &mut State<'_>) {
    match state.stack.pop() {
        Some(OpenNode {
            nodes,
            start,
            type_: OpenNodeType::Parameter { default, name },
        }) => if state.get_byte(state.scan_position + 2).await == Some(b'}') {
            if let Some(name) = name {
                let start_position = state.scan_position;
                state.flush(start_position).await;
                let nodes = std::mem::replace(&mut state.nodes, nodes);
                state.nodes.push(Node::Parameter {
                    default: Some(default.unwrap_or(nodes)),
                    end: state.scan_position,
                    name,
                    start,
                });
            } else {
                let start_position = state.skip_whitespace_backwards(state.scan_position).await;
                state.flush(start_position).await;
                let nodes = std::mem::replace(&mut state.nodes, nodes);
                state.nodes.push(Node::Parameter {
                    default: None,
                    end: state.scan_position,
                    name: nodes,
                    start,
                });
            }
            state.scan_position += 3;
            state.flushed_position = state.scan_position;
        } else {
            state.warnings.push(Warning {
                end: state.scan_position + 2,
                message: WarningMessage::UnexpectedEndTagRewinding,
                start: state.scan_position,
            });
            state.rewind(nodes, start);
        },
        Some(OpenNode {
            nodes,
            start,
            type_:
                OpenNodeType::Template {
                    name,
                    mut parameters,
                },
        }) => {
            let position = state.skip_whitespace_backwards(state.scan_position).await;
            state.flush(position).await;
            state.scan_position += 2;
            state.flushed_position = state.scan_position;
            let name = match name {
                None => std::mem::replace(&mut state.nodes, nodes),
                Some(name) => {
                    let parameters_length = parameters.len();
                    let parameter = &mut parameters[parameters_length - 1];
                    parameter.end = position;
                    parameter.value = std::mem::replace(&mut state.nodes, nodes);
                    name
                }
            };
            state.nodes.push(Node::Template {
                end: state.scan_position,
                name,
                parameters,
                start,
            });
        }
        Some(OpenNode { nodes, start, .. }) => {
            state.warnings.push(Warning {
                end: state.scan_position + 2,
                message: WarningMessage::UnexpectedEndTagRewinding,
                start: state.scan_position,
            });
            state.rewind(nodes, start);
        }
        _ => {
            state.warnings.push(Warning {
                end: state.scan_position + 2,
                message: WarningMessage::UnexpectedEndTag,
                start: state.scan_position,
            });
            state.scan_position += 2;
        }
    }
}

pub async fn parse_template_separator(state: &mut State<'_>) {
    match state.stack.last_mut() {
        Some(OpenNode {
            type_: OpenNodeType::Template { name, parameters },
            ..
        }) => {
            let position = crate::state::skip_whitespace_backwards(state.wiki_text, state.scan_position).await;
            crate::state::flush(
                &mut state.nodes,
                state.flushed_position,
                position,
                state.wiki_text,
            ).await;
            state.flushed_position =
                crate::state::skip_whitespace_forwards(state.wiki_text, state.scan_position + 1).await;
            state.scan_position = state.flushed_position;
            if name.is_none() {
                *name = Some(std::mem::take(&mut state.nodes));
            } else {
                let parameters_length = parameters.len();
                let parameter = &mut parameters[parameters_length - 1];
                parameter.end = position;
                parameter.value = std::mem::take(&mut state.nodes);
            }
            parameters.push(Parameter {
                end: 0,
                name: None,
                start: state.scan_position,
                value: vec![],
            });
        }
        _ => unreachable!(),
    }
}

pub async fn parse_template_start(state: &mut State<'_>) {
    let scan_position = state.scan_position;
    if state.get_byte(state.scan_position + 2).await == Some(b'{') {
        let position = state.skip_whitespace_forwards(scan_position + 3).await;
        state.push_open_node(
            OpenNodeType::Parameter {
                default: None,
                name: None,
            },
            position,
        ).await;
    } else {
        let position = state.skip_whitespace_forwards(scan_position + 2).await;
        state.push_open_node(
            OpenNodeType::Template {
                name: None,
                parameters: vec![],
            },
            position,
        ).await;
    }
}
