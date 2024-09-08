use crate::reporting::get_subslice_span;
use nom::bytes::complete::{take_till, tag, take_while};
use nom::character::complete::{not_line_ending};
use nom::combinator::{opt, recognize};
use nom::sequence::{terminated, delimited};
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum LexTag {
    Name,
    Atom,
    String,
    PoisonString,
    Unknown,
}

pub struct Lexer<'input> {
    input: &'input str,
    remaining_input: &'input str, // Tracks remaining unprocessed input
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer {
            input,
            remaining_input: input, // Initialize with full input
        }
    }

    // Use nom to take till newline
    fn take_line(input: &str) -> IResult<&str, &str> {
        terminated(take_till(|c| c == '\n'), opt(tag("\n")))(input)
    }

    // Detect a string (anything enclosed in quotes)
    fn parse_string(input: &str) -> IResult<&str, &str> {
        delimited(tag("\""), take_till(|c| c == '\"'), tag("\""))(input)
    }

    // Detect an atom (starts with `:` and followed by non-space characters)
    fn parse_atom(input: &str) -> IResult<&str, &str> {
        recognize(terminated(tag(":"), not_line_ending))(input)
    }

    // Detect a name (anything that's not an atom or string)
    fn parse_name(input: &str) -> IResult<&str, &str> {
        take_till(|c| c == '\n' || c == ' ')(input)
    }

    // Parse and skip leading whitespace
    fn parse_empty(input: &str) -> IResult<&str, &str> {
        take_while(|c: char| c.is_whitespace())(input)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, LexTag, usize), ()>;

    fn next(&mut self) -> Option<Self::Item> {

        // Skip leading whitespace
        match Lexer::parse_empty(self.remaining_input) {
            Ok((new_input, _)) => {
                self.remaining_input = new_input; // Update remaining input after skipping whitespace
            }
            Err(_) => return None, // If something fails, we end the parsing
        }

        if self.remaining_input.is_empty() {
            return None;
        }

        // Parse the next line after skipping whitespace
        match Lexer::take_line(self.remaining_input) {
            Ok((remaining, line)) => {
                let trimmed_line = line.trim();
                let span = get_subslice_span(self.input, line);

                // Try to match the line to known token types
                let token = if let Ok((_, _)) = Lexer::parse_string(trimmed_line) {
                    // println!("Emitting string: {:?}", trimmed_line);
                    LexTag::String
                } else if let Ok((_, _)) = Lexer::parse_atom(trimmed_line) {
                    // println!("Emitting atom: {:?}", trimmed_line);
                    LexTag::Atom
                } else if let Ok((_, _)) = Lexer::parse_name(trimmed_line) {
                    // println!("Emitting name: {:?}", trimmed_line);
                    LexTag::Name
                } else {
                    // println!("Emitting unknown token: {:?}", trimmed_line);
                    LexTag::Unknown
                };

                // Update the remaining input
                self.remaining_input = remaining;

                // Return the token with its span
                Some(Ok((span.start().to_usize(), token, span.end().to_usize())))
            }
            Err(_) => {
                // If parsing fails, return None
                None
            }
        }
    }
}
