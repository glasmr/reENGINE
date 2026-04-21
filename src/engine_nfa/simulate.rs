use std::cmp::PartialEq;
use std::collections::HashSet;
use crate::types::nfa_types::{EpsilonCondition, State, Transition, NFA};
use crate::matcher::matcher;

pub struct Simulator {
    nfa: NFA,
    input_str_vec: Vec<char>,
    str_pos: usize
}
#[derive(Debug, PartialEq)]
pub enum SearchType {Substring, Fullstring}

impl Simulator {
    pub fn new(nfa: NFA) -> Self {
        Simulator{
            nfa,
            input_str_vec: Vec::new(),
            str_pos: 0
        }
    }


    pub fn simulate(&mut self, input: String, search_type: SearchType) -> bool {
        self.input_str_vec = input.chars().collect();
        let start_state = self.nfa.start_state;
        let start_set: HashSet<usize> = self.epsilon_closure(&HashSet::from([start_state]));
        let mut state_set: HashSet<usize> = start_set.clone();
        //dbg!(&state_set);
        while self.str_pos < self.input_str_vec.len() {
            let current_char: char = self.input_str_vec[self.str_pos];
            if state_set.is_empty() {return false}
            state_set = self.move_next_state(&state_set, current_char);
            //println!("Moving state: {}", current_char);
            state_set = self.epsilon_closure(&state_set);
            self.str_pos += 1;
            if search_type == SearchType::Substring {
                state_set.extend(&self.epsilon_closure(&HashSet::from([start_state])));
                for state in &state_set {
                    if let State::Match = self.nfa.states[*state] {
                        return true;
                    }
                }
            }
        }
        for state in &state_set {
            if let State::Match = self.nfa.states[*state] {
                return true;
            }
        }
        false
    }
    fn move_next_state(&mut self, state_set: &HashSet<usize>, c: char) -> HashSet<usize> {
        let mut next_state_set: HashSet<usize> = HashSet::new();

        for state in state_set {
            match self.nfa.states[*state].clone() {
                State::Single(transition) => {
                    if let Transition::Literal(next, to_match) = transition {
                        if matcher(c, &to_match) {
                            next_state_set.insert(next);
                        }
                    }
                }
                _ => {}
            }
        }
        //dbg!(&next_state_set);
        next_state_set
    }
    fn epsilon_closure(&mut self, states: &HashSet<usize>) -> HashSet<usize> {
        let mut stack: Vec<usize> = Vec::from(states.clone().into_iter().collect::<Vec<usize>>());
        let mut visited: HashSet<usize> = HashSet::from(states.clone());
        let mut next_state_set: HashSet<usize> = HashSet::from(states.clone());

        while !stack.is_empty() {
            let current_state = stack.pop().unwrap();
            match self.nfa.states[current_state].clone() {
                State::Single(Transition::Epsilon(next, condition)) => {
                    if !visited.contains(&next){
                        if self.epsilon_condition(&condition) {
                            stack.push(next);
                            next_state_set.insert(next);
                            visited.insert(next);
                        }
                    }
                }
                State::Split(transition_1, transition_2) => {
                    if let Transition::Epsilon(next, condition) = transition_1 {
                        if !visited.contains(&next){
                            if self.epsilon_condition(&condition) {
                                stack.push(next);
                                next_state_set.insert(next);
                                visited.insert(next);
                            }
                        }
                    }
                    if let Transition::Epsilon(next, condition) = transition_2 {
                        if !visited.contains(&next){
                            if self.epsilon_condition(&condition) {
                                stack.push(next);
                                next_state_set.insert(next);
                                visited.insert(next);
                            }
                        }
                    }
                }
                State::Match => {next_state_set.insert(current_state);}
                _ => {}
            }
        }
        next_state_set
    }

    fn epsilon_condition(&self, condition: &EpsilonCondition) -> bool {
        match condition {
            EpsilonCondition::Unconditional => true,
            EpsilonCondition::StartAnchor => {self.str_pos == 0}
            EpsilonCondition::EndAnchor => {self.str_pos == self.input_str_vec.len() - 1}
            EpsilonCondition::WordBoundary => {
                self.is_word_boundary(&self.str_pos, &self.input_str_vec)
            }
            EpsilonCondition::NonWordBoundary => {!self.is_word_boundary(&self.str_pos, &self.input_str_vec)}
        }
    }
    fn is_word(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }
    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }
    fn is_word_boundary(&self, str_pos: &usize, input: &Vec<char>) -> bool {
        let left = if *str_pos == 0 {
            false
        } else {
            self.is_word(input[*str_pos - 1]) || self.is_digit(input[*str_pos - 1])
        };
        let right = if *str_pos >= input.len() - 1 {
            false
        } else {
            self.is_word(input[*str_pos + 1]) || self.is_digit(input[*str_pos + 1])
        };
        left != right
    }
}