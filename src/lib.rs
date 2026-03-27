use crate::types::nfa_types::NFA;

mod tokenizer;
mod types;
mod parse_ast;
mod engine_nfa;
pub mod matcher;

pub struct Regex {
    nfa: Option<NFA>
}
impl Regex {
    pub fn new() -> Self {
        Regex {
            nfa: None
        }
    }
    
    pub fn compile(&mut self, input: &str) -> Result<(), String> {
        let tokens = tokenizer::tokenize(input)?;
        let mut parser = parse_ast::Parser::new(tokens);
        let ast = parser.parse_regex()?;
        let mut nfa_builder = engine_nfa::nfa_builder::BuilderNFA::new();
        self.nfa = Some(nfa_builder.compile(&ast)?);
        
        Ok(())
    }
    
    pub fn search(&self, input: &str) -> Result<(bool, Option<Vec<char>>), String> {
        let mut regex_sim = engine_nfa::simulate::Simulator::new(self.nfa.clone().unwrap());
        let sim_result = regex_sim.simulate(input.to_string());
        if sim_result.is_none() {
            return Ok((false, None));
        }
        
        let sim_result_unwrapped = sim_result.unwrap();
        let str_vec = sim_result_unwrapped.0.chars().collect::<Vec<char>>();
        let mut matches: Vec<char> = Vec::new();
        dbg!(&str_vec);
        dbg!(&sim_result_unwrapped.1);
        let mut sim_res_iter  = sim_result_unwrapped.1.iter();
        let mut idx: Option<&usize> = None;
        loop {
            idx = sim_res_iter.next();
            if idx.is_none() {break;}
            println!("{}", idx.unwrap());
            matches.push(str_vec[*idx.unwrap()]);
        }
        Ok((true, Some(matches)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_regex() {
        let mut regex = Regex::new();
        regex.compile("ab|c").unwrap();
        let result = regex.search("ab").unwrap();
        dbg!(result);
    }
}