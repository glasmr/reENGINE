use std::ops::Range;

#[derive(Clone)]
pub struct Capture {
    text: String,
    start: usize,
    end: usize
}
impl Capture {
    pub fn as_str(&self) -> &str { &self.text }
    pub fn start(&self) -> usize { self.start }
    pub fn end(&self) -> usize { self.end }
    pub fn range(&self) -> Range<usize> { self.start .. self.end }
}

pub struct Captures {
    captures: Vec<Capture>,
    index: usize
}

impl Captures {
    pub fn new(raw_captures: Vec<(usize, usize)>) -> Self {
        Captures { captures: Vec::new(), index: 0 }
    }
}

impl Iterator for Captures {
    type Item = Capture;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.captures.len() {
            let capture = self.captures[self.index].clone();
            self.index += 1;
            Some(capture)
        } else {
            None
        }
    }
}

