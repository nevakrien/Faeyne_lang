#[derive(Debug, PartialEq, Clone)]
pub enum LexToken<'a> {
    Word(&'a str),
    Atom(&'a str),
    String(&'a str),
    PoisonString(&'a str),
    Unknowen(&'a str),
}

pub struct Lexer<'input> {
    input: &'input str,
    pos: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer { input, pos: 0 }
    }
}

impl<'input> Iterator for Lexer<'input> {
    
    type Item = Result<((),LexToken<'input>,()), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        // Tokenize each line as a string
        let line = self.input.trim();
        let token = if line.starts_with("\"") && line.ends_with("\"") {
            LexToken::String(line)
        } else if line.starts_with(":") {
            LexToken::Atom(line)
        } else if !line.is_empty() {
            LexToken::Word(line)
        } else {
            LexToken::Unknowen(line)
        };

        // For simplicity, assume each line is a token and stop after one
        self.pos = self.input.len(); // Make sure to stop after one pass
        Some(Ok(((),token,())))	
    }
}

