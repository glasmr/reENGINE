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
        unimplemented!()
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
                let state = self.walk_tree(next)?;
                match quantifier {
                    QuantifierType::ZeroOrMore => {}
                    QuantifierType::OneOrMore => {}
                    QuantifierType::ZeroOrOne => {}
                    QuantifierType::Repetition(m, n) => {}
                }
                unimplemented!()
            }
            NodeAST::Concatenation(next_a, next_b) => {
                let state_a = self.walk_tree(next_a)?;
                let state_b = self.walk_tree(next_b)?;
                self.concat_nfa(state_a, state_b)
            }
            NodeAST::Alternation(next_a, next_b) => {
                let state_a = self.walk_tree(next_a)?;
                let state_b = self.walk_tree(next_b)?;

                self.alternate_nfa(state_a, state_b)
            }
        }
        
    }

    fn zero_or_more_nfa(&mut self, state: (usize, usize)) -> Result<(usize, usize), String> {
        let begin_state_index = self.nfa_vec.len();
        let end_state_index = begin_state_index + 1;
        //BEGIN state
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(state.0)), Some(Transition::Epsilon(Some(end_state_index)))))
        ));
        //END state
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None), None))
        ));
        self.nfa_vec[state.1].change_state_type(StateType::Split);
        self.nfa_vec[state.1].transition(
            Some((Transition::Epsilon(Some(state.0)), Some(Transition::Epsilon(Some(end_state_index)))))
        );

        Ok((begin_state_index, end_state_index))
    }

    fn one_or_more_nfa(&mut self, state: (usize, usize)) -> Result<(usize, usize), String> {
        unimplemented!()
    }

    fn zero_or_one_nfa(&mut self, state: (usize, usize)) -> Result<(usize, usize), String> {
        unimplemented!()
    }

    fn repetition_nfa(&mut self, state: (usize, usize), m: usize, n: Option<usize>) -> Result<(usize, usize), String> {
        unimplemented!()
    }
    fn alternate_nfa(&mut self, state_a: (usize, usize), state_b: (usize, usize)) -> Result<(usize, usize), String> {
        let state_index = self.nfa_vec.len();
        let end_state_index = state_index + 1;
        self.nfa_vec.push(State::new(
            StateType::Split,
            Some((Transition::Epsilon(Some(state_a.0)), Some(Transition::Epsilon(Some(state_b.0))))),
        ));
        self.nfa_vec.push(State::new(
            StateType::Literal,
            Some((Transition::Epsilon(None), None))
        ));

        self.nfa_vec[state_a.0].connect_next_state(end_state_index)?;
        self.nfa_vec[state_b.0].connect_next_state(end_state_index)?;

        Ok((state_index, end_state_index))
    }
    
    /// Connects the dangling ends of state A (first) to the input of state B (second)
    fn concat_nfa(&mut self, state_a: (usize, usize), state_b: (usize, usize)) -> Result<(usize, usize), String> {
        self.nfa_vec[state_a.0].connect_next_state(state_b.0)?;
        Ok((state_a.0, state_b.1))
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
    fn complete_nfa(&mut self, nfa: (usize, Vec<usize>)) -> Result<(usize, usize), String> {unimplemented!()}
    
}