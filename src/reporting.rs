use codespan::{ByteIndex, Span};
use std::collections::LinkedList;

#[derive(Debug,PartialEq)]
pub enum Error {
    Match(MatchError),
    Sig(SigError),
    Missing(UndefinedName),
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
pub struct UndefinedName {
    //placeholder
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

#[test]
fn test_subslice_span_and_diagnostic_reporting() {
    let source = "Hello, world!\nThis is a test.\nAnother line here.";
    
    // Define a subslice (we'll pretend it's an error in the source code)
    let subslice = &source[14..18]; // This is "This"
    
    // Use the utility function to get the Span
    let span = get_subslice_span(source, subslice);

    // Use codespan to map byte indices to line/column information
    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use codespan_reporting::term::{self, termcolor::{Buffer}};
    use codespan_reporting::files::SimpleFiles;

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
