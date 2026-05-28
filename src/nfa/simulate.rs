use std::cmp::PartialEq;
use std::collections::HashSet;
use crate::nfa::types::nfa_types::{EpsilonCondition, State, Transition, NFA};
use crate::nfa::matcher::matcher;

pub struct Simulator {
    nfa: NFA,
    input_str_vec: Vec<char>,
    str_pos: usize
}

impl Simulator {
    pub fn new(nfa: NFA) -> Self {
        Simulator{
            nfa,
            input_str_vec: Vec::new(),
            str_pos: 0
        }
    }


    /*pub fn simulate(&mut self, input: String, search_type: SearchType) -> bool {
        self.input_str_vec = input.chars().collect();
        let start_state = self.nfa.start_state;
        let start_set: HashSet<usize> = self.epsilon_closure(&HashSet::from([start_state]));
        let mut state_set: HashSet<usize> = start_set.clone();
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
    }*/

    //TAKES: Input String, findall bool option
    //RETURNS: Vec of matches (start_idx, end_idx)
    pub fn simulate_nonoverlap(&mut self, input: String, findall: bool) -> Vec<(usize, usize)> {
        let mut matches: Vec<(usize, usize)> = Vec::new();
        let mut n_matches: usize = 0;
        self.input_str_vec = input.chars().collect();

        let start_state = (self.nfa.start_state, self.str_pos);
        let mut state_set: HashSet<(usize, usize)> = HashSet::new();
        state_set.insert(start_state);

        while self.str_pos < self.input_str_vec.len() {
            let current_char: char = self.input_str_vec[self.str_pos];
            let mut contains_start: bool = false;
            for (state, idx) in &state_set {
                if *state == start_state.0 {
                    contains_start = true;
                    break;
                }
            }
            if !contains_start {state_set.insert((start_state.0, self.str_pos));}
            state_set = self.epsilon_closure(&state_set);
            state_set = self.move_next_state(&state_set, current_char);
            for state in &state_set {
                if let State::Match = self.nfa.states[(*state).0] {
                    matches.push(((*state).0, self.str_pos));
                }
                if n_matches == 0 && !findall {
                    return matches;
                }
                n_matches += 1;
            }
        }

        matches
    }

    pub fn simulate_overlapping(&mut self, input: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        unimplemented!();
    }

    //TAKES: state set hashmap (state idx, str pos idx), char to match
    //RETURNS; State set hashmap (state idx, str pos idx)
    fn move_next_state(&mut self, state_set: &HashSet<(usize, usize)>, c: char) -> HashSet<(usize, usize)> {
        let mut next_state_set: HashSet<(usize, usize)> = HashSet::new();

        for state in state_set {
            match self.nfa.states[(*state).0].clone() {
                State::Single(transition) => {
                    if let Transition::Literal(next, to_match) = transition {
                        if matcher(c, &to_match) {
                            next_state_set.insert((next, (*state).1));
                        }
                    }
                }
                _ => {}
            }
        }
        //dbg!(&next_state_set);
        next_state_set
    }

    //TAKES: state set hashmap (state idx, str pos idx)
    //RETURNS; State set hashmap (state idx, str pos idx)
    fn epsilon_closure(&mut self, states: &HashSet<(usize, usize)>) -> HashSet<(usize, usize)> {
        //let mut stack: Vec<usize> = Vec::from(states.clone().into_iter().collect::<Vec<usize>>());
        let mut stack: Vec<(usize, usize)> = Vec::from(states.clone().into_iter().collect::<Vec<(usize, usize)>>());
        
        let mut visited: HashSet<usize> = HashSet::from(states.clone().into_iter().map(|(x, _)| x).collect::<HashSet<_>>());
        let mut next_state_set: HashSet<(usize, usize)> = HashSet::from(states.clone());

        while !stack.is_empty() {
            let current_state = stack.pop().unwrap();
            match self.nfa.states[current_state.0].clone() {
                State::Single(Transition::Epsilon(next, condition)) => {
                    if !visited.contains(&next){
                        if self.epsilon_condition(&condition) {
                            stack.push((next, current_state.1));
                            next_state_set.insert((next, current_state.1));
                            visited.insert(next);
                        }
                    }
                }
                State::Split(transition_1, transition_2) => {
                    if let Transition::Epsilon(next, condition) = transition_1 {
                        if !visited.contains(&next){
                            if self.epsilon_condition(&condition) {
                                stack.push((next, current_state.1));
                                next_state_set.insert((next, current_state.1));
                                visited.insert(next);
                            }
                        }
                    }
                    if let Transition::Epsilon(next, condition) = transition_2 {
                        if !visited.contains(&next){
                            if self.epsilon_condition(&condition) {
                                stack.push((next, current_state.1));
                                next_state_set.insert((next, current_state.1));
                                visited.insert(next);
                            }
                        }
                    }
                }
                State::Match => {next_state_set.insert((current_state.0, current_state.1));}
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