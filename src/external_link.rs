// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use tokio::task::yield_now;

use crate::state::State;
use crate::{Warning, Configuration, Node, WarningMessage};
use crate::state::OpenNodeType;

pub async fn parse_external_link_end<'a>(
    state: &mut State,
    start_position: usize,
    nodes: Vec<Node>,
) {
    let scan_position = state.scan_position;
    state.flush(scan_position).await;
    state.scan_position += 1;
    state.flushed_position = state.scan_position;
    let nodes = std::mem::replace(&mut state.nodes, nodes);
    state.nodes.push(Node::ExternalLink {
        end: state.scan_position,
        nodes,
        start: start_position,
    });
}

pub async fn parse_external_link_end_of_line(state: &mut State) {
    let end = state.scan_position;
    let open_node = state.stack.pop().unwrap();
    state.warnings.push(Warning {
        end,
        message: WarningMessage::InvalidLinkSyntax,
        start: open_node.start,
    });
    state.rewind(open_node.nodes, open_node.start);
    yield_now().await;
}

pub async fn parse_external_link_start(state: &mut State, configuration: &Configuration) {
    let scheme_start_position = state.scan_position + 1;
    match configuration
        .protocols
        .find(&state.wiki_text.as_ref()[scheme_start_position..])
    {
        Err(_) => {
            state.scan_position = scheme_start_position;
        }
        Ok(_) => {
            state.push_open_node(OpenNodeType::ExternalLink, scheme_start_position).await;
        }
    }
    yield_now().await;
}
