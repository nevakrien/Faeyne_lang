use codespan::Span;

#[cfg(test)]
use codespan::ByteIndex;

use std::collections::LinkedList;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor::Buffer};
use codespan_reporting::files::SimpleFiles;

use lalrpop_util::ParseError;
use ast::lexer::LexTag;

use ast::ast::FuncSig;
use ast::ast::StringTable;

#[derive(Debug,PartialEq)]
pub enum Error {
    Match(MatchError),
    Sig(SigError),
    ZeroDiv,

    MissingCall(String),


    Missing(UndefinedName),
    UnreachableCase(FuncSig),
    NoneCallble(NoneCallble),
    Stacked(InternalError),
    StackedTail(InternalError),
    IllegalSelfRef(IllegalSelfRef),
    
    Recursion(RecursionError),
    StackOverflow,

    Bug(&'static str),
    //UndocumentedError,
}


pub type ErrList = LinkedList<Error>;

pub fn vec_to_list(errors: Vec<Error>) -> ErrList {
        let mut err_list = LinkedList::new();
        for error in errors {
            err_list.push_back(error);
        }
        err_list
}

impl Error {
    #[cold]
#[inline(never)]
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

#[cold]
#[inline(never)]
pub fn bug_error(message: &'static str) -> ErrList {
    Error::Bug(message).to_list()
}

#[cold]
#[inline(never)]
pub fn overflow_error() -> ErrList {
    Error::StackOverflow.to_list() 
}

#[cold]
#[inline(never)]
pub fn sig_error() -> ErrList {
    Error::Sig(SigError{}).to_list() 
}

#[cold]
#[inline(never)]
pub fn zero_div_error() -> ErrList {
    Error::ZeroDiv.to_list() 
}

#[cold]
#[inline(never)]
pub fn recursion_error(depth:usize) -> ErrList {
    Error::Recursion(RecursionError{depth}).to_list()
}

#[cold]
#[inline(never)]
pub fn match_error(span:Span) -> ErrList {
    Error::Match(MatchError{span}).to_list()
}

#[cold]
#[inline(never)]
pub fn stacked_error(message:&'static str,err:ErrList,span:Span) -> ErrList {
    Error::Stacked(InternalError{
            message,
            err,
            span,
        }).to_list()
}

#[cold]
#[inline(never)]
pub fn tail_stacked_error(message:&'static str,err:ErrList,span:Span) -> ErrList {
    Error::StackedTail(InternalError{
            message,
            err,
            span,
        }).to_list()
}

#[cold]
#[inline(never)]
pub fn missing_func_error(name:String) -> ErrList {
    Error::MissingCall(name).to_list()
}


#[derive(Debug,PartialEq)]
pub struct RecursionError{
    pub depth:usize
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
    pub span: Span,
    pub value: String,
}

// #[derive(Debug,PartialEq)]
// pub struct UnreachableCase {
//     // pub name : u32,
//     pub sig : FuncSig,
// }

#[derive(Debug,PartialEq)]
pub struct UndefinedName {
    pub id : u32,
}

#[derive(Debug,PartialEq)]
pub struct InternalError {
    pub message: &'static str,
    pub span : Span,
    pub err : ErrList
}

#[derive(Debug,PartialEq)]
pub struct IllegalSelfRef {
    pub span : Span,
}

pub trait DiagnosticDisplay {
    fn display_with_table(&self, table: &StringTable) -> String;
}


impl DiagnosticDisplay for FuncSig {
    fn display_with_table(&self, table: &StringTable) -> String {
        let args_names: Vec<_>= self.args
            .iter()
            .map(|&id| table.get_display_str(id).unwrap_or("Unknown"))
            .collect();

        format!("[{}]", args_names.join(", "))
    }
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
    use ast::lexer::get_subslice_span;
    

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
    let config = term::Config::default();

    // Emit each error directly
    for err in err_list {
        emit_error(err, file_id, &mut buffer, &config, &files, table);
    }

    // Print the full diagnostic report
    println!("{}", String::from_utf8(buffer.into_inner()).unwrap());
}

fn emit_error(
    err: &Error,
    file_id: usize,
    buffer: &mut Buffer,
    config: &term::Config,
    files: &SimpleFiles<&str, &str>,
    table: &StringTable,
) {
    let diagnostic = match err {
        Error::Match(m_err) => Diagnostic::error()
            .with_message("Match error")
            .with_labels(vec![
                Label::primary(file_id, m_err.span.start().to_usize()..m_err.span.end().to_usize())
                    .with_message("match error: could not find a case"),
            ]),

        Error::Sig(_s_err) => Diagnostic::error()
            .with_message("Signature error: ___"),

        Error::Missing(UndefinedName { id }) => Diagnostic::error()
            .with_message(format!(
                "Undefined name error: {}",
                table.get_display_str(*id).unwrap_or("Unknown name")
            )),

        Error::MissingCall(s) => Diagnostic::error()
            .with_message(format!(
                "Attempted to call non existent function: {s}",
                
            )),

        Error::UnreachableCase(  sig ) => {
            let name_str = table.get_display_str(sig.name).unwrap_or("Unknown name");
            let sig_display = sig.display_with_table(table);
            Diagnostic::error()
                .with_message(format!(
                    "The function '{}' with signature {} is defined more than once.",
                    name_str, sig_display
                ))
        },

        Error::NoneCallble(NoneCallble{span,value}) => Diagnostic::error()
            .with_message("Attempted to call a None Callble object")
            .with_labels(vec![
                Label::primary(file_id, span.start().to_usize()..span.end().to_usize())
                    .with_message(format!("this = {}",value)),
            ]),
            

        Error::Stacked(InternalError { span, err,message }) => {
            let diagnostic = Diagnostic::error().with_message(*message).with_labels(vec![
                Label::primary(file_id, span.start().to_usize()..span.end().to_usize())
                    ,
            ]);
            term::emit(buffer, config, files, &diagnostic).unwrap();


            // Emit each error inside `Error::Stacked` recursively
            for e in err {
                emit_error(e, file_id, buffer, config, files, table);
            }

            return;
        },

         Error::StackedTail(InternalError { span, err,message }) => {
            let diagnostic = Diagnostic::error().with_message(*message).with_labels(vec![
                Label::primary(file_id, span.start().to_usize()..span.end().to_usize())
                    ,
            ]).with_notes(vec![
                "Note: some self recursive calls may be missing due to tail call optimization".to_string()
            ]);
            term::emit(buffer, config, files, &diagnostic).unwrap();


            // Emit each error inside `Error::Stacked` recursively
            for e in err {
                emit_error(e, file_id, buffer, config, files, table);
            }

            return;
        },

        Error::IllegalSelfRef(IllegalSelfRef{span}) => {
            let error = Diagnostic::error()
                .with_message("Illegal use of Self")
                .with_labels(vec![
                        Label::primary(file_id, span.start().to_usize()..span.end().to_usize())
                            .with_message("this self ref creates a cycle"),
                    ]);
            term::emit(buffer, config, files, &error).unwrap();
            Diagnostic::help().with_message("try refering to the function by name (define it with def)")

        },
        Error::Recursion(RecursionError { depth }) => Diagnostic::error()
            .with_message(format!("Recursion Depth of {} reached", depth))
            .with_notes(vec![
                "This is likely caused by an infinite loop".to_string(),
            ]),
        Error::Bug(message) => {
            let diagnostic = Diagnostic::error()
                .with_message("An internal bug has occurred.")
                .with_notes(vec![
                    format!("Details: {}", message),
                ]);
            term::emit(buffer, config, files, &diagnostic).unwrap();
            Diagnostic::help().with_message("This is not your fault, but rather an implementation bug. Please report this to the maintainers.")
        },
        Error::StackOverflow => {
            let diagnostic = Diagnostic::error()
                .with_message("StackOverflow");
            term::emit(buffer, config, files, &diagnostic).unwrap();


            Diagnostic::help().with_message("probably caused by an infinite loop or excessive memory consumbtion")
        },

        Error::ZeroDiv => Diagnostic::error()
            .with_message("attempted to divide by zero"),
    };

    // Emit the diagnostic for the current error
    term::emit(buffer, config, files, &diagnostic).unwrap();
}

// fn add_error


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
    let unreachable_case_err = Error::UnreachableCase(
        
        FuncSig {name: unreachable_name_id,args:vec![undef_id]}, // Assuming FuncSig has a default or placeholder
    );

    err_list.push_back(match_err);
    err_list.push_back(sig_err);
    err_list.push_back(undef_err);
    err_list.push_back(unreachable_case_err);

    // Report the errors
    report_err_list(&err_list, source, &table);
}


#[test]
fn test_err_list_reporting_with_stacking() {
    
    let mut table = StringTable::new();
    let undef_id = table.get_id("baba");
    let unreachable_name_id = table.get_id("foo");

    let source = "fn main() { let x = 5; baba }";
    let mut err_list = LinkedList::new();

    // Example errors for testing
    let match_err = Error::Match(MatchError {
        span: Span::new(ByteIndex(3), ByteIndex(7)),
    });

    let sig_err = Error::Sig(SigError {});
    let _undef_err = Error::Missing(UndefinedName { id: undef_id });

    // Simulate unreachable case
    let _unreachable_case_err = Error::UnreachableCase(
        // name: unreachable_name_id,
         FuncSig { name: unreachable_name_id, args: vec![undef_id] },
    );

    // Add an internal error wrapped inside another error (stacked errors)
    let internal_err = Error::Stacked(InternalError {
        span: Span::new(ByteIndex(23), ByteIndex(27)),
        message: "junk",
        err: vec_to_list(vec![
            Error::Missing(UndefinedName { id: undef_id }),
            Error::UnreachableCase(
                
                FuncSig {name: unreachable_name_id, args: vec![undef_id] },
            ),

        ]),
    });

    // Add errors to the list
    err_list.push_back(match_err);
    err_list.push_back(sig_err);
    err_list.push_back(internal_err);

    // Report the errors
    report_err_list(&err_list, source, &table);
}

// You would need to implement `report_err_list` that iterates over `err_list` and calls `add_to_error` for each error, creating a complete diagnostic report.
