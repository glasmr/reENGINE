use crate::types::token_types::CharClassType;

pub struct NFA {
    states: Vec<State>
}
pub struct State {
    state_type: StateType,
    transitions: Option<(Transition, Option<Transition>)>,
}
impl State {
    pub fn new(state_type: StateType, transitions: Option<(Transition, Option<Transition>)>) -> State {
        State {
            state_type,
            transitions
        }
    }
    pub fn transition(&mut self, transitions: Option<(Transition, Option<Transition>)>) {
        self.transitions = transitions;
    }

    pub fn change_state_type(&mut self, state_type: StateType) {
        self.state_type = state_type
    }

    pub fn connect_next_state(&mut self, state_idx: usize) -> Result<(), String> {
        match self.transitions {
            Some((first, second)) => {
                match first {
                    Transition::Epsilon(mut _idx) => {_idx = Some(state_idx)}
                    Transition::Literal(_, mut _idx) => {_idx = Some(state_idx)}
                    Transition::AnchorStart(mut _idx) => {_idx = Some(state_idx)}
                    Transition::AnchorEnd(mut _idx) => {_idx = Some(state_idx)}
                }
                match second {
                    Some(Transition::Epsilon(mut _idx)) => {_idx = Some(state_idx)}
                    Some(Transition::Literal(_, mut _idx)) => {_idx = Some(state_idx)}
                    Some(Transition::AnchorStart(mut _idx)) => {_idx = Some(state_idx)}
                    Some(Transition::AnchorEnd(mut _idx)) => {_idx = Some(state_idx)}
                    None => {}
                }
            }
            None => {return Err(String::from("No transitions given"))}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Transition {
    Epsilon(Option<usize>),
    Literal(CharToMatch, Option<usize>),
    AnchorStart(Option<usize>),
    AnchorEnd(Option<usize>),
}


pub enum StateType {
    Literal,
    Split,
    Match
}

#[derive(Debug, Clone, Copy)]
pub enum CharToMatch {
    Literal(char),
    CharacterClass(CharClassType),
    Any
    //...
}