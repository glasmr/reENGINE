use crate::engine_nfa::simulate::SearchType;
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
    
    pub fn search(&self, input: &str, search_type: SearchType) -> Result<bool, String> {
        let mut regex_sim = engine_nfa::simulate::Simulator::new(self.nfa.clone().unwrap());
        let sim_result = regex_sim.simulate(input.to_string(), search_type);
        if !sim_result {
            return Ok(false);
        }
        
        let sim_result_unwrapped = sim_result;
        //println!("{:?}", sim_result_unwrapped);
        Ok(sim_result_unwrapped)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_regex() {
        let mut regex = Regex::new();
        regex.compile("ab*c").unwrap();
        let _result = regex.search("xabbcy",  SearchType::Substring).unwrap();
        //dbg!(result);
    }

    #[test]
    fn test_ab_or_c_alternation_fullstring() {
        let mut regex = Regex::new();
        regex.compile("ab|c").unwrap();

        let mut result = regex.search("ab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("c",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("ac",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);

        result = regex.search("abc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);

        let result = regex.search("a",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);

        let result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_ab_or_c_alternation_substring() {
        let mut regex = Regex::new();
        regex.compile("ab|c").unwrap();

        let mut result = regex.search("ab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("c",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("ac",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("abc",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("a",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_a_kleen_start_fullstring() {
        let mut regex = Regex::new();
        regex.compile("a*").unwrap();

        let mut result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("b",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);

        result = regex.search("aab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_a_kleen_start_substring() {
        let mut regex = Regex::new();
        regex.compile("a*").unwrap();

        let mut result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("b",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_concat_ab_kleen_c_fullstring() {
        let mut regex = Regex::new();
        regex.compile("ab*c").unwrap();

        let mut result = regex.search("ac",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("abbc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);

        result = regex.search("xabbcy",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);

        result = regex.search("bc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_concat_ab_kleen_c_substring() {
        let mut regex = Regex::new();
        regex.compile("ab*c").unwrap();

        let mut result = regex.search("ac",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("abbc",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("xabbcy",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("bc",  SearchType::Substring).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn edge_cases_simple() {
        let mut regex = Regex::new();
        regex.compile("a*b*").unwrap();

        let mut result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("ba",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("ba",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        regex.compile("(ab)*").unwrap();
        result = regex.search("abab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("abab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aba",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("aba",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        regex.compile("a+").unwrap();
        result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("bab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("bab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_start_anchor() {
        let mut regex = Regex::new();
        regex.compile("^abc").unwrap();

        let mut result = regex.search("abc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("abc",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("xabc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("xabc",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("abcd",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("abcd",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        regex.compile("^a*b").unwrap();
        result = regex.search("aaab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aaab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("xaaab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("xaaab",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        /*regex.compile("^").unwrap(); //Skip anchor only pattern for now
        result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, true);*/
    }

    #[test]
    fn test_end_anchor() {
        let mut regex = Regex::new();
        regex.compile("abc$").unwrap();

        let mut result = regex.search("abc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("abc",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("abcd",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("abcd",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        regex.compile("a*$").unwrap();
        result = regex.search("bbaaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("bbaaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("aaab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_start_and_end_anchor() {
        let mut regex = Regex::new();
        regex.compile("^abc$").unwrap();

        let mut result = regex.search("abc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("abc",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("xabc",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("xabc",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("abcd",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("abcd",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("xabcd",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("xabcd",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        //regex.compile("^$").unwrap();
    }

    #[test]
    fn test_word_boundary() {
        let mut regex = Regex::new();
        regex.compile("\\bcat\\b").unwrap();

        let mut result = regex.search("the cat sat",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("the cat sat",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("cat",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("cat",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("concatenate",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("concatenate",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("cats",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("cats",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        regex.compile("\\bcat").unwrap();

        result = regex.search("cats",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("cats",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("concatenate",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("concatenate",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        regex.compile("\\b\\w+\\b").unwrap();

        result = regex.search("hello",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("hello",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("hello world",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("hello world",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

   /* #[test]
    fn test_non_word_boundary() {
        let mut regex = Regex::new();
        regex.compile("B\\cat\\B").unwrap();

        let mut result = regex.search("concatenate",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("concatenate",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }*/

    #[test]
    fn test_bounded_repetition_single() {
        let mut regex = Regex::new();
        regex.compile("a{3}").unwrap();

        let mut result = regex.search("aaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("aa",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("aaaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("aaaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        regex.compile("a{0}").unwrap();
        result = regex.search("",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("a",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("a",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bounded_repetition_multiple() {
        let mut regex = Regex::new();
        regex.compile("a{2, 4}").unwrap();

        let mut result = regex.search("aa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaaa",  SearchType::Fullstring).unwrap(); //test sometimes passes and sometimes fails
        assert_eq!(result, true);
        result = regex.search("aaaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("a",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("a",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("aaaaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("aaaaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("baab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("baab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bounded_repetition_unbounded() {
        let mut regex = Regex::new();
        regex.compile("a{2,}").unwrap();

        let mut result = regex.search("aa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("aaaaaaa",  SearchType::Fullstring).unwrap();
        assert_eq!(result, true);
        result = regex.search("aaaaaaa",  SearchType::Substring).unwrap();
        assert_eq!(result, true);

        result = regex.search("a",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("a",  SearchType::Substring).unwrap();
        assert_eq!(result, false);

        result = regex.search("baab",  SearchType::Fullstring).unwrap();
        assert_eq!(result, false);
        result = regex.search("baab",  SearchType::Substring).unwrap();
        assert_eq!(result, true);
    }
}