/*
Let's start by defining my parsers grammar
This parser will use recursive decent


We will define a BNF grammar, with which the parser will follow
        <regex>       ::=   regex '|' top
                      |     regex

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
                      |     '(' <regex> ')'

        <literal>   ::=     "any 'Literal' token"
        <any>       ::=     "any 'Any' token"

 */
use crate::types::token_types::{GroupType, Token};
use crate::types::ast_types::{NodeAST, QuantifierType};

use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens:Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse_regex(&mut self) -> Result<Box<NodeAST>, String> {
        let node = self.parse_anchor()?;
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

    fn parse_anchor(&mut self) -> Result<Box<NodeAST>, String> {
        let current_token = self.tokens.peek();
        if *current_token.unwrap() == Token::StartAnchor {
            self.tokens.next();
            let node = self.parse_term()?;
            return Ok(Box::new(NodeAST::AnchorStart(node)));
        }
        let node = self.parse_term()?;
        let next_token = self.tokens.peek();
        if *next_token.unwrap() == Token::EndAnchor {
            self.tokens.next();
            return Ok(Box::new(NodeAST::AnchorEnd(node)));
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<Box<NodeAST>, String> {
        let node = self.parse_factor()?;
        let next_token = self.tokens.peek();

        match next_token {
            Some(token) => match token {
                Token::Literal(_) | Token::Any | Token::LeftParen(_, _) => {
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
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrMore)))
                }
                Token::OneOrMore => {
                    self.tokens.next();
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::OneOrMore)))
                }
                Token::ZeroOrOne => {
                    self.tokens.next();
                    Ok(Box::new(NodeAST::Quantifier(node, QuantifierType::ZeroOrOne)))
                }
                Token::Repetition(m, n) => {
                    let m = *m;
                    let n = *n;
                    self.tokens.next();
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
            Token::Any => Ok(Box::new(NodeAST::Any)),
            Token::LeftParen(group_type, group) => {
                match group_type {
                    GroupType::NonCapturing => {
                        let node = self.parse_regex()?;

                        match self.tokens.next() {
                            Some(Token::RightParen) => Ok(node),
                            Some(unexpected_token) => Err(format!("Unexpected token {:?}, expected )", unexpected_token)),
                            None => Err(String::from("Unexpected EOF")),
                        }
                    }
                    GroupType::Capturing => {
                        //self.group += 1;
                        let node = self.parse_regex()?;
                        match self.tokens.next() {

                            Some(Token::RightParen) => {
                                Ok(Box::new(NodeAST::CaptureGroup(node, group.unwrap())))
                            }
                            Some(unexpected_token) => Err(format!("Unexpected token {:?}, expected )", unexpected_token)),
                            None => Err(String::from("Unexpected EOF")),
                        }
                    }
                }
            }
            _ => {
                //dbg!(token);
                Err(String::from("Error: Unexpected token"))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;
    #[test]
    fn test_parser() {
        let tokens = tokenize("");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_regex();
        dbg!(&ast);
    }
}