use crate::reporting::get_subslice_span;
use nom::bytes::complete::{is_a,is_not,take_till,take_while,take_while1};
use nom::combinator::{opt, recognize};
use nom::branch::alt;
use nom::multi::many0_count;
use nom::IResult;
use nom::sequence::{preceded,pair,terminated};
use nom::character::complete::{anychar,one_of};


#[derive(Debug, PartialEq, Clone)]
pub enum LexTag {
    Name,
    Atom,
    String,
    PoisonString,
    Unknowen,

    Import,
    FuncDec,
    Lambda,

    Match,

    Float(f64), // We can parse in a way that never overflows
    Int(Result<i64, f64>), // If we get a float here, we know it overflowed

    Dot,
    Ender,
    Comma,

    // Parens
    OpenParen,
    CloseParen,

    OpenCurly,
    CloseCurly,

    OpenSquare,
    CloseSquare,

    Pipe,

    // Operators
    Plus,
    Minus,
    Mul,
    Div,
    IntDiv,
    Pow,

    Modolo,
    DoubleAnd,
    DoubleOr,
    DoubleXor,

    Eq,
    DoubleEq,
    
    And,
    Or,
    Xor,
    
    Smaller,
    SmallerEq,
    Bigger,

    Arrow,
    SmallArrow
}

type RawResult<'a> = IResult<&'a str, &'a str, ()>;
type LexResult<'a> = IResult<&'a str, LexTag, ()>;

fn lex_comment<'a>(input: &'a str) -> RawResult<'a>{
    recognize(
        preceded(is_a("#"), 
            terminated(take_till(|c| c == '\n'),opt(is_a("\n")))
    ))(input)
}

fn lex_spaces<'a>(input: &'a str) -> RawResult<'a>{
    take_while1(|c: char| c.is_whitespace() || c=='\n')(input)
    // recognize(many0_count(one_of(" \n")))(input)
}

fn lex_skipble<'a>(input: &'a str) -> RawResult<'a> {
    recognize(many0_count(alt((lex_comment,lex_spaces))))(input)
}

fn skip<'a>(input: &'a str) -> (&'a str, &'a str) {
    recognize(opt(lex_skipble))(input).unwrap()
}

fn get_line<'a>(input: &'a str) -> (&'a str, &'a str) {
    fn lex_line<'a>(input: &'a str) -> RawResult<'a> {
        take_till(|c| c == '\n')(input)
    }
    let (input, _dump) = skip(input);
    let result = recognize(opt(lex_line))(input).unwrap();
    //println!("After skipping \"{}\" remaining in line: \"{}\" remaining: \"{}\"\n", _dump,result.1,result.0); // Debug output
    result
}



fn lex_unknowen<'a>(input: &'a str) -> LexResult<'a>{
    let (input,_)=recognize(pair(anychar,take_while(|c:char| !c.is_ascii())))(input)?;
    Ok((input,LexTag::Unknowen))
}

fn lex_string<'a>(input: &'a str) -> LexResult<'a> {
    // Match the string delimiters (either ' or ")
    let (input, d) = recognize(one_of("'\""))(input)?;
    
    // Process the content inside the delimiters
    let (input, last) = preceded(
        // Handle either escaped delimiters (like \' or \") or any character except the delimiter
        many0_count(alt((
            recognize(preceded(is_a("\\"), one_of(d))), // Handles escaped delimiters
            is_not(d),                     // Everything else that's not the delimiter
        ))),
        opt(one_of(d))  // Expect to match the closing delimiter
    )(input)?;

    match last {
        None => Ok((input,LexTag::PoisonString)),
        Some(_) => Ok((input,LexTag::String)),
    }
}

fn lext_token<'a>(input: &'a str) -> LexResult<'a>{
    alt((
        // lex_word,
        // lex_atom,
        // lex_ender,
        // lex_delimiter,
        // lex_operator,
        // lex_number,
        lex_string,
        lex_unknowen,
    ))(input)
}

pub struct Lexer<'input> {
    original_input: &'input str,
    index: usize,
    line: &'input str, // Tracks remaining unprocessed input
    next_input: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let (rem, dump) = skip(input);
        let span = get_subslice_span(input, dump);
        let (next, line) = get_line(rem);

        //println!("Initial line: '{}', Next input: '{}'", line, next); // Debug output

        Lexer {
            original_input: input,
            index: span.end().to_usize(),
            line: line,
            next_input: next,
        }
    }

    fn skip(&mut self) {
        match lex_skipble(self.line) {
            Err(_) => {return;},
            Ok((line, _dump)) => {
                //println!("Skipped to: '{}'", line); // Debug output
                self.line = line;
            }
        }

        if self.line.len() == 0  {
            let (next_input, line) = get_line(self.next_input);
            self.next_input = next_input;
            self.line = line;
            //println!("Moved to next input: '{}', Next line: '{}'", self.next_input, self.line); // Debug output
        }

        let span = get_subslice_span(self.original_input, self.line);
        self.index = span.start().to_usize();
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, LexTag, usize), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip();
        // if self.line.len() == 0 {
        //     //println!("End of input reached."); // Debug output
        //     return None;
        // }

        match lext_token(self.line) {
            Err(_) => {
                //println!("End of input reached."); // Debug output
                None
            },
            Ok((rem,tag)) => {
                //println!("Processing line: '{}'", self.line); // Debug output
                let span = get_subslice_span(self.original_input, rem);
                let end = span.start().to_usize();

                let ans = (self.index,tag,end);

                self.line = rem;
                self.index = end;
                Some(Ok(ans))
            }
        }
    }
}
