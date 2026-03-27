use std::collections::{HashMap, HashSet};
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
        let (start, end) = match self.walk_tree(ast) {
            Ok(res) => res,
            Err(e) => return Err(format!("Error walking tree: {}", e)),
        };
        self.nfa_vec[end].change_state_type(StateType::Match);
        self.nfa_vec[end].transition(Some((Transition::Epsilon(None, None), None)));
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

            NodeAST::CaptureGroup(next, grp_n) => {
                let nfa = self.walk_tree(next)?;
                self.capture_group(nfa, *grp_n)
            }

            NodeAST::NonCapturingGroup(next) => {
                let nfa = self.walk_tree(next)?;
                self.non_capture_group(nfa)
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
        }
        
    }

    fn start_anchor(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let state_idx = self.nfa_vec.len();

        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::AnchorStart(Some(nfa.0)), None))
        ));

        Ok((state_idx, nfa.1))
    }

    fn end_anchor(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let state_idx = self.nfa_vec.len();

        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::AnchorEnd(Some(nfa.0)), None))
        ));

        Ok((state_idx, nfa.1))
    }

    fn non_capture_group(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_idx = self.nfa_vec.len();
        let end_state_idx = begin_state_idx + 1;
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::NonCapturingGroupStart(Some(nfa.0)), None)),
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));
        self.nfa_vec[nfa.1].transition(
            Some((Transition::NonCapturingGroupEnd(Some(end_state_idx)), None))
        );
        Ok((begin_state_idx, end_state_idx))
    }

    fn capture_group(&mut self, nfa: (usize, usize), group_num: u8) -> Result<(usize, usize), String> {
        let begin_state_idx = self.nfa_vec.len();
        let end_state_idx = begin_state_idx + 1;
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::CaptureGroupStart(group_num ,Some(nfa.0)), None)),
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));
        self.nfa_vec[nfa.1].transition(
            Some((Transition::CaptureGroupEnd(group_num, Some(end_state_idx)), None))
        );
        Ok((begin_state_idx, end_state_idx))
    }

    fn zero_or_more_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_index = self.nfa_vec.len();
        let end_state_index = begin_state_index + 1;
        //BEGIN state
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(nfa.0), None), Some(Transition::Epsilon(Some(end_state_index), None))))
        ));
        //END state
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));
        self.nfa_vec[nfa.1].change_state_type(StateType::Split);
        self.nfa_vec[nfa.1].transition(
            Some((Transition::Epsilon(Some(nfa.0), None), Some(Transition::Epsilon(Some(end_state_index), None))))
        );

        Ok((begin_state_index, end_state_index))
    }

    fn one_or_more_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        self.nfa_vec[nfa.1].change_state_type(StateType::Split);
        self.nfa_vec[nfa.1].connect_second_transition(nfa.0)?;
        Ok(nfa)
    }

    fn zero_or_one_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let nfa_start_state_index = self.nfa_vec.len();
        let nfa_end_state_index = nfa_start_state_index + 1;
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(nfa.0), None), Some(Transition::Epsilon(Some(nfa_end_state_index), None))))
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));
        self.nfa_vec[nfa.1].connect_first_transition(nfa_end_state_index)?;

        Ok((nfa_start_state_index, nfa_end_state_index))
    }

    fn repetition_nfa(&mut self, nfa: (usize, usize), m: usize, n: Option<usize>) -> Result<(usize, usize), String> {
        // In order for the last 'm' nfa to be made into a 'one or more'
        // we cannot construct it in the loop because we will have
        // no way to access it after, so we will remove it from the
        // loop and handle it in the match
        let mut m_nfa = nfa;
        if m != 0 {
            for _ in 0..m - 1 {
                let new_nfa = self.copy_nfa(nfa)?;
                m_nfa = self.concat_nfa(m_nfa, new_nfa)?;
                dbg!(&m_nfa);
            }
        }

        match n {
            Some(n) => {
                if m != 0 {
                    if m == 1 && n == 1 {return Ok(nfa)}
                    /*let new_nfa = self.copy_nfa(nfa)?;
                    m_nfa = self.concat_nfa(m_nfa, new_nfa)?;*/
                    dbg!(&m_nfa);
                    if m == n {return Ok(m_nfa)}
                }; // last 'm' nfa handled here (when n exists)
                if n == 0 { //checks if {0} below is how to deal with that
                    match self.nfa_vec[nfa.0].get_transitions() { //replace nfa with epsilon transition
                        Some(transitions) => {
                            match transitions.0 {
                                Transition::CaptureGroupStart(gs, _) => { //preserve group numbering
                                    self.nfa_vec[nfa.0].transition(
                                        Some((Transition::Epsilon(
                                            Some(nfa.1), Some(gs)
                                        ), None)
                                    ));
                                    return Ok(nfa)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    self.nfa_vec[nfa.0].transition(
                        Some((Transition::Epsilon(
                            Some(nfa.1), None), None)
                        ));
                    return Ok(nfa)
                }
                let remaining = n - m;
                let mut rem_nfa = nfa;
                for _ in 0..remaining {
                    rem_nfa = self.zero_or_one_nfa(rem_nfa)?;
                }
                Ok(self.concat_nfa(m_nfa, rem_nfa)?)
            }
            None => {
                let last_nfa = self.one_or_more_nfa(nfa)?; //last 'm' nfa handled here (when no n)
                Ok(self.concat_nfa(m_nfa, last_nfa)?)
            }
        }
    }
    fn alternate_nfa(&mut self, nfa_a: (usize, usize), nfa_b: (usize, usize)) -> Result<(usize, usize), String> {
        let nfa_start_state_idx = self.nfa_vec.len();
        let nfa_end_state_index = nfa_start_state_idx + 1;
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(nfa_a.0), None), Some(Transition::Epsilon(Some(nfa_b.0), None)))),
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));

        self.nfa_vec[nfa_a.1].connect_first_transition(nfa_end_state_index)?;
        self.nfa_vec[nfa_b.1].connect_first_transition(nfa_end_state_index)?;

        Ok((nfa_start_state_idx, nfa_end_state_index))
    }
    
    fn concat_nfa(&mut self, nfa_a: (usize, usize), nfa_b: (usize, usize)) -> Result<(usize, usize), String> {
        self.nfa_vec[nfa_a.1].connect_first_transition(nfa_b.0)?;
        Ok((nfa_a.0, nfa_b.1))
    }

    fn build_simple_nfa(&mut self, c: CharToMatch) -> Result<(usize, usize), String> {
        let state_index = self.nfa_vec.len();
        let end_state_index = state_index + 1;
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Literal(c, Some(end_state_index)), None))
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None, None), None))
        ));
        Ok((state_index, end_state_index))
    }

    fn copy_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let copy_start_state_idx = self.nfa_vec.len();

        let mut mappings: HashMap<usize, usize> = HashMap::new(); //maps old -> new nodes

        let start_state = self.nfa_vec[nfa.0].clone();
        let start_state_idx = nfa.0;
        self.deep_copy_dfs(start_state, start_state_idx, &mut mappings, nfa.1);

        let copy_end_state_idx = self.nfa_vec.len() - 1;

        for i in copy_start_state_idx .. copy_end_state_idx {
            let state = self.nfa_vec[i].clone();
            println!("{i}");
            let next_state_idx = self.get_transitions_next_state_idx(
                state.get_transitions().ok_or(String::from("Failed to get transition of copied nfa"))?);
            match state.get_state_type() {
                StateType::Literal => {
                    match next_state_idx {
                        Some(n) => {
                            let mapped_state = mappings[&n.0];
                            self.nfa_vec[i].connect_first_transition(mapped_state)?;
                        }
                        None => {}
                    }

                }
                StateType::Split => {
                    let first_mapped_state = mappings[&next_state_idx.unwrap().0];
                    let second_mapped_state = mappings[&next_state_idx.unwrap().1.unwrap()];
                    self.nfa_vec[i].connect_first_transition(first_mapped_state)?;
                    self.nfa_vec[i].connect_second_transition(second_mapped_state)?;
                }
                _ => {}
            }
        }

        Ok((copy_start_state_idx, copy_end_state_idx))
    }

    fn deep_copy_dfs(&mut self, state: State, state_idx: usize, mappings: &mut HashMap<usize, usize>, last_state: usize) {
        if mappings.contains_key(&state_idx) {return}
        let new_state_idx = self.nfa_vec.len();
        self.nfa_vec.push(State::new(
            state.get_state_type(),
            state.get_transitions()
        ));
        mappings.insert(state_idx, new_state_idx);

        if last_state == state_idx {return}

        let transitions = state.get_transitions();
        if transitions.is_none() { return }
        let next_transition_state_idx = self.get_transitions_next_state_idx(transitions.unwrap());
        if next_transition_state_idx.is_none() {return}

        let (first_transition, second_transition) = next_transition_state_idx.unwrap();

        self.deep_copy_dfs(self.nfa_vec[first_transition].clone(), first_transition, mappings, last_state);
        if second_transition.is_some() {
            self.deep_copy_dfs(self.nfa_vec[second_transition.unwrap()].clone(), second_transition.unwrap(), mappings, last_state);
        }
    }

    fn get_transitions_next_state_idx(&self, transitions: (Transition, Option<Transition>)) -> Option<(usize, Option<usize>)> {
        let first_transition: usize;
        let second_transition: Option<usize>;
        match transitions {
            (first, second) => {
                match first {
                    Transition::Epsilon(next, _) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::Literal(_, next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::AnchorStart(next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::AnchorEnd(next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::CaptureGroupStart(_, next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::CaptureGroupEnd(_, next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::NonCapturingGroupStart(next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                    Transition::NonCapturingGroupEnd(next) => {
                        match next {
                            Some(next) => {first_transition = next;}
                            None => {return None;}
                        }
                    }
                }
                match second {
                    Some(second) => {
                        match second {
                            Transition::Epsilon(next, _) => {
                                match next {
                                    Some(next) => {second_transition = Some(next)}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::Literal(_, next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::AnchorStart(next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::AnchorEnd(next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::CaptureGroupStart(_, next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::CaptureGroupEnd(_, next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::NonCapturingGroupStart(next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                            Transition::NonCapturingGroupEnd(next) => {
                                match next {
                                    Some(next) => {second_transition = Some(next);}
                                    None => {second_transition = None}
                                }
                            }
                        }
                    }
                    None => {second_transition = None}
                }
            }
        }
        Some((first_transition, second_transition))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::parse_ast::Parser;
    #[test]
    fn test() {
        /*let mut tokens = tokenize("(ab|c){2}").unwrap();
        let mut ast = Parser::new(tokens).parse_regex().unwrap();
        let nfa_1 = BuilderNFA::new().compile(&ast).unwrap();
        tokens = tokenize("(ab|c)(ab|c)").unwrap();
        ast = Parser::new(tokens).parse_regex().unwrap();
        let nfa_2 = BuilderNFA::new().compile(&ast).unwrap();
        debug_assert_eq!(nfa_1, nfa_2);*/
    }
}