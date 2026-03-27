use std::collections::HashSet;
use crate::types::nfa_types::{StateType, Transition, NFA};
use crate::matcher::matcher;

pub struct Simulator{
    nfa: NFA,
}

impl Simulator {
    pub fn new(nfa: NFA) -> Self {
        Simulator{
            nfa
        }
    }

    pub fn simulate(&mut self, input: String) -> Option<(String, Vec<usize>)> {
        let mut matches: (String, Vec<usize>) = (input.clone(), Vec::new());
        let mut state_set: HashSet<usize> = HashSet::new();
        state_set.insert(self.nfa.start_state);
        let mut next_state_set: HashSet<usize> = HashSet::new();

        let input_str_vec = input.chars().collect::<Vec<char>>();

        let mut i: usize = 0;
        let mut simulate = true;
        while simulate {
            if i == input_str_vec.len() {simulate = false;}
            let current_c = if i < input_str_vec.len() {input_str_vec[i]} else {input_str_vec[i - 1]};
            println!("current_c = {}", current_c);
            for state_idx in state_set.iter() {
                let state = self.nfa.states[*state_idx].clone();
                match state.get_state_type().clone() {
                    StateType::Literal | StateType::Split => {
                        match state.get_first_transition() {
                            Some(first_transition) => {
                                match first_transition {
                                    Transition::Literal(char_to_match, next) => {
                                        if matcher(current_c, char_to_match) {
                                            next_state_set.insert(next?);
                                            matches.1.push(i);
                                            i += 1;
                                        }
                                    }
                                    Transition::Epsilon(next, _)
                                    | Transition::CaptureGroupStart(_, next)
                                    | Transition::CaptureGroupEnd(_, next)
                                    | Transition::NonCapturingGroupStart(next)
                                    | Transition::NonCapturingGroupEnd(next)
                                    | Transition::AnchorStart(next)
                                    | Transition::AnchorEnd(next) => {
                                        self.calculate_epsilon_closure(&mut next_state_set, *state_idx);
                                        if state.get_state_type().clone() == StateType::Literal {continue;};
                                    }
                                }
                            }
                            None => {}
                        }
                        match state.get_second_transition() {
                            Some(second_transition) => {
                                match second_transition {
                                    Transition::Literal(char_to_match, next) => {
                                        if matcher(current_c, char_to_match) {
                                            next_state_set.insert(next?);
                                            matches.1.push(i);
                                            i += 1;
                                        }
                                    }
                                    Transition::Epsilon(next, _)
                                    | Transition::CaptureGroupStart(_, next)
                                    | Transition::CaptureGroupEnd(_, next)
                                    | Transition::NonCapturingGroupStart(next)
                                    | Transition::NonCapturingGroupEnd(next)
                                    | Transition::AnchorStart(next)
                                    | Transition::AnchorEnd(next) => {
                                        self.calculate_epsilon_closure(&mut next_state_set, *state_idx);
                                        continue
                                    }
                                }
                            }
                            None => {continue}
                        }
                    }
                    StateType::Match => {return Some(matches)}
                }
            }
            dbg!(&next_state_set);
            if next_state_set.is_empty() {return None}
            state_set = next_state_set.clone();
            dbg!(&state_set);
            next_state_set.clear();
        }
        None
    }

    fn calculate_epsilon_closure(&mut self, next_set: &mut HashSet<usize>, state_idx: usize) {
        let state = self.nfa.states[state_idx].clone();
        dbg!(&state);
        match state.get_state_type() {
            StateType::Literal => {
                match state.get_first_transition() {
                    Some(first_transition) => {
                        match first_transition {
                            Transition::Literal(_, _) => {return}
                            Transition::Epsilon(next, _)
                            | Transition::CaptureGroupStart(_, next)
                            | Transition::CaptureGroupEnd(_, next)
                            | Transition::NonCapturingGroupStart(next)
                            | Transition::NonCapturingGroupEnd(next)
                            | Transition::AnchorStart(next)
                            | Transition::AnchorEnd(next) => {
                                dbg!(&next);
                                if next.is_none() {return}
                                if next_set.contains(&next.unwrap()) {return}
                                next_set.insert(next.unwrap());
                                self.calculate_epsilon_closure(next_set, next.unwrap());
                            }
                        }
                    }
                    None => {return}
                }
            }
            StateType::Split => {
                match state.get_first_transition() {
                    Some(first_transition) => {
                        match first_transition {
                            Transition::Literal(_, _) => { return }
                            Transition::Epsilon(next, _)
                            | Transition::CaptureGroupStart(_, next)
                            | Transition::CaptureGroupEnd(_, next)
                            | Transition::NonCapturingGroupStart(next)
                            | Transition::NonCapturingGroupEnd(next)
                            | Transition::AnchorStart(next)
                            | Transition::AnchorEnd(next) => {
                                if next.is_none() { return }
                                if next_set.contains(&next.unwrap()) { return }
                                next_set.insert(next.unwrap());
                                self.calculate_epsilon_closure(next_set, next.unwrap());
                            }
                        }
                    }
                    None => {}
                }
                match state.get_second_transition() {
                    Some(second_transition) => {
                        match second_transition {
                            Transition::Literal(_, _) => { return }
                            Transition::Epsilon(next, _)
                            | Transition::CaptureGroupStart(_, next)
                            | Transition::CaptureGroupEnd(_, next)
                            | Transition::NonCapturingGroupStart(next)
                            | Transition::NonCapturingGroupEnd(next)
                            | Transition::AnchorStart(next)
                            | Transition::AnchorEnd(next) => {
                                if next.is_none() { return }
                                if next_set.contains(&next.unwrap()) { return }
                                next_set.insert(next.unwrap());
                                self.calculate_epsilon_closure(next_set, next.unwrap());
                            }
                        }
                    }
                    None => {return}
                }
            }
            StateType::Match => {next_set.insert(state_idx); return}
        }
    }

}