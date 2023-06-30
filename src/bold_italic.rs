// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use tokio::task::yield_now;

use crate::state::State;
use crate::Node;

pub async fn parse_bold_italic(state: &mut State) {
    let scan_position = state.scan_position;
    state.flush(scan_position).await;
    let start_position = state.scan_position;
    state.scan_position += 2;
    while state.get_byte(state.scan_position).await == Some(b'\'') {
        state.scan_position += 1;
    }
    let length = state.scan_position - start_position;
    if length < 3 {
        state.flushed_position = state.scan_position;
        state.nodes.push(Node::Italic {
            end: state.flushed_position,
            start: start_position,
        });
    } else if length < 5 {
        state.flushed_position = start_position + 3;
        state.nodes.push(Node::Bold {
            end: state.flushed_position,
            start: start_position,
        });
    } else {
        state.flushed_position = start_position + 5;
        state.nodes.push(Node::BoldItalic {
            end: state.flushed_position,
            start: start_position,
        });
    }
    yield_now().await;
}
