use crate::capture::{Capture, Captures};
use crate::error::RegexError;
use crate::nfa::simulate::{Simulator};

pub struct Regex {
    simulator: Simulator,
}
impl Regex {
    pub fn new(pattern: &str) -> Result<Self, RegexError> {
        Ok(
            Regex { simulator: Simulator::new(pattern)? }
        )
    }

    pub fn is_match(&self, text: &str) -> bool {
        unimplemented!()
    }

    pub fn find(&self, text: &str) -> Option<Capture> {

        unimplemented!()
    }

    pub fn find_all(&self, text: &str) -> Option<Captures> {
        unimplemented!()
    }

    pub fn find_all_iter(&self, text: &str) -> Captures {
        unimplemented!()
    }
}

pub struct RegexBuilder {}
impl RegexBuilder {}