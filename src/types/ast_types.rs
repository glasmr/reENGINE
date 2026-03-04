use crate::types::token_types::CharClassType;

#[derive(Debug)]
pub enum NodeAST {
    Literal(char),
    CharacterClass(CharClassType),
    Any,
    AnchorStart(Box<NodeAST>),
    AnchorEnd(Box<NodeAST>),
    CaptureGroup(Box<NodeAST>, u8),
    Quantifier(Box<NodeAST>, QuantifierType),
    Concatenation(Box<NodeAST>, Box<NodeAST>),
    Alternation(Box<NodeAST>, Box<NodeAST>)
}

#[derive(Debug)]
pub enum QuantifierType {
    ZeroOrMore, // *
    OneOrMore, // +
    ZeroOrOne, // ?
    Repetition(usize, Option<usize>), // {m, n}
}