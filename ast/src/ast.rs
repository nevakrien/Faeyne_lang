use unescape::unescape;
use std::collections::HashMap;
use codespan::Span;
use crate::id::*;
// use crate::system::preload_table;
//names are represented as a u32 which is a key into our table names

#[derive(Debug,PartialEq,Clone)]
pub enum Statment {
    Assign(u32, Value),
    Call(FunctionCall),
    Match(MatchStatment), // New case for match statements
}



#[derive(Debug,PartialEq,Clone)]
pub struct FuncDec {
    pub sig: FuncSig,
    pub body: FuncBlock,
}

#[derive(Debug,PartialEq,Clone)]
pub struct FuncSig {
    pub name: u32,     // Function name ID from the StringTable
    pub args: Vec<u32>, // names of args
}

#[derive(Debug,PartialEq,Clone)]
pub struct FuncBlock{
    pub body: Vec<Statment>, 
    pub ret: Option<Ret>,
}

#[derive(Debug,PartialEq,Clone)]
pub enum Ret{
    Imp(Value),
    Exp(Value),
}

impl Ret {
    pub fn get_value(&self) -> &Value {
        match self {
            Ret::Imp(v) => v,
            Ret::Exp(v) => v,
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Lambda {
    pub sig: Vec<u32>,
    pub body: FuncBlock,
    pub debug_span: Span,
}

#[derive(Debug,PartialEq,Clone)]
pub struct FunctionCall {
    pub name: FValue,     //
    pub args: Vec<Value>, // Arguments to the function call
    pub debug_span: Span,
}

#[derive(Debug,PartialEq,Clone)]
pub enum FValue {
    SelfRef(Span),
    Name(u32),
    FuncCall(Box<FunctionCall>),
    Lambda(Box<Lambda>),
    MatchLambda(Box<MatchLambda>),
    BuildIn(BuildIn),
}

#[derive(Debug,PartialEq,Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Atom(u32),
    String(u32),
    Variable(u32),
    SelfRef(Span),
    FuncCall(FunctionCall),
    Lambda(Box<Lambda>),
    MatchLambda(Box<MatchLambda>),
    BuildIn(BuildIn),
    Nil,
    Match(MatchStatment),
}

impl From<FValue> for Value {
    fn from(fval: FValue) -> Self {
        match fval {
            FValue::Name(name) => Value::Variable(name),
            FValue::FuncCall(func_call) => Value::FuncCall(*func_call),
            FValue::Lambda(lam) => Value::Lambda(lam),
            FValue::MatchLambda(m) => Value::MatchLambda(m),
            FValue::BuildIn(build_in) => Value::BuildIn(build_in),
            FValue::SelfRef(span) => Value::SelfRef(span),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Atom(u32),
    String(u32),
    Bool(bool),
    Nil,
}
impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Int(i) => Value::Int(i),
            Literal::Float(f) => Value::Float(f),
            Literal::Atom(a) => Value::Atom(a),
            Literal::String(s) => Value::String(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Nil => Value::Nil,
        }
    }
}



#[derive(Debug,PartialEq,Clone)]
pub enum MatchPattern {
    Literal(Literal), 
    Variable(u32),   
    Wildcard,          // The `_` pattern
    //Tuple(Vec<MatchPattern>), // Matching a tuple
}


#[derive(Debug,PartialEq,Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern, // The pattern to match
    pub result: MatchOut, // Result of the match arm (a Value or a block)
}


// Result type for match arm
#[derive(Debug,PartialEq,Clone)]
pub enum MatchOut {
    Value(Value),
    Block(FuncBlock),
}




#[derive(Debug,PartialEq,Clone)]
pub struct MatchStatment {
    pub val: Box<Value>,      // The expression being matched
    pub arms: Vec<MatchArm>,  // The match arms
    pub debug_span: Span,
}

//these are expressions that look like match fn {...} 
//and they make a lamda function that pattrn matches arguments like a regular funtion
//the intended use is for things like arrays with  arr = match fn {0 => a, 1 => y}; arr(0)==a; 
#[derive(Debug,PartialEq,Clone)]
pub struct MatchLambda {
    pub arms: Vec<MatchArm>,  
    pub debug_span: Span,
}


#[derive(Debug,PartialEq,Clone,Copy)]
pub enum BuildIn {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Modulo,
    Pow,

    Equal,
    NotEqual,
    Smaller,
    Bigger,
    SmallerEq,
    BiggerEq,

    And,
    Or,
    Xor,

    DoubleAnd,
    DoubleOr,
    DoubleXor,
}

#[derive(Debug,PartialEq,Clone)]
pub struct ImportFunc{
    pub path: u32,
    pub name: u32,
}

#[derive(Debug,PartialEq,Clone)]
pub enum OuterExp {
    ImportFunc(ImportFunc),
    FuncDec(FuncDec),
}

#[derive(Debug,PartialEq,Clone)]
pub struct StringTable<'input> {
    map: HashMap<&'input str, u32>,
    vec: Vec<&'input str>,
}



#[allow(clippy::new_without_default)]
impl<'input> StringTable<'input> {
    pub fn new() -> Self {
        let mut table = Self {
            map: HashMap::new(),
            vec: Vec::new(),
        };

        preload_table(&mut table);
        
        table
    }

    // Returns the ID of the string, inserting it if it doesn't exist.
    pub fn get_id(&mut self, s: &'input str) -> u32 {
        if let Some(&id) = self.map.get(s) {
            id
        } else {
            let id = self.vec.len() as u32;
            self.vec.push(s);
            self.map.insert(s, id);
            id
        }
    }

    // Returns the ID of the string, inserting it if it doesn't exist.
    pub fn check_id(&self, s: &'input str) -> Option<u32> {
        if let Some(&id) = self.map.get(s) {
            Some(id)
        } else {
            None
        }
    }

    pub fn get_existing_id(&self, s: &'input str) -> u32 {
        self.map[s]
    }

    // Returns the string corresponding to an ID, or an error if the ID is out of bounds.
    pub fn get_raw_str(&self, id: u32) -> &'input str {
        self.vec.get(id as usize).copied().unwrap()
    }

    // Returns the string corresponding to an ID, or an error if the ID is out of bounds.
    pub fn get_display_str(&self, id: u32) -> Option<&'input str> {
        self.vec.get(id as usize).copied()
    }

    // Returns the string corresponding to an ID, or an error if the ID is out of bounds.
    pub fn get_escaped_string(&self, id: u32) -> String {
        self.vec.get(id as usize).map(|r|unescape(&r[1..r.len()-1]).unwrap()).unwrap()
    }

}



#[test]
fn test_string_table() {
    let input = "hello world";
    let mut table = StringTable::new();

    let id_hello = table.get_id(&input[0..5]);
    let id_world = table.get_id(&input[6..11]);


    // Check that we can retrieve "hello" by its ID
    let retrieved_hello = table.get_raw_str(id_hello);
    assert_eq!(retrieved_hello, "hello");

    // Check that we can retrieve "world" by its ID
    let retrieved_world = table.get_raw_str(id_world);
    assert_eq!(retrieved_world, "world");

    // Ensure that inserting "hello" again returns the same ID
    let id_hello_again = table.get_id("hello");
    assert_eq!(id_hello_again, id_hello);
}

#[test]
fn test_string_table_unescape() {
    let input = "\"hello world\\n\"";
    let mut table = StringTable::new();

    let id = table.get_id(input);//&input[1..14]


    // Check that we can retrieve "hello" by its ID
    let retrieved_hello = table.get_escaped_string(id);
    assert_eq!(retrieved_hello, "hello world\n");
}