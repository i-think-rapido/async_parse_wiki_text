// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use crate::state::State;
use crate::{Configuration, Node};

pub async fn parse_character_entity(state: &mut State, configuration: &Configuration) {
    if let Ok((match_length, character)) = configuration
        .character_entities
        .find(&state.wiki_text.as_ref()[state.scan_position + 1..])
    {
        let start_position = state.scan_position;
        state.flush(start_position).await;
        state.flushed_position = match_length + start_position + 1;
        state.scan_position = state.flushed_position;
        state.nodes.push(Node::CharacterEntity {
            character,
            end: state.scan_position,
            start: start_position,
        });
    } else {
        state.scan_position += 1;
    }
}
