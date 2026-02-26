
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(char),
    Any,

    ZeroOrMore, // *
    OneOrMore, // +
    ZeroOrOne, // ?

    Alternation, // |
    LeftParen(GroupType), // (
    RightParen, // )

    StartAnchor, // ^
    EndAnchor, // $

    BackReference(u8), // \(1-9)

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

#[derive(Debug, Clone, PartialEq)]
pub enum CharClassType {
    Word,
    NonWord,
    Digit,
    NonDigit,
    Whitespace,
    NonWhitespace,
}