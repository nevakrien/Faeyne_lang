use codespan::{ByteIndex, Span};
use std::collections::LinkedList;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor::Buffer};
use codespan_reporting::files::SimpleFiles;

use lalrpop_util::ParseError;
use crate::lexer::LexTag;

use crate::ir::FuncSig;
use crate::ast::StringTable;

#[derive(Debug,PartialEq)]
pub enum Error {
    Match(MatchError),
    Sig(SigError),
    Missing(UndefinedName),
    UnreachableCase(UnreachableCase),
    NoneCallble(NoneCallble)
    //UndocumentedError,
}


pub type ErrList = LinkedList<Error>;
impl Error {
    pub fn to_list(self) -> LinkedList<Self> {
        let mut l = LinkedList::new();
        l.push_back(self);
        l
    }
}

pub fn append_err_list(mut a: Result<(),ErrList>, b:Result<(),ErrList>) -> Result<(),ErrList>{
    match &mut a {
        Ok(()) => b,
        Err(l1) => match b{
            Ok(()) => a,
            Err(mut l2) => {
                l1.append(&mut l2);
                a
            }
        }
    }
}


#[derive(Debug,PartialEq)]
pub struct MatchError {
    pub span: Span
}


#[derive(Debug,PartialEq)]
pub struct SigError {
    //placeholder
}

#[derive(Debug,PartialEq)]
pub struct NoneCallble {
    //placeholder
}

#[derive(Debug,PartialEq)]
pub struct UnreachableCase {
    pub name : usize,
    pub sig : FuncSig,
}

#[derive(Debug,PartialEq)]
pub struct UndefinedName {
    pub id : usize,
}

pub trait DiagnosticDisplay {
    fn display_with_table(&self, table: &StringTable) -> String;
}


impl DiagnosticDisplay for FuncSig {
    fn display_with_table(&self, table: &StringTable) -> String {
        let args_names: Vec<_>= self.arg_ids
            .iter()
            .map(|&id| table.get_string(id).unwrap_or("Unknown"))
            .collect();

        format!("[{}]", args_names.join(", "))
    }
}


//subslice has to be a oart of source
pub fn get_subslice_span<'a>(source: &'a str, subslice: &'a str) -> Span {
    // Ensure both `source` and `subslice` have the same lifetime to imply they come from the same memory buffer
    assert!(
        source.as_ptr() <= subslice.as_ptr()
            && subslice.as_ptr() <= unsafe { source.as_ptr().add(source.len()) },
        "Subslice is not part of the source string"
    );

    // Use pointer arithmetic to calculate the start index of the subslice within the source
    let start_index = subslice.as_ptr() as usize - source.as_ptr() as usize;
    let end_index = start_index + subslice.len();

    // Return a Span with the calculated start and end indices
    Span::new(ByteIndex(start_index as u32), ByteIndex(end_index as u32))
}

// Function to handle and report parsing errors
pub fn report_parse_error(err: ParseError<usize, LexTag, ()>, input_ref: &str,_table: &StringTable)  {
    let mut buffer = Buffer::ansi();
    let mut files = SimpleFiles::new();
    let file_id = files.add("input", input_ref);

    let diagnostic = match err {
        ParseError::InvalidToken { location } => Diagnostic::error()
            .with_message("Invalid token")
            .with_labels(vec![Label::primary(file_id, location..location + 1)]),
        ParseError::UnrecognizedEof { location, expected } => Diagnostic::error()
            .with_message("Unexpected end of file")
            .with_labels(vec![Label::primary(file_id, location..location + 1)])
            .with_notes(expected),
        ParseError::UnrecognizedToken { token, expected } => Diagnostic::error()
            .with_message("Unrecognized token")
            .with_labels(vec![Label::primary(file_id, token.0..token.2)])
            .with_notes(expected),
        ParseError::ExtraToken { token } => Diagnostic::error()
            .with_message("Extra token")
            .with_labels(vec![Label::primary(file_id, token.0..token.2)]),
        ParseError::User { .. } => unreachable!(),
    };

    let config = term::Config::default();
    term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();

    println!("{}", String::from_utf8(buffer.into_inner()).unwrap());
    // panic!("Parse error occurred");
}

#[test]
fn test_subslice_span_and_diagnostic_reporting() {
    let source = "Hello, world!\nThis is a test.\nAnother line here.";
    
    // Define a subslice (we'll pretend it's an error in the source code)
    let subslice = &source[14..18]; // This is "This"
    
    // Use the utility function to get the Span
    let span = get_subslice_span(source, subslice);



    let mut files = SimpleFiles::new();
    let file_id = files.add("example.rs", source);

    // Create a diagnostic message (error report) including the &str (subslice) in the label message
    let diagnostic = Diagnostic::note()
        .with_labels(vec![
            Label::primary(file_id, span)
                .with_message(format!("This should be '{}'", subslice)),
        ]);

    // Create a buffer to collect the diagnostic output instead of writing directly to stdout
    let mut buffer = Buffer::ansi();

    // Configure reporting
    let config = term::Config::default();
    
    // Write the diagnostic message to the buffer
    term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();

    // Convert the buffer to a string and print it (or use it for other purposes)
    let output = String::from_utf8(buffer.into_inner()).expect("Buffer should contain valid UTF-8");
    println!("{}", output);

    // Assertion to ensure the span correctly maps back to the original subslice
    let start_index = span.start().to_usize();
    let end_index = span.end().to_usize();
    let extracted_subslice = &source[start_index..end_index];

    // Ensure the extracted subslice matches the original one
    assert_eq!(extracted_subslice, subslice, "The extracted subslice does not match the original subslice.");
}



pub fn report_err_list(err_list: &ErrList, input_ref: &str, table: &StringTable) {
    let mut buffer = Buffer::ansi();
    let mut files = SimpleFiles::new();
    let file_id = files.add("input", input_ref);

    for err in err_list {
        let diagnostic = match err {
            Error::Match(m_err) => Diagnostic::error()
                .with_message("Match error")
                .with_labels(vec![
                    Label::primary(file_id, m_err.span.start().to_usize()..m_err.span.end().to_usize())
                        .with_message("Issue with match expression"),
                ]),

            Error::Sig(_s_err) => Diagnostic::error()
                .with_message("Signature error: ___"),

            Error::Missing(UndefinedName { id }) => Diagnostic::error()
                .with_message(format!(
                    "Undefined name error: {}",
                    table.get_string(*id).unwrap_or("Unknown name")
                )),

            Error::UnreachableCase(UnreachableCase { name, sig }) => {
                let name_str = table.get_string(*name).unwrap_or("Unknown name");
                let sig_display = sig.display_with_table(table);
                Diagnostic::error()
                    .with_message(format!(
                        "Unreachable case error: The case '{}' with signature {} is unreachable.",
                        name_str, sig_display
                    ))
            },

            Error::NoneCallble(_none_call) => Diagnostic::error()
                .with_message("NoneCallable error: Attempted to call a non-callable object."),

            // Placeholder for other potential error types
            // Error::UndocumentedError => todo!("Report Undocumented Error")
        };

        let config = term::Config::default();
        term::emit(&mut buffer, &config, &files, &diagnostic).unwrap();
    }

    // Print the full diagnostic report
    println!("{}", String::from_utf8(buffer.into_inner()).unwrap());
}


#[test]
fn test_err_list_reporting() {
    let mut table = StringTable::new();
    let undef_id = table.get_id("baba");
    let unreachable_name_id = table.get_id("foo");

    let source = "fn main() { let x = 5; baba}";
    let mut err_list = LinkedList::new();

    // Example errors for testing
    let match_err = Error::Match(MatchError {
        span: Span::new(ByteIndex(3), ByteIndex(7)),
    });

    let sig_err = Error::Sig(SigError {});
    let undef_err = Error::Missing(UndefinedName { id: undef_id });

    // Simulate unreachable case
    let unreachable_case_err = Error::UnreachableCase(UnreachableCase {
        name: unreachable_name_id,
        sig: FuncSig {arg_ids:vec![undef_id]}, // Assuming FuncSig has a default or placeholder
    });

    err_list.push_back(match_err);
    err_list.push_back(sig_err);
    err_list.push_back(undef_err);
    err_list.push_back(unreachable_case_err);

    // Report the errors
    report_err_list(&err_list, source, &table);
}
