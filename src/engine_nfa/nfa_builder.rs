use std::collections::{HashMap};
/// Builds a NFA data structure from the supplies AST.
/// The NFA is a cyclic-directed graph stored in a vec, rather
/// than a pointer based data structure.
/// Instead of a state pointing to another state in memory
/// using a pointer, they are instead pointed to by an
/// array index.
/// Each builder function is given a sub nfa
/// which is a tuple (usize, usize) each corresponding to
/// (sub_nfa_start_index, sub_nfa_end_index)
/// These indexes are then used in the construction of the nfa.
/// Each builder function returns the same (usize, usize)
/// corresponding to the start and end indexes


use crate::types::{
    ast_types::NodeAST,
    nfa_types::*
};
use crate::types::ast_types::QuantifierType;
use crate::types::nfa_types::EpsilonCondition;

pub struct BuilderNFA {
    nfa_vec: Vec<State>,
}

impl BuilderNFA {
    pub fn new() -> BuilderNFA {
        BuilderNFA {
            nfa_vec: Vec::new(),
        }
    }

    pub fn compile(&mut self, ast: &NodeAST) -> Result<NFA, String> {
        //dbg!(&ast);
        let (start, end) = match self.walk_tree(ast) {
            Ok(res) => res,
            Err(e) => return Err(format!("Error walking tree: {}", e)),
        };
        self.nfa_vec[end] = State::Match;
        let states = self.nfa_vec.to_vec();
        Ok(NFA{
            states,
            start_state: start,
            end_state: end
        })
    }

    fn walk_tree(&mut self, ast: &NodeAST) -> Result<(usize, usize), String> {
        match ast {
            NodeAST::Literal(char) => {self.build_simple_nfa(CharToMatch::Literal(*char))}

            NodeAST::CharacterClass(char_class_type) => {self.build_simple_nfa(CharToMatch::CharacterClass(*char_class_type))}

            NodeAST::CharacterSet(negated, character_sets) => {
                self.build_simple_nfa(CharToMatch::CharacterSet(*negated, character_sets.clone()))}

            NodeAST::Any => {self.build_simple_nfa(CharToMatch::Any)}

            NodeAST::AnchorStart(next) => {
                let nfa = self.walk_tree(next)?;
                self.start_anchor(nfa)
            }

            NodeAST::AnchorEnd(next) => {
                let nfa = self.walk_tree(next)?;
                self.end_anchor(nfa)
            }

            NodeAST::Group(next) => {
                let nfa = self.walk_tree(next)?;
                self.group(nfa)
            }

            NodeAST::Quantifier(next, quantifier) => {
                let nfa = self.walk_tree(next)?;
                match quantifier {
                    QuantifierType::ZeroOrMore => {self.zero_or_more_nfa(nfa)}
                    QuantifierType::OneOrMore => {self.one_or_more_nfa(nfa)}
                    QuantifierType::ZeroOrOne => {self.zero_or_one_nfa(nfa)}
                    QuantifierType::Repetition(m, n) => {self.repetition_nfa(nfa, *m, *n)}
                }
            }

            NodeAST::Concatenation(next_a, next_b) => {
                let state_a = self.walk_tree(next_a)?;
                let state_b = self.walk_tree(next_b)?;
                self.concat_nfa(state_a, state_b)
            }

            NodeAST::Alternation(next_a, next_b) => {
                let nfa_a = self.walk_tree(next_a)?;
                let nfa_b = self.walk_tree(next_b)?;

                self.alternate_nfa(nfa_a, nfa_b)
            }

            NodeAST::WordBoundary => {self.build_special_nfa(EpsilonCondition::WordBoundary)}

            NodeAST::NonWordBoundary => {self.build_special_nfa(EpsilonCondition::NonWordBoundary)}
        }
        
    }

    fn start_anchor(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let state_idx = self.nfa_vec.len();

        self.nfa_vec.push(State::Single(Transition::Epsilon(nfa.0, EpsilonCondition::StartAnchor)));

        Ok((state_idx, nfa.1))
    }

    fn end_anchor(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let state_idx = self.nfa_vec.len();

        self.nfa_vec.push(State::Single(Transition::DanglingTransition));
        self.nfa_vec[nfa.1] = State::Single(Transition::Epsilon(state_idx, EpsilonCondition::EndAnchor));


        Ok((nfa.0, state_idx))
    }

    fn group(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        Ok(nfa)
    }

    fn zero_or_more_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_index = self.nfa_vec.len();
        let end_state_index = begin_state_index + 1;

        //BEGIN state
        self.nfa_vec.push(State::Split(
            Transition::Epsilon(nfa.0, EpsilonCondition::Unconditional),
            Transition::Epsilon(end_state_index, EpsilonCondition::Unconditional)
        ));

        //END state
        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        self.nfa_vec[nfa.1] = State::Split(
            Transition::Epsilon(nfa.0, EpsilonCondition::Unconditional),
            Transition::Epsilon(end_state_index, EpsilonCondition::Unconditional)
        );

        Ok((begin_state_index, end_state_index))
    }

    fn one_or_more_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_index = self.nfa_vec.len();
        let end_state_index = begin_state_index + 1;

        self.nfa_vec.push(State::Single(Transition::Epsilon(nfa.0, EpsilonCondition::Unconditional)));
        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        self.nfa_vec[nfa.1] = State::Split(
            Transition::Epsilon(nfa.0, EpsilonCondition::Unconditional),
            Transition::Epsilon(end_state_index, EpsilonCondition::Unconditional)
        );

        Ok((begin_state_index, end_state_index))
    }

    fn zero_or_one_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let nfa_start_state_index = self.nfa_vec.len();
        let nfa_end_state_index = nfa_start_state_index + 1;

        self.nfa_vec.push(State::Split(
            Transition::Epsilon(nfa.0, EpsilonCondition::Unconditional),
            Transition::Epsilon(nfa.1, EpsilonCondition::Unconditional)
        ));

        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        self.nfa_vec[nfa.1] = State::Single(Transition::Epsilon(nfa_end_state_index, EpsilonCondition::Unconditional));

        Ok((nfa_start_state_index, nfa_end_state_index))
    }

    fn alternate_nfa(&mut self, nfa_a: (usize, usize), nfa_b: (usize, usize)) -> Result<(usize, usize), String> {
        let nfa_start_state_idx = self.nfa_vec.len();
        let nfa_end_state_index = nfa_start_state_idx + 1;

        self.nfa_vec.push(State::Split(
            Transition::Epsilon(nfa_a.0, EpsilonCondition::Unconditional),
            Transition::Epsilon(nfa_b.0, EpsilonCondition::Unconditional)
        ));

        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        self.nfa_vec[nfa_a.1] = State::Single(Transition::Epsilon(nfa_end_state_index, EpsilonCondition::Unconditional));
        self.nfa_vec[nfa_b.1] = State::Single(Transition::Epsilon(nfa_end_state_index, EpsilonCondition::Unconditional));

        Ok((nfa_start_state_idx, nfa_end_state_index))
    }
    
    fn concat_nfa(&mut self, nfa_a: (usize, usize), nfa_b: (usize, usize)) -> Result<(usize, usize), String> {
        self.nfa_vec[nfa_a.1] = State::Single(Transition::Epsilon(nfa_b.0, EpsilonCondition::Unconditional));
        Ok((nfa_a.0, nfa_b.1))
    }

    fn build_simple_nfa(&mut self, c: CharToMatch) -> Result<(usize, usize), String> {
        let state_index = self.nfa_vec.len();
        let end_state_index = state_index + 1;

        self.nfa_vec.push(State::Single(
            Transition::Literal(end_state_index, c)
        ));

        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        Ok((state_index, end_state_index))
    }

    fn build_special_nfa(&mut self, epsilon_condition: EpsilonCondition) -> Result<(usize, usize), String> {
        let state_index = self.nfa_vec.len();
        let end_state_index = state_index + 1;

        self.nfa_vec.push(State::Single(
            Transition::Epsilon(end_state_index, epsilon_condition)
        ));
        self.nfa_vec.push(State::Single(Transition::DanglingTransition));

        Ok((state_index, end_state_index))
    }

    fn repetition_nfa(&mut self, nfa: (usize, usize), m: usize, n: Option<usize>) -> Result<(usize, usize), String> {
        // if m and n are ==, then {m}
        // if n is None, then {m,}

        let mut m_nfa = nfa;

        if m == 0 {
            if let Some(n) = n {
                if n == 0 {
                    return self.build_special_nfa(EpsilonCondition::Unconditional);
                }
            }
        } else {
            for _ in 0..m - 1 {
                let new_nfa = self.copy_nfa(nfa)?;
                m_nfa = self.concat_nfa(m_nfa, new_nfa)?;
            }
        }

         match (m, n) {
            (m, Some(n)) => {
                if m == n { return Ok(m_nfa) }
                let n_remaining: usize = n - m;
                let mut n_nfa = self.copy_nfa(nfa)?;
                n_nfa = self.zero_or_one_nfa(n_nfa)?;
                for _ in 0..n_remaining - 1 {
                    let mut new_nfa = self.copy_nfa(nfa)?;
                    new_nfa = self.zero_or_one_nfa(new_nfa)?;
                    n_nfa = self.concat_nfa(n_nfa, new_nfa)?;
                }
                self.concat_nfa(m_nfa, n_nfa)
            }
            (_m, None) => {
                let mut last_nfa = self.copy_nfa(nfa)?;
                last_nfa = self.zero_or_more_nfa(last_nfa)?;
                let final_nfa = self.concat_nfa(last_nfa, m_nfa)?;
                Ok(final_nfa)
            }
        }
    }
    fn copy_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let copy_start_state_idx = self.nfa_vec.len();

        let mut mappings: HashMap<usize, usize> = HashMap::new(); //maps old -> new nodes

        let start_state_idx = nfa.0;
        self.deep_copy_dfs(start_state_idx, &mut mappings, nfa.1);

        let copy_end = self.nfa_vec.len();


        for i in copy_start_state_idx .. copy_end {
            match &mut self.nfa_vec[i] {
                State::Single(transition) => {
                    if transition.next_state().is_none() {continue}
                    let current_next_state = transition.next_state().unwrap();
                    if !mappings.contains_key(&current_next_state) {continue}
                    transition.update_next_state(mappings[&current_next_state]);
                }
                State::Split(transition_1, transition_2) => {
                    let next_next_state_1 = transition_1.next_state().unwrap();
                    let next_next_state_2 = transition_2.next_state().unwrap();

                    transition_1.update_next_state(mappings[&next_next_state_1]);
                    transition_2.update_next_state(mappings[&next_next_state_2]);
                }
                State::Match => {}
            }
        }

        Ok((copy_start_state_idx, mappings[&nfa.1]))
    }

    fn deep_copy_dfs(&mut self, state_idx: usize, mappings: &mut HashMap<usize, usize>, last_state: usize) {
        if mappings.contains_key(&state_idx) {return}
        let new_state_idx = self.nfa_vec.len();
        self.nfa_vec.push(self.nfa_vec[state_idx].clone());
        mappings.insert(state_idx, new_state_idx);

        if last_state == state_idx {
            //self.nfa_vec[new_state_idx] = State::Single(Transition::DanglingTransition);
            return
        }

        match &self.nfa_vec[state_idx] {
            State::Single(transition) => {
                if transition.next_state().is_none() {return}
                let next_state_idx = transition.next_state().unwrap();
                self.deep_copy_dfs(next_state_idx, mappings, last_state);
            }
            State::Split(transition_1, transition_2) => {
                let new_state_idx_1 = transition_1.next_state().unwrap();
                let new_state_idx_2 = transition_2.next_state().unwrap();
                self.deep_copy_dfs(new_state_idx_1, mappings, last_state);
                self.deep_copy_dfs(new_state_idx_2, mappings, last_state);
            }
            _ => {return}
        }
    }
}
