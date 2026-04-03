use crate::types::nfa_types::CharToMatch;
use crate::types::token_types::{CharClassType, CharSetType};

pub fn matcher(c: char, m: &CharToMatch) -> bool {
    match m {
        CharToMatch::Literal(ch) => c == *ch,
        CharToMatch::Any => true,
        CharToMatch::CharacterClass(char_class) => {match_character_class(c, char_class)}
        CharToMatch::CharacterSet(negated, char_set) => {
            let mut result: bool = false;
            for c_class in char_set.iter() {
                if match_character_class(c, c_class) {result = true; break;}
            }
            if negated.eq(&CharSetType::Negated) {
                return !result;
            }
            result
        }
    }
}

fn match_character_class(c: char, char_class: &CharClassType) -> bool {
    match char_class {
        CharClassType::Any => true,
        CharClassType::Literal(ch) => c == *ch,
        CharClassType::Word => {c.is_ascii_alphanumeric() || c == '_'}
        CharClassType::NonWord => {!c.is_ascii_alphanumeric() && c != '_'}
        CharClassType::Digit => {c.is_ascii_digit()}
        CharClassType::NonDigit => {!c.is_ascii_digit()}
        CharClassType::Whitespace => {c.is_ascii_whitespace()}
        CharClassType::NonWhitespace => {!c.is_ascii_whitespace()}
        CharClassType::Range(first, second) => {c >= *first && c <= *second}
    }
}