//! Generates a Vec<Token> of tokens from the supplied str.
//! Tokens are later parsed by a separate module.
//! # The tokenizer currently supports #
//! Differential capturing and non-capturing groups,
//! character escapes, octal escapes, hex escapes, Unicode escapes,
//! start ^ and end $ anchors,
//! elementary regex operators (*, +, ?, |),
//! Character classes (\s, \S, \w, \W, \d, \D),
//! Charter sets '[abc]'
//! quantifiers '{1, 3}'
//! # The tokenizer does NOT (yet) support #
//! lazy quantifiers
//! back references
//! Lookaround
//! word boundaries,
//! Named capture groups


use std::collections::HashSet;
use std::str::FromStr;
use crate::types::token_types::{CharClassType, Token, CharSetType};


pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let len = input.len();
    if len == 0 {tokens.push(Token::Epsilon); return Ok(tokens);}
    
    let mut i: usize = 0;
    while i < len {
        let ch = input.chars().nth(i).unwrap();
        if i == len - 1 { //check for dangling backslash
            if ch == '\\' {return Err(String::from("Error: Dangling backslash!"))}
        }

        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {tokens.push(Token::Literal(ch))}
            '.' => {tokens.push(Token::WildCard)}
            '*' => {tokens.push(Token::ZeroOrMore)}
            '+' => {tokens.push(Token::OneOrMore)}
            '?' => {tokens.push(Token::ZeroOrOne)}
            '|' => {tokens.push(Token::Alternation)}
            '(' => {tokens.push(Token::LeftParen)}
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
                    'b' => {tokens.push(Token::WordBoundary)}
                    'B' => {tokens.push(Token::NonWordBoundary)}
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
                            /*let b_ref = u8::from_str_radix(&next_char.to_string(), 10).unwrap(); //Using NFA, no backref support
                            tokens.push(Token::BackReference(b_ref));*/
                            tokens.push(Token::Literal(next_char));
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
                                let bytes_str = [first_digi, sec_digi, third_digi, four_digi].iter().collect::<String>();
                                let digits = u32::from_str_radix(bytes_str.as_str(), 16).unwrap();
                                let unicode_val = char::from_u32(digits).unwrap();
                                tokens.push(Token::Literal(unicode_val));
                                i += 5;
                                continue;
                            }
                        }
                        else {tokens.push(Token::Literal(next_char))}
                    }
                    _ => {tokens.push(Token::Literal(next_char))}
                }

            }
            '{' => {
                let q_pos = i;
                let mut quantifier_str = String::new();
                i += 1; //consume the '{'
                let mut c = input.chars().nth(i).unwrap();
                while c != '}' {
                    if i == len - 1 {return Err(format!("Error at Quantifier! Position {q_pos}: No closing }}!"))}
                    quantifier_str.push(c);
                    i += 1;
                    c = input.chars().nth(i).unwrap();
                }
                let token = match parse_quantifier(quantifier_str) {
                    Ok(token) => token,
                    Err(e) => {
                        return Err(format!("Error at Quantifier position {q_pos}, {}", e));
                    }
                };
                tokens.push(token);
            }
            '[' => {
                let cc_pos = i;
                let mut cc_string = String::new();
                i += 1; //consume the '['
                let mut c = input.chars().nth(i).unwrap();
                if c == ']' || c == '^' { // A close bracket directly after opening is literal
                    if c == '^' {
                        if input.chars().nth(i + 1).unwrap() == ']' {
                            cc_string.push(c);
                            cc_string.push(']');
                            i += 2;
                            c = input.chars().nth(i).unwrap();
                        }
                    } else {
                        cc_string.push(c);
                        i += 1;
                        c = input.chars().nth(i).unwrap();
                    }
                }
                while c != ']' {
                    if i == len - 1 {return Err(format!("Error at Character Class at Position {cc_pos}, no closing ']'"))}
                    if c == '\\' {
                        if input.chars().nth(i + 1).unwrap() == ']' { //escaped ']'
                            cc_string.push(']');
                            i += 2; //consume both '\' and ']'
                            c = input.chars().nth(i).unwrap();
                            continue;
                        }
                    }
                    cc_string.push(c);
                    i += 1;
                    c = input.chars().nth(i).unwrap();
                }
                let token = match parse_character_set(cc_string) {
                    Ok(token) => token,
                    Err(e) => {return Err(format!("Error at Character Class at Position {cc_pos}, {}", e));}
                };
                tokens.push(token);
            }
            _ => {tokens.push(Token::Literal(ch))} //Any unknown char can just be added as a literal
        }
        i += 1;
    }
    Ok(tokens)
}

fn parse_quantifier(q_string: String) -> Result<Token, String> {
    if q_string.len() == 0 {return Err(String::from("Error: empty quantifier!"))}
    if !q_string.contains(',') {
        match usize::from_str(q_string.as_str()) {
            Ok(value) => {
                Ok(Token::Repetition(value, Some(value)))
            },
            Err(_) => Err(format!("Error: invalid quantifier value! '{q_string}'"))
        }
    } else {
        let split_quantifier_str = match q_string.split_once(',') {
            Some(split) => split,
            None => return Err(format!("Error: malformed quantifier! '{q_string}'"))
        };
        let value_1 = match usize::from_str(split_quantifier_str.0.trim()) {
            Ok(value) => value,
            Err(_) => return Err(format!("Error: error parsing first quantifier value '{}'", split_quantifier_str.0))
        };
        if split_quantifier_str.1.trim().len() == 0 {
            return Ok(Token::Repetition(value_1, None));
        }
        let value_2 = match usize::from_str(split_quantifier_str.1.trim()) {
            Ok(value) => value,
            Err(_) => return Err(format!("Error: error parsing second quantifier value '{}'", split_quantifier_str.0))
        };
        if value_1 >= value_2 {
            return Err(String::from("Error: invalid quantifier value! First value must be less than the second value!"))
        }
        Ok(Token::Repetition(value_1, Some(value_2)))
    }
}

fn parse_character_set(cs_string: String) -> Result<Token, String> {
    //This function can use some serious cleanup
    let mut char_class_vec: Vec<CharClassType> = Vec::new();
    let mut len = cs_string.len();
    if len == 0 {return Err(String::from("Error: empty character set!"))}
    let chars = cs_string.chars().collect::<Vec<char>>();

    let mut negate_class = CharSetType::NonNegated;
    let mut str_pos: usize = 0;
    if chars[str_pos] == '^' { //check if it is negated set
        if len == 1 {return Ok(Token::CharacterSet(negate_class, vec![CharClassType::Any]))}
        negate_class = CharSetType::Negated;
        str_pos += 1;
    }

    if chars[str_pos] == '-' || chars[len - 1] == '-' { //check both ends for literal '-'
        char_class_vec.push(CharClassType::Literal('-'));
        if chars[str_pos] == '-' {
            str_pos += 1;
        }
        if chars[len - 1] == '-' {
            len -= 1;
        }
    }

    let mut indices_to_remove: Vec<usize> = Vec::new();
    let mut ranges: Vec<(usize, usize)> = Vec::new(); //indexes to range values

    let trimmed_str = &cs_string[str_pos..len];
    let trimmed_str_char_idx = trimmed_str.char_indices().collect::<Vec<(usize, char)>>();
    for i in 0..trimmed_str_char_idx.len() {
        let ci = trimmed_str_char_idx[i];
        if ci.1 == '-' {
            if trimmed_str_char_idx[i - 1].1 == '\\' {
                continue;
            }
            if i > 1 {
                if trimmed_str_char_idx[i - 2].1 == '\\'{
                    return Err(String::from("Error: malformed character set! Cannot range with escaped character!"));
                }
            }
            if trimmed_str_char_idx[i + 1].1 == '\\' {
                return Err(String::from("Error: malformed character set! Cannot range with escaped character!"));
            }
            ranges.push((ci.0 - 1, ci.0 + 1));
            indices_to_remove.append(&mut vec![ci.0 - 1, ci.0, ci.0 + 1])
        }
    }

    for range in ranges {
        let trimmed_chars = trimmed_str.chars().collect::<Vec<char>>();
        let lower = trimmed_chars[range.0];
        let upper = trimmed_chars[range.1];
        if lower as u8 > upper as u8 {return Err(format!("Invalid range! {lower} - {upper}"))}
        char_class_vec.push(CharClassType::Range(lower, upper))
    }

    let indices_to_remove_set = HashSet::<usize>::from_iter(indices_to_remove);
    let mut remaining_vals: Vec<char> = Vec::new();
    for val in trimmed_str.char_indices() {
        if indices_to_remove_set.contains(&val.0) {continue}
        remaining_vals.push(val.1);
    }



    let mut remaining_vals_iter = remaining_vals.iter();
    while let Some(remaining) = remaining_vals_iter.next() {
        if *remaining == '\\' {
            let next_val = remaining_vals_iter.next().unwrap();
            match next_val {
                'w' => {char_class_vec.push(CharClassType::Word)},
                'W' => {char_class_vec.push(CharClassType::NonWord)}
                'd' => {char_class_vec.push(CharClassType::Digit)}
                'D' => {char_class_vec.push(CharClassType::NonDigit)}
                's' => {char_class_vec.push(CharClassType::Whitespace)}
                'S' => {char_class_vec.push(CharClassType::NonWhitespace)}
                _ => {char_class_vec.push(CharClassType::Literal(*next_val))}
            }
            continue;
        }
        char_class_vec.push(CharClassType::Literal(*remaining));
    }

    Ok(Token::CharacterSet(negate_class, char_class_vec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_literals() {
        let tokens = tokenize("abc").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('a'), Token::Literal('b'), Token::Literal('c')];
        assert_eq!(tokens, expected);

    }
    #[test]
    fn test_escaped_special_chars() {
        let tokens = tokenize("\\*\\?\\+\\(\\)\\|\\.\\^\\$").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('*'), Token::Literal('?'), Token::Literal('+'),
                                        Token::Literal('('), Token::Literal(')'), Token::Literal('|'),
                                        Token::Literal('.'), Token::Literal('^'), Token::Literal('$')];
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_general_escapes() {}
    #[test]
    fn test_octal_escapes() {
        let tokens = tokenize("\\012").unwrap();
        let expected: Vec<Token> = vec![Token::Literal(10 as char)];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\02").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('\0'), Token::Literal('2')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\070").unwrap();
        let expected: Vec<Token> = vec![Token::Literal(56 as char)];
        assert_eq!(tokens, expected);
        let tokens = tokenize("a\\456a").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('a'), Token::Literal(255 as char), Token::Literal('a')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\900").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('9'), Token::Literal('0'), Token::Literal('0')];
        assert_eq!(tokens, expected);
        let tokens = tokenize("\\080").unwrap();
        let expected: Vec<Token> = vec![Token::Literal('\0'), Token::Literal('8'), Token::Literal('0')];
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_hex_escapes() {
        let mut input_string = "\\x4e"; // hex for N
        let mut token = tokenize(input_string);
        let mut expected_token: Result<Vec<Token>, String> = Ok(vec![Token::Literal('N')]);
        assert_eq!(token, expected_token);

        input_string = "\\x00"; //VALID
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('\0')]);
        assert_eq!(token, expected_token);

        input_string = "a\\x4eb"; //VALID
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('a'), Token::Literal('N'), Token::Literal('b')]);
        assert_eq!(token, expected_token);
    }
    #[test]
    fn test_unicode_escapes() {
        let mut input_string = "\\u00a9"; // hex for N
        let mut token = tokenize(input_string);
        let mut expected_token: Result<Vec<Token>, String> = Ok(vec![Token::Literal('©')]);
        assert_eq!(token, expected_token);

        input_string = "\\u20AC"; //VALID
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('€')]);
        assert_eq!(token, expected_token);

        input_string = "a\\u20ACb"; //VALID
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('a'), Token::Literal('€'), Token::Literal('b')]);
        assert_eq!(token, expected_token);
    }
    #[test]
    fn test_char_set() {
        let mut input_str = "[a-zA-Z]";
        let mut tokens = tokenize(input_str).unwrap();
        let mut expected: Vec<Token> = vec![Token::CharacterSet(CharSetType::NonNegated, vec![CharClassType::Range('a', 'z'), CharClassType::Range('A', 'Z')])];
        assert_eq!(tokens, expected);

        input_str = "[]abc]"; //literal ]
        tokens = tokenize(input_str).unwrap();
        expected = vec![Token::CharacterSet(CharSetType::NonNegated,
                vec![CharClassType::Literal(']'), CharClassType::Literal('a'), CharClassType::Literal('b'), CharClassType::Literal('c')])];
        assert_eq!(tokens, expected);

        input_str = "[\\d]"; //escaped
        tokens = tokenize(input_str).unwrap();
        expected = vec![Token::CharacterSet(CharSetType::NonNegated,
                    vec![CharClassType::Digit])];
        assert_eq!(tokens, expected);

        input_str = "[\\]\\[]"; //escaped
        tokens = tokenize(input_str).unwrap();
        expected = vec![Token::CharacterSet(CharSetType::NonNegated,
                    vec![CharClassType::Literal(']'), CharClassType::Literal('[')])];
        assert_eq!(tokens, expected);

        input_str = "a[z]bc"; //escaped
        tokens = tokenize(input_str).unwrap();
        expected = vec![Token::Literal('a'),
                        Token::CharacterSet(CharSetType::NonNegated,vec![CharClassType::Literal('z')]),
                        Token::Literal('b'),
                        Token::Literal('c')];
        assert_eq!(tokens, expected);

        input_str = "[abc"; //unclosed ] "Error at Character Class at Position {cc_pos}, no closing ']'"
        let tokens = tokenize(input_str);
        let expected = Err(String::from("Error at Character Class at Position 0, no closing ']'"));
        assert_eq!(tokens, expected);
    }
    #[test]
    fn test_parse_quantifier() {
        let mut q_test_str = String::from("1,2"); // range 1 - 2
        let mut q_token = parse_quantifier(q_test_str);
        let mut actual_token: Result<Token, String> = Ok(Token::Repetition(1, Some(2)));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("2"); //exactly 2
        q_token = parse_quantifier(q_test_str);
        actual_token = Ok(Token::Repetition(2, Some(2)));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("2,"); //2 or more
        q_token = parse_quantifier(q_test_str);
        actual_token = Ok(Token::Repetition(2, None));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("2,1"); //invalid - err
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: invalid quantifier value! First value must be less than the second value!"));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("4784279837"); //large num
        q_token = parse_quantifier(q_test_str);
        actual_token = Ok(Token::Repetition(4784279837, Some(4784279837)));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("-2"); //err - negative
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: invalid quantifier value! '-2'"));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("0,7"); //0 - 7
        q_token = parse_quantifier(q_test_str);
        actual_token = Ok(Token::Repetition(0, Some(7)));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from(",7"); //err, no starting number
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: error parsing first quantifier value ''"));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("-2, 1"); //err - negative
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: error parsing first quantifier value '-2'"));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from(""); //empty "Error: empty quantifier!"
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: empty quantifier!"));
        assert_eq!(q_token, actual_token);

        q_test_str = String::from("abc"); //chars
        q_token = parse_quantifier(q_test_str);
        actual_token = Err(String::from("Error: invalid quantifier value! 'abc'"));
        assert_eq!(q_token, actual_token);
    }
    #[test]
    fn test_parse_character_set() {
        let mut char_set_str = String::from("abc");
        let mut char_set_token = parse_character_set(char_set_str);
        let mut expected_token: Result<Token, String> = Ok(Token::CharacterSet(CharSetType::NonNegated,
        vec![CharClassType::Literal('a'), CharClassType::Literal('b'), CharClassType::Literal('c')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-z"); //valid a-z
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
            vec![CharClassType::Range('a', 'z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-Z"); //err, start code larger than end
        char_set_token = parse_character_set(char_set_str);
        expected_token = Err(String::from("Invalid range! a - Z"));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-zA-Z"); //valid a-z, A-Z
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Range('a', 'z'), CharClassType::Range('A', 'Z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("-az"); //literal -, a, z
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('-'), CharClassType::Literal('a'), CharClassType::Literal('z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("az-"); //a, z, literal -
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('-'), CharClassType::Literal('a'), CharClassType::Literal('z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("-a-z-"); //lit -, a-z, lit -
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('-'), CharClassType::Range('a', 'z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("^abc"); //NOT abc
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::Negated,
                                                vec![CharClassType::Literal('a'), CharClassType::Literal('b'), CharClassType::Literal('c')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a^b"); //a, ^, b
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('a'), CharClassType::Literal('^'), CharClassType::Literal('b')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("\n"); // \n
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('\n')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("]abc"); //literal ], a, b, c
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                    vec![CharClassType::Literal(']'), CharClassType::Literal('a'), CharClassType::Literal('b'), CharClassType::Literal('c')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-z0-9_"); //a-z, 0-9, _
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Range('a', 'z'), CharClassType::Range('0', '9'), CharClassType::Literal('_')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a\\-z"); //a, lit -, z
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Literal('a'), CharClassType::Literal('-'), CharClassType::Literal('z')]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("\\d"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Digit]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("^\\d"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::Negated,
                                                vec![CharClassType::Digit]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-z\\W"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Range('a', 'z'), CharClassType::NonWord]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("\\sa-z"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Ok(Token::CharacterSet(CharSetType::NonNegated,
                                                vec![CharClassType::Range('a', 'z'), CharClassType::Whitespace]));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("\\s-\\w"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Err(String::from("Error: malformed character set! Cannot range with escaped character!"));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("a-\\w"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Err(String::from("Error: malformed character set! Cannot range with escaped character!"));
        assert_eq!(char_set_token, expected_token);

        char_set_str = String::from("\\s-a"); //
        char_set_token = parse_character_set(char_set_str);
        expected_token = Err(String::from("Error: malformed character set! Cannot range with escaped character!"));
        assert_eq!(char_set_token, expected_token);
    }
    #[test]
    fn test_random_things() {
        let mut input_string = "abc\\"; //trailing backslash
        let mut token = tokenize(input_string);
        let mut expected_token: Result<Vec<Token>, String> = Err(String::from("Error: Dangling backslash!"));
        assert_eq!(token, expected_token);

        input_string = "\\\\"; //trailing backslash
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('\\')]);
        assert_eq!(token, expected_token);

        input_string = ".*"; //VALID
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::WildCard, Token::ZeroOrMore]);
        assert_eq!(token, expected_token);

        input_string = "\\."; //trailing backslash
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Literal('.')]);
        assert_eq!(token, expected_token);

        input_string = ""; //trailing backslash
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::Epsilon]);
        assert_eq!(token, expected_token);

        input_string = "^a$"; //anchors
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::StartAnchor, Token::Literal('a'), Token::EndAnchor]);
        assert_eq!(token, expected_token);

        input_string = "^a|^b"; //trailing backslash
        token = tokenize(input_string);
        expected_token = Ok(vec![Token::StartAnchor, Token::Literal('a'), Token::Alternation, Token::StartAnchor, Token::Literal('b')]);
        assert_eq!(token, expected_token);
    }
    #[test]
    fn test_quantifiers() {
        let mut input_string = "{2}";
        let mut tokens = tokenize(input_string).unwrap();
        let mut expected: Vec<Token> = vec![Token::Repetition(2, Some(2))];
        assert_eq!(tokens, expected);

        input_string = "{2, 10}";
        tokens = tokenize(input_string).unwrap();
        expected = vec![Token::Repetition(2, Some(10))];
        assert_eq!(tokens, expected);

        input_string = "{2,}";
        tokens = tokenize(input_string).unwrap();
        expected = vec![Token::Repetition(2, None)];
        assert_eq!(tokens, expected);

        input_string = "{4,3}";
        let tokens_res = tokenize(input_string);
        assert!(tokens_res.is_err());

        input_string = "{2"; //unclosed
        let tokens_res = tokenize(input_string);
        assert!(tokens_res.is_err());

        input_string = "{2}+";
        tokens = tokenize(input_string).unwrap();
        expected = vec![Token::Repetition(2, Some(2)), Token::OneOrMore];
        assert_eq!(tokens, expected);

        input_string = "a{2}b";
        tokens = tokenize(input_string).unwrap();
        expected = vec![Token::Literal('a'), Token::Repetition(2, Some(2)), Token::Literal('b')];
        assert_eq!(tokens, expected);

        input_string = "a{2}bc";
        tokens = tokenize(input_string).unwrap();
        expected = vec![Token::Literal('a'), Token::Repetition(2, Some(2)), Token::Literal('b'), Token::Literal('c')];
        assert_eq!(tokens, expected);
    }
}