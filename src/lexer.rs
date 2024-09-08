use crate::reporting::get_subslice_span;
use codespan::Span;
use nom::bytes::complete::take_till;
use nom::character::complete::line_ending;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::sequence::terminated;
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
        terminated(take_till(|c| c == '\n'),opt(tag("\n")))(input)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, LexTag, usize), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_input.is_empty() {
            return None;
        }

        // Parse next line with nom
        match Lexer::take_line(self.remaining_input) {
            Ok((remaining, line)) => {
                // Tokenize the line as a string (assuming every line is a string)
                let trimmed_line = line.trim();

                // Calculate the span using the utility function
                let span = get_subslice_span(self.input, line);

                // Return LexTag::String for every line that is non-empty
                if !trimmed_line.is_empty() {
                    self.remaining_input = remaining; // Update remaining input
                    
                    Some(Ok((span.start().to_usize(), LexTag::String, span.end().to_usize())))
                } else {
                    // Skip empty lines and continue to the next line
                    self.remaining_input = remaining; // Update remaining input
                    self.next() // Recursively call next to get the next valid line
                }
            }
            Err(_) => {
                // If parsing fails, return None
                None
            }
        }
    }
}
