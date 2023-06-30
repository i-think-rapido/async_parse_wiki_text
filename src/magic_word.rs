// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use crate::state::State;
use crate::{Configuration, Node};

pub async fn parse_magic_word(state: &mut State, configuration: &Configuration) {
    if let Ok((match_length, _)) = configuration
        .magic_words
        .find(&state.wiki_text.as_ref()[state.scan_position + 2..])
    {
        let end_position = match_length + state.scan_position + 2;
        if state.get_byte(end_position).await == Some(b'_')
            && state.get_byte(end_position + 1).await == Some(b'_')
        {
            let scan_position = state.scan_position;
            state.flush(scan_position).await;
            state.flushed_position = end_position + 2;
            state.nodes.push(Node::MagicWord {
                end: state.flushed_position,
                start: state.scan_position,
            });
            state.scan_position = state.flushed_position;
            return;
        }
    }
    state.scan_position += 1;
}
