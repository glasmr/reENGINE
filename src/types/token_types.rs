#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(char),
    Any,

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

    StartCharSet(CharSetType),
    EndCharSet,
    CharacterClass(CharClassType)

}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {
    Capturing,
    NonCapturing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharSetType {
    NonNegated,
    Negated
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CharClassType {
    Word,
    NonWord,
    Digit,
    NonDigit,
    Whitespace,
    NonWhitespace,
}