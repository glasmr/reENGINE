use crate::types::token_types::{CharClassType, CharSetType};

#[derive(Debug, PartialEq)]
pub enum NodeAST {
    Literal(char),
    CharacterClass(CharClassType),
    CharacterSet(CharSetType, Vec<CharClassType>),
    WordBoundary,
    NonWordBoundary,
    Any,
    AnchorStart(Box<NodeAST>),
    AnchorEnd(Box<NodeAST>),
    Group(Box<NodeAST>),
    Quantifier(Box<NodeAST>, QuantifierType),
    Concatenation(Box<NodeAST>, Box<NodeAST>),
    Alternation(Box<NodeAST>, Box<NodeAST>)
}

#[derive(Debug, PartialEq)]
pub enum QuantifierType {
    ZeroOrMore, // *
    OneOrMore, // +
    ZeroOrOne, // ?
    Repetition(usize, Option<usize>), // {m, n}
}