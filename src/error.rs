
#[derive(Debug, thiserror::Error)]
pub enum RegexError {
    #[error("parse error at position {pos}")]
    Parse {pos: usize},

}

