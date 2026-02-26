//! Generates a Vec<Token> of tokens from the supplied str.
//! Tokens are later parsed by a separate module.
//! ## The tokenizer currently supports ##
//! Differential capturing and non-capturing groups,
//! character escapes, octal escapes, hex escapes, Unicode escapes,
//! start ^ and end $ anchors,
//! elementary regex operators (*, +, ?, |),
//! Character classes (\s, \S, \w, \W, \d, \D),
//! back references.
//! ## The tokenizer does NOT (yet) support ##
//! Charter sets '[abc]',
//! Lookaround,
//! word boundaries,
//! Named capture groups,
//! quantifiers '{1, 3}'

use crate::types::{CharClassType, GroupType, Token};

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let len = input.len();
    let mut i: usize = 0;
    while i < len {
        let ch = input.chars().nth(i).unwrap();

        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {tokens.push(Token::Literal(ch))}
            '.' => {tokens.push(Token::Any)}
            '*' => {tokens.push(Token::ZeroOrMore)}
            '+' => {tokens.push(Token::OneOrMore)}
            '?' => {tokens.push(Token::ZeroOrOne)}
            '|' => {tokens.push(Token::Alternation)}
            '(' => { //check if group is capturing or not
                if i + 2 < len {
                    if input.chars().nth(i + 1).unwrap() == '?' &&
                        input.chars().nth(i + 2).unwrap() == ':' {
                        tokens.push(Token::LeftParen(GroupType::NonCapturing));
                        i += 3;
                        continue;
                    }
                }
                tokens.push(Token::LeftParen(GroupType::Capturing))
            }
            ')' => {tokens.push(Token::RightParen)}
            '^' => {tokens.push(Token::StartAnchor)}
            '$' => {tokens.push(Token::EndAnchor)}
            '\\' => {
                i += 1;
                let next_char = input.chars().nth(i).unwrap();
                match next_char {
                    't' => {tokens.push(Token::Literal('\t'))} //tab
                    'n' => {tokens.push(Token::Literal('\n'))} // newline
                    'v' => {tokens.push(Token::Literal(11 as char))} //vertical tab \v
                    'f' => {tokens.push(Token::Literal(12 as char))} // form feed \f
                    'r' => {tokens.push(Token::Literal('\r'))} //carriage return
                    'w' => {tokens.push(Token::CharacterClass(CharClassType::Word))}
                    'W' => {tokens.push(Token::CharacterClass(CharClassType::NonWord))}
                    'd' => {tokens.push(Token::CharacterClass(CharClassType::Digit))}
                    'D' => {tokens.push(Token::CharacterClass(CharClassType::NonDigit))}
                    's' => {tokens.push(Token::CharacterClass(CharClassType::Whitespace))}
                    'S' => {tokens.push(Token::CharacterClass(CharClassType::NonWhitespace))}
                    '0'..='9' => {
                        //Octal escapes do not have a flag, only 3 number digits
                        if i + 2 < len {
                            let sec_digi = input.chars().nth(i + 1).unwrap();
                            let third_digi = input.chars().nth(i + 2).unwrap();
                            if next_char.is_digit(8) &&
                                sec_digi.is_digit(8) &&
                                    third_digi.is_digit(8) {
                                let octal_arr: String = [next_char, sec_digi, third_digi].into_iter().collect();
                                let octal = u32::from_str_radix(&octal_arr, 8).unwrap();
                                //most regex only supports octal up to 255, so if it's higher, we default to 255
                                let mut octal_char = 255;
                                if octal < 255 {
                                    octal_char = octal;
                                }
                                tokens.push(Token::Literal(char::from_u32(octal_char).unwrap()));
                                i += 3;
                                continue;
                            }
                        }
                        if next_char == '0' {tokens.push(Token::Literal('\0'))}
                        else {
                            let b_ref = u8::from_str_radix(&next_char.to_string(), 10).unwrap();
                            tokens.push(Token::BackReference(b_ref));
                        }
                    }
                    'x' => {
                        if i + 2 < len {
                            let first_digi = input.chars().nth(i + 1).unwrap();
                            let sec_digi = input.chars().nth(i + 2).unwrap();
                            if first_digi.is_digit(16) &&
                                sec_digi.is_digit(16) {
                                let hex: String = vec![first_digi, sec_digi].into_iter().collect();
                                let hax_val = u8::from_str_radix(&hex, 16).unwrap();
                                tokens.push(Token::Literal(hax_val as char));
                            }
                            i += 3;
                            continue;
                        } else {tokens.push(Token::Literal(next_char))}
                    }
                    'u' => {
                        if i + 4 < len {
                            let first_digi = input.chars().nth(i + 1).unwrap();
                            let sec_digi = input.chars().nth(i + 2).unwrap();
                            let third_digi = input.chars().nth(i + 3).unwrap();
                            let four_digi = input.chars().nth(i + 4).unwrap();
                            if first_digi.is_digit(16) &&
                                sec_digi.is_digit(16) &&
                                third_digi.is_digit(16) &&
                                four_digi.is_digit(16) {
                                let digits = u32::from_be_bytes([first_digi as u8, sec_digi as u8, third_digi as u8, four_digi as u8]);
                                let unicode_val = char::from_u32(digits).unwrap();
                                tokens.push(Token::Literal(unicode_val));
                                i += 4;
                                continue;
                            }
                        }
                        else {tokens.push(Token::Literal(next_char))}
                    }
                    _ => {tokens.push(Token::Literal(next_char))}
                }

            }
            _ => {tokens.push(Token::Literal(ch))} //Any unknown char can just be added as a literal
        }
        i += 1;
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_literals() {
        let tokens = tokenize("abc");
        let expected: Vec<Token> = vec![Token::Literal('a'), Token::Literal('b'), Token::Literal('c')];
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_escaped_special_chars() {
        let tokens = tokenize("\\*\\?\\+\\(\\)\\|\\.\\^\\$");
        let expected: Vec<Token> = vec![Token::Literal('*'), Token::Literal('?'), Token::Literal('+'),
                                        Token::Literal('('), Token::Literal(')'), Token::Literal('|'),
                                        Token::Literal('.'), Token::Literal('^'), Token::Literal('$')];
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_groups() {
        let tokens = tokenize("(abc)(?:abc)");
        let expected: Vec<Token> = vec![Token::LeftParen(GroupType::Capturing), Token::Literal('a'), Token::Literal('b'),
                                        Token::Literal('c'), Token::RightParen, Token::LeftParen(GroupType::NonCapturing),
                                        Token::Literal('a'), Token::Literal('b'), Token::Literal('c'), Token::RightParen];
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_octal_escapes() {
        let tokens = tokenize("\\012");
        let expected: Vec<Token> = vec![Token::Literal(10 as char)];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\02");
        let expected: Vec<Token> = vec![Token::Literal('\0'), Token::Literal('2')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\070");
        let expected: Vec<Token> = vec![Token::Literal(56 as char)];
        assert_eq!(tokens, expected);
        let tokens = tokenize("a\\456a");
        let expected: Vec<Token> = vec![Token::Literal('a'), Token::Literal(255 as char), Token::Literal('a')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\900");
        let expected: Vec<Token> = vec![Token::BackReference(9), Token::Literal('0'), Token::Literal('0')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\080");
        let expected: Vec<Token> = vec![Token::Literal('\0'), Token::Literal('8'), Token::Literal('0')];
        assert_eq!(tokens, expected);
    }
}