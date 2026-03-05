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
        let (start_state, end_state) = self.complete_nfa((start, end))?;
        let states = self.nfa_vec.to_vec();
        Ok(NFA{
            states,
            start_state, 
            end_state
        })
    }

    fn walk_tree(&mut self, ast: &NodeAST) -> Result<(usize, usize), String> {
        match ast {
            NodeAST::Literal(char) => {self.build_simple_nfa(CharToMatch::Literal(*char))}

            NodeAST::CharacterClass(char_class_type) => {self.build_simple_nfa(CharToMatch::CharacterClass(*char_class_type))}

            NodeAST::Any => {self.build_simple_nfa(CharToMatch::Any)}

            NodeAST::AnchorStart(next) => {unimplemented!()}

            NodeAST::AnchorEnd(next) => {unimplemented!()}

            NodeAST::CaptureGroup(next, grp_n) => {unimplemented!()}

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

    fn zero_or_more_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_index = self.nfa_vec.len();
        let end_state_index = begin_state_index + 1;
        //BEGIN state
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(nfa.0)), Some(Transition::Epsilon(Some(end_state_index)))))
        ));
        //END state
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None), None))
        ));
        self.nfa_vec[nfa.1].change_state_type(StateType::Split);
        self.nfa_vec[nfa.1].transition(
            Some((Transition::Epsilon(Some(nfa.0)), Some(Transition::Epsilon(Some(end_state_index)))))
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
            Some((Transition::Epsilon(Some(nfa.0)), Some(Transition::Epsilon(Some(nfa_end_state_index)))))
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None), None))
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
        for _ in 0..m - 1 {
            m_nfa = self.concat_nfa(m_nfa, nfa)?
        }
        match n {
            Some(n) => {
                m_nfa = self.concat_nfa(m_nfa, nfa)?; // last 'm' nfa handled here (when n exists)
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
            Some((Transition::Epsilon(Some(nfa_a.0)), Some(Transition::Epsilon(Some(nfa_b.0))))),
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None), None))
        ));

        self.nfa_vec[nfa_a.0].connect_first_transition(nfa_end_state_index)?;
        self.nfa_vec[nfa_b.0].connect_first_transition(nfa_end_state_index)?;

        Ok((nfa_start_state_idx, nfa_end_state_index))
    }
    
    /// Connects the dangling ends of state A (first) to the input of state B (second)
    fn concat_nfa(&mut self, nfa_a: (usize, usize), nfa_b: (usize, usize)) -> Result<(usize, usize), String> {
        self.nfa_vec[nfa_a.0].connect_first_transition(nfa_b.0)?;
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
            Some((Transition::Epsilon(None), None))
        ));
        Ok((state_index, end_state_index))
    }

    // Adds the final match state to the nfa and returns the index
    // of the start, and of the end.
    fn complete_nfa(&mut self, nfa: (usize, usize)) -> Result<(usize, usize), String> {
        let matched_state_index = self.nfa_vec.len();
        self.nfa_vec.push(State::new(
            StateType::Match,
            None
        ));
        Ok(self.concat_nfa(nfa, (matched_state_index, matched_state_index))?)
    }
    
}