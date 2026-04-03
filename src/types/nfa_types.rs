use crate::types::token_types::{CharClassType, CharSetType};

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub struct NFA {
    pub states: Vec<State>,
    pub start_state: usize,
    pub end_state: usize
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Single(Transition),
    Split(Transition, Transition),
    Match,
}

impl State {
    #[allow(dead_code)]
    pub fn state_type(&self) -> &str {
        match self {
            State::Single(_) => {"literal"},
            State::Split(_, _) => {"split"},
            State::Match => {"match"},
        }
    }

    #[allow(dead_code)]
    pub fn transition_next(&self) -> Option<(usize, Option<usize>)> {
        if let State::Single(n) = self {
            let next = n.next_state().unwrap();
            return Some((next, None))
        }
        if let State::Split(first, second) = self {
            let first_next = first.next_state().unwrap();
            let second_next = second.next_state().unwrap();
            return Some((first_next, Some(second_next)))
        }
        None
    }

    #[allow(dead_code)]
    pub fn update_first_transition_next(&mut self, next_state: usize) {
        if let State::Single(next) = self {
            next.update_next_state(next_state)
        }
        if let State::Split(first, _) = self {
            first.update_next_state(next_state)
        }
    }

    #[allow(dead_code)]
    pub fn update_second_transition_next(&mut self, next_state: usize) {
        if let State::Split(_, second) = self {
            second.update_next_state(next_state)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    Literal(usize, CharToMatch),
    Epsilon(usize, EpsilonCondition),
    DanglingTransition,
}

impl Transition {
    pub fn next_state(&self) -> Option<usize> {
        if let Transition::Literal(n, _) = self {return Some(*n)}
        if let Transition::Epsilon(n, _) = self {return Some(*n)}
        None
    }

    #[allow(dead_code)]
    pub fn next_char(&self) -> Option<CharToMatch> {
        if let Transition::Literal(_, char) = self {return Some(char.clone())}
        None
    }

    #[allow(dead_code)]
    pub fn next_condition(&self) -> Option<EpsilonCondition> {
        if let Transition::Epsilon(_, condition) = self {
            return Some(condition.clone())
        }
        None
    }

    pub fn update_next_state(&mut self, next_state: usize) {
        if let Transition::Literal(n, _) = self {
            *n = next_state
        }
        if let Transition::Epsilon(n, _) = self {
            *n = next_state
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EpsilonCondition {
    Unconditional,
    StartAnchor,
    EndAnchor,
    WordBoundary,
    NonWordBoundary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharToMatch {
    Literal(char),
    CharacterClass(CharClassType),
    CharacterSet(CharSetType, Vec<CharClassType>),
    Any
}