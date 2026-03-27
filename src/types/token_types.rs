#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Epsilon,
    Literal(char),
    WildCard,

    ZeroOrMore, // *
    OneOrMore, // +
    ZeroOrOne, // ?
    Repetition(usize, Option<usize>), // {n, m}

    Alternation, // |
    LeftParen(GroupType, Option<u8>), // (
    RightParen, // )

    StartAnchor, // ^
    EndAnchor, // $

    //BackReference(u8), // \(1-9)
    CharacterClass(CharClassType),
    CharacterSet(CharSetType, Vec<CharClassType>)

}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {
    Capturing,
    NonCapturing,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CharSetType {
    NonNegated,
    Negated
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CharClassType {
    Word, //w
    NonWord, //W
    Digit, //d
    NonDigit, //D
    Whitespace, //s
    NonWhitespace, //S
    Literal(char),
    Range(char, char),
    Any // match anything 
}