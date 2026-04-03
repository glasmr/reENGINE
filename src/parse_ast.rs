/*
Let's start by defining my parsers grammar
This parser will use recursive decent


We will define a BNF grammar, with which the parser will follow
        <regex>       ::=   anchor '|' regex
                      |     anchor

        <anchor>     ::=    '^' term
                      |     term '$'
                      |     term

        <term>      ::=     <factor> <term>
                      |     <factor>

        <factor>    ::=     <primary> '*'
                      |     <primary> '+'
                      |     <primary> '?'
                      |     <primary> {m, n}
                      |     <primary>

        <primary>   ::=     <literal> **bracket expressions are dealt with by lexer, and treated like a literal
                      |     <any>
                      |     <word-boundary>
                      |     '(' <regex> ')'

        <literal>   ::=     "any 'Literal' token"
        <any>       ::=     "any 'Any' token"
        <word-boundary> ::= '\b' | '\B'

 */
use crate::types::token_types::{Token};
use crate::types::ast_types::{NodeAST, QuantifierType};

use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn parse_regex(&mut self) -> Result<Box<NodeAST>, String> {
        let node = self.parse_start_anchor()?;
        let next_token = self.tokens.peek();
        match next_token {
            Some(token) => match token {
                Token::Alternation => {
                    self.tokens.next();
                    let next_node = self.parse_regex()?;
                    Ok(Box::new(NodeAST::Alternation(node, next_node)))
                }
                _ => {Ok(node)}
            }
            None => {Ok(node)}
        }
    }

    fn parse_start_anchor(&mut self) -> Result<Box<NodeAST>, String> {
        let current_token = self.tokens.peek();
        if *current_token.ok_or(String::from("Error: Unexpected EOF"))? == Token::StartAnchor {
            self.tokens.next();
            let node = self.parse_end_anchor()?;
            return Ok(Box::new(NodeAST::AnchorStart(node)));
        }
        let node = self.parse_end_anchor()?;
        Ok(node)
        /*let next_token = self.tokens.peek();
        match next_token {
            Some(token) => {
                match token {
                    Token::EndAnchor => {
                        self.tokens.next();
                        Ok(Box::new(NodeAST::AnchorEnd(node)))
                    }
                    _ => { Ok(node) }
                }
            }
            None => { Ok(node) }
        }*/
    }

    fn parse_end_anchor(&mut self) -> Result<Box<NodeAST>, String> {
        let node = self.parse_term()?;
        let next_token = self.tokens.peek();
        if !next_token.is_none() {
            if let Token::EndAnchor = next_token.unwrap() {
                self.tokens.next();
                return Ok(Box::new(NodeAST::AnchorEnd(node)));
            }
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<Box<NodeAST>, String> {
        let node = self.parse_factor()?;
        let next_token = self.tokens.peek();

        match next_token {
            Some(token) => match token {
                Token::Literal(_) | Token::WildCard | Token::LeftParen
                | Token::CharacterSet(_, _) | Token::CharacterClass(_)
                | Token::WordBoundary | Token::NonWordBoundary => {
                    let next_node = self.parse_term()?;
                    Ok(Box::new(NodeAST::Concatenation(node, next_node)))
                }
                _ => {Ok(node)}
            }
             None => Ok(node)
        }
    }

    fn parse_factor(&mut self) -> Result<Box<NodeAST>, String> {
        let node =self.parse_primary()?;
        let next_token = self.tokens.peek();
        match next_token {
            Some(token) => match token {
                Token::ZeroOrMore => {
                    self.tokens.next();
                    let validate_next = self.validate_next_not_quantifier();
                    if validate_next.is_err() {return Err(validate_next.unwrap_err())}
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrMore)))
                }
                Token::OneOrMore => {
                    self.tokens.next();
                    let validate_next = self.validate_next_not_quantifier();
                    if validate_next.is_err() {return Err(validate_next.unwrap_err())}
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::OneOrMore)))
                }
                Token::ZeroOrOne => {
                    self.tokens.next();
                    let validate_next = self.validate_next_not_quantifier();
                    if validate_next.is_err() {return Err(validate_next.unwrap_err())}
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrOne)))
                }
                Token::Repetition(m, n) => {
                    let m = *m;
                    let n = *n;
                    self.tokens.next();
                    let validate_next = self.validate_next_not_quantifier();
                    if validate_next.is_err() {return Err(validate_next.unwrap_err())}
                    match (m, n) { //Normalize equivalent quantifiers
                        (0, Some(1)) => {return Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrOne)))} // ?
                        (1, None) => {return Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::OneOrMore)))} // +
                        (0, None) => {return Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrMore)))} // *
                        _ => {}
                    }
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::Repetition(m, n))))
                }
                _ => {Ok(node)}
            }
            None => Ok(node)
        }
    }

    fn parse_primary(&mut self) -> Result<Box<NodeAST>, String> {
        let token = self.tokens.next().ok_or(String::from("Error(Primary, Next Token): Unexpected EOF"))?;
        //dbg!(&token);

        match token {
            Token::Literal(value) => Ok(Box::new(NodeAST::Literal(value))),
            Token::CharacterClass(value) => Ok(Box::new(NodeAST::CharacterClass(value))),
            Token::CharacterSet(negated, character_sets) => {Ok(Box::new(NodeAST::CharacterSet(negated, character_sets)))}
            Token::WildCard => Ok(Box::new(NodeAST::Any)),
            Token::WordBoundary => {Ok(Box::new(NodeAST::WordBoundary))}
            Token::NonWordBoundary => {Ok(Box::new(NodeAST::NonWordBoundary))}
            Token::LeftParen => {
                let node = self.parse_regex()?;

                match self.tokens.next() {
                    Some(Token::RightParen) => {
                        Ok(Box::new(NodeAST::Group(node)))
                    }
                    Some(unexpected_token) => Err(format!("Unexpected token {:?}, expected )", unexpected_token)),
                    None => Err(String::from("Unexpected EOF")),
                }
            }
            _ => {
                //dbg!(token);
                Err(format!("Error: Unexpected token {:?}", token))
            }
        }
    }

    fn validate_next_not_quantifier(&mut self) -> Result<(), String> {
        match self.tokens.peek() {
            Some(token) => match token {
                Token::ZeroOrOne => {Err(String::from("Parse Error: Lazy Quantifiers not supported!"))},
                Token::ZeroOrMore | Token::OneOrMore | Token::Repetition(_, _) => {Err(String::from("Parse Error: Quantifiers cannot be chained!"))}
                _ => Ok(())
            }
            None => Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    use crate::types::ast_types::NodeAST;
    use crate::types::token_types::{CharClassType, CharSetType};

    #[test]
    fn test_precedence() {
        let mut input = "ab*";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        let mut expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Literal('b')),
                QuantifierType::ZeroOrMore
            ))
        ));
        assert_eq!(ast, expected_ast);

        input = "ab|cd";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Literal('a')),
                Box::new(NodeAST::Literal('b')),
            )),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Literal('c')),
                Box::new(NodeAST::Literal('d')),
            ))
        ));
        assert_eq!(ast, expected_ast);

        input = "a|b*";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Literal('b')),
                QuantifierType::ZeroOrMore
            ))
        ));
        assert_eq!(ast, expected_ast);

        input = "^a|b$";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::AnchorStart(Box::new(NodeAST::Literal('a')))),
            Box::new(NodeAST::AnchorEnd(Box::new(NodeAST::Literal('b'))))
        ));
        assert_eq!(ast, expected_ast);

        input = "^a+";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        expected_ast = Box::new(NodeAST::AnchorStart(Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('a')), QuantifierType::OneOrMore))));
        assert_eq!(ast, expected_ast);

        input = "a|b|c";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex().unwrap();
        expected_ast = Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('a')),
                             Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('b')), Box::new(NodeAST::Literal('c'))))));
        assert_eq!(ast, expected_ast);
    }
    #[test]
    fn test_alternation_associativity() {
        let mut input = "a|b|c";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('a')),
                                                             Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('b')),
                                                                                           Box::new(NodeAST::Literal('c'))))));
        assert_eq!(ast.unwrap(), expected_ast);

        input = "a|b|c|d";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('b')),
                                      Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('c')),
                                                                    Box::new(NodeAST::Literal('d'))))))));
        assert_eq!(ast.unwrap(), expected_ast);

        input = "(a|b)|c";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Alternation(Box::new(NodeAST::Group(
            Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Literal('b'))))
        )), Box::new(NodeAST::Literal('c'))));
        assert_eq!(ast.unwrap(), expected_ast);

        input = "a|(b|c)";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Group(
            Box::new(NodeAST::Alternation(Box::new(NodeAST::Literal('b')), Box::new(NodeAST::Literal('c'))))
        ))));
        assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_alternation_arms_full_concat() {
        let mut input = "ab|cd";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Literal('b')))),
            Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('c')), Box::new(NodeAST::Literal('d'))))
        ));
        assert_eq!(ast.unwrap(), expected_ast);

        input = "abc|def";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('b')), Box::new(NodeAST::Literal('c')))))),
            Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('d')), Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('e')), Box::new(NodeAST::Literal('f'))))))));
        assert_eq!(ast.unwrap(), expected_ast);

        input = "ab|cd|ef";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Alternation(
            Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('a')), Box::new(NodeAST::Literal('b')))),
            Box::new(NodeAST::Alternation(
                Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('c')), Box::new(NodeAST::Literal('d')))),
                Box::new(NodeAST::Concatenation(Box::new(NodeAST::Literal('e')), Box::new(NodeAST::Literal('f')))))
            ))
        );
        assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_alternation_empty_arms() {
        let mut input = "|a";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        //dbg!(&ast);
        assert!(ast.is_err());

        input = "a|";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        //dbg!(&ast);
        assert!(ast.is_err());

        input = "|";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        //dbg!(&ast);
        assert!(ast.is_err());

        input = "a||b";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        //dbg!(&ast);
        assert!(ast.is_err());
    }
    #[test]
    fn test_concat_with_quantifiers() {
        let mut input = "ab*";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('b')), QuantifierType::ZeroOrMore)))
        );
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "ab+c";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('b')), QuantifierType::OneOrMore)),
                Box::new(NodeAST::Literal('c'))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a*b*";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('a')), QuantifierType::ZeroOrMore)),
            Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('b')), QuantifierType::ZeroOrMore))

        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "ab?c";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Quantifier(
                    Box::new(NodeAST::Literal('b')),
                    QuantifierType::ZeroOrOne
                )),
                Box::new(NodeAST::Literal('c'))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a*b+c?d";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('a')), QuantifierType::ZeroOrMore)),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('b')), QuantifierType::OneOrMore)),
                Box::new(NodeAST::Concatenation(
                    Box::new(NodeAST::Quantifier(Box::new(NodeAST::Literal('c')), QuantifierType::ZeroOrOne)),
                    Box::new(NodeAST::Literal('d'))
                ))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_grouping_inside_concat() {
        let mut input = "a(bc)d";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Group(
                    Box::new(NodeAST::Concatenation(
                        Box::new(NodeAST::Literal('b')),
                        Box::new(NodeAST::Literal('c'))
                    )))),
                Box::new(NodeAST::Literal('d'))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "(ab)(cd)";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Group(
                Box::new(NodeAST::Concatenation(
                    Box::new(NodeAST::Literal('a')),
                    Box::new(NodeAST::Literal('b'))
                ))
            )),
            Box::new(NodeAST::Group(
                Box::new(NodeAST::Concatenation(
                    Box::new(NodeAST::Literal('c')),
                    Box::new(NodeAST::Literal('d'))
                ))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a(b|c)d";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Group(
                    Box::new(NodeAST::Alternation(
                        Box::new(NodeAST::Literal('b')),
                        Box::new(NodeAST::Literal('c'))
                    ))
                )),
                Box::new(NodeAST::Literal('d'))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_quantifier_single_atom() {
        let mut input = "ab*";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Literal('b')),
                QuantifierType::ZeroOrMore
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a*b";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Literal('a')),
                QuantifierType::ZeroOrMore
            )),
            Box::new(NodeAST::Literal('b'))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "abc*";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Literal('a')),
            Box::new(NodeAST::Concatenation(
                Box::new(NodeAST::Literal('b')),
                Box::new(NodeAST::Quantifier(
                    Box::new(NodeAST::Literal('c')),
                    QuantifierType::ZeroOrMore
                ))
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = ".+";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Any),
            QuantifierType::OneOrMore
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "[a-z]?";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::CharacterSet(CharSetType::NonNegated, vec![CharClassType::Range('a', 'z')])),
            QuantifierType::ZeroOrOne
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_group_is_single_atom() {
        let mut input = "(ab)?";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Group(
                Box::new(NodeAST::Concatenation(
                    Box::new(NodeAST::Literal('a')),
                    Box::new(NodeAST::Literal('b'))
                ))
            )),
            QuantifierType::ZeroOrOne
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "(a|b)+";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Group(
                Box::new(NodeAST::Alternation(
                    Box::new(NodeAST::Literal('a')),
                    Box::new(NodeAST::Literal('b'))
                ))
            )),
            QuantifierType::OneOrMore
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "(abc)*";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Group(
                Box::new(NodeAST::Concatenation(
                    Box::new(NodeAST::Literal('a')),
                    Box::new(NodeAST::Concatenation(
                        Box::new(NodeAST::Literal('b')),
                        Box::new(NodeAST::Literal('c'))
                    ))
                ))
            )),
            QuantifierType::ZeroOrMore
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "(ab)+(cd)?";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Concatenation(
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Group(
                    Box::new(NodeAST::Concatenation(
                        Box::new(NodeAST::Literal('a')),
                        Box::new(NodeAST::Literal('b'))
                    ))
                )),
                QuantifierType::OneOrMore
            )),
            Box::new(NodeAST::Quantifier(
                Box::new(NodeAST::Group(
                    Box::new(NodeAST::Concatenation(
                        Box::new(NodeAST::Literal('c')),
                        Box::new(NodeAST::Literal('d'))
                    ))
                )),
                QuantifierType::ZeroOrOne
            ))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);
    }
    #[test]
    fn test_bounded_repetition() {
        let mut input = "a{3}";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        let mut expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::Repetition(3, Some(3))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{3,}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::Repetition(3, None)
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{3,7}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::Repetition(3, Some(7))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{0,1}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::ZeroOrOne
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{1,}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::OneOrMore
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{0,}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::ZeroOrMore
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{0}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        expected_ast = Box::new(NodeAST::Quantifier(
            Box::new(NodeAST::Literal('a')),
            QuantifierType::Repetition(0, Some(0))
        ));
        debug_assert_eq!(ast.unwrap(), expected_ast);

        input = "a{2, 1}";
        debug_assert!(tokenize(input).is_err());
    }
    #[test]
    fn test_chained_quantifier() {
        let mut input = "a*?";
        let mut ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        assert!(ast.is_err());

        input = "a**";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        assert!(ast.is_err());

        input = "a*+";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        assert!(ast.is_err());

        input = "a{2}+";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        //dbg!(&ast);
        assert!(ast.is_err());

        input = "a?{2}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        assert!(ast.is_err());

        input = "a{2}{2,3}";
        ast = Parser::new(tokenize(input).unwrap()).parse_regex();
        assert!(ast.is_err());
    }
    #[test]
    fn test_nested_grouping() {
        
    }
    #[test]
    fn test_things_known_to_break() {}
}