use crate::types::token_types::{CharClassType, CharSetType};

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub struct NFA {
    pub states: Vec<State>,
    pub start_state: usize,
    pub end_state: usize
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn connect_first_transition(&mut self, state_idx: usize) -> Result<(), String> {
        match self.transitions.clone() {
            Some((first, second)) => {
                match first {
                    Transition::Epsilon(_, _) => {
                        self.transitions = Some((Transition::Epsilon(Some(state_idx), None), second));
                    }
                    Transition::Literal(char, _) => {
                        self.transitions = Some((Transition::Literal(char, Some(state_idx)), second));
                    }
                    Transition::AnchorStart(_) => {
                        self.transitions = Some((Transition::AnchorStart(Some(state_idx)), second));
                    }
                    Transition::AnchorEnd(_) => {
                        self.transitions = Some((Transition::AnchorEnd(Some(state_idx)), second));
                    }
                    Transition::CaptureGroupStart(group, _) => {
                        self.transitions = Some((Transition::CaptureGroupStart(group, Some(state_idx)), second));
                    }
                    Transition::CaptureGroupEnd(group, _) => {
                        self.transitions = Some((Transition::CaptureGroupEnd(group, Some(state_idx)), second));
                    }
                    Transition::NonCapturingGroupStart(_) => {
                        self.transitions = Some((Transition::NonCapturingGroupStart(Some(state_idx)), second));
                    }
                    Transition::NonCapturingGroupEnd(_) => {
                        self.transitions = Some((Transition::NonCapturingGroupEnd(Some(state_idx)), second));
                    }
                }
            }
            None => {return Err(String::from("No transitions given"))}
        }
        Ok(())
    }
    pub fn connect_second_transition(&mut self, state_idx: usize) -> Result<(), String> {
        match self.transitions.clone() {
            Some((first, second)) => {
                match second {
                    Some(Transition::Epsilon(_, _)) => {
                        self.transitions = Some((first, Some(Transition::Epsilon(Some(state_idx), None))));
                    }
                    Some(Transition::Literal(char, _)) => {
                        self.transitions = Some((first, Some(Transition::Literal(char, Some(state_idx)))));
                    }
                    Some(Transition::AnchorStart(_)) => {
                        self.transitions = Some((first, Some(Transition::AnchorStart(Some(state_idx)))));
                    }
                    Some(Transition::AnchorEnd(_)) => {
                        self.transitions = Some((first, Some(Transition::AnchorEnd(Some(state_idx)))));
                    }
                    Some(Transition::CaptureGroupStart(group, _)) => {
                        self.transitions = Some((first, Some(Transition::CaptureGroupStart(group, Some(state_idx)))));
                    }
                    Some(Transition::CaptureGroupEnd(group, _)) => {
                        self.transitions = Some((first, Some(Transition::CaptureGroupEnd(group, Some(state_idx)))));
                    }
                    Some(Transition::NonCapturingGroupStart(_)) => {
                        self.transitions = Some((first, Some(Transition::NonCapturingGroupStart(Some(state_idx)))));
                    }
                    Some(Transition::NonCapturingGroupEnd(_)) => {
                        self.transitions = Some((first, Some(Transition::NonCapturingGroupEnd(Some(state_idx)))));
                    }
                    None => {
                        //We will assume if it is None, then it was a literal changed
                        //to a split state, in that case we will assume an Epsilon Transition
                        //return Err(String::from("No transitions set in position 2!"))
                        self.transitions.as_mut().unwrap().1 = Some(Transition::Epsilon(Some(state_idx), None))
                    }
                }
            }
            None => {return Err(String::from("No transitions given"))}
        }
        Ok(())
    }
    pub fn get_state_type(&self) -> StateType {
        self.state_type.clone()
    }
    pub fn get_transitions(&self) -> Option<(Transition, Option<Transition>)> {
        self.transitions.clone()
    }

    pub fn get_first_transition(&self) -> Option<Transition> {
        match self.transitions.clone() {
            Some((first, _)) => {
                Some(first)
            }
            None => {None}
        }
    }
    pub fn get_second_transition(&self) -> Option<Transition> {
        match self.transitions.clone() {
            Some((_, second)) => {
                second
            }
            None => {None}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    Epsilon(Option<usize>, Option<u8>), //Second u8 is to mark a (){0} group number
    Literal(CharToMatch, Option<usize>),
    AnchorStart(Option<usize>),
    AnchorEnd(Option<usize>),
    CaptureGroupStart(u8, Option<usize>),
    CaptureGroupEnd(u8, Option<usize>),
    NonCapturingGroupStart(Option<usize>),
    NonCapturingGroupEnd(Option<usize>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateType {
    Literal,
    Split,
    Match,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharToMatch {
    Literal(char),
    CharacterClass(CharClassType),
    CharacterSet(CharSetType, Vec<CharClassType>),
    Any
    //...
}