use std::collections::HashMap;
use codespan::Span;
//names are represented as a usize which is a key into our table names

#[derive(Debug,PartialEq)]
pub enum Statment {
    Assign(usize, Value),
    Call(FunctionCall),
    Match(MatchStatment), // New case for match statements
}



#[derive(Debug,PartialEq)]
pub struct FuncDec {
    pub sig: FuncSig,
    pub body: FuncBlock,
}

#[derive(Debug,PartialEq)]
pub struct FuncSig {
    pub name: usize,     // Function name ID from the StringTable
    pub args: Vec<usize>, // names of args
}

#[derive(Debug,PartialEq)]
pub struct FuncBlock{
    pub body: Vec<Statment>, 
    pub ret: Option<Ret>,
}

#[derive(Debug,PartialEq)]
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

#[derive(Debug,PartialEq)]
pub struct Lambda {
    pub sig: Vec<usize>,
    pub body: FuncBlock,
}

#[derive(Debug,PartialEq)]
pub struct FunctionCall {
    pub name: FValue,     //
    pub args: Vec<Value>, // Arguments to the function call
    pub debug_span: Span,
}

#[derive(Debug,PartialEq)]
pub enum FValue {
    Name(usize),
    FuncCall(Box<FunctionCall>),
    Lambda(Box<Lambda>),
    MatchLambda(Box<MatchLambda>),
    BuildIn(BuildIn),
}

#[derive(Debug,PartialEq)]
pub enum Value {
    Int(Result<i64, f64>),
    Float(f64),
    Bool(bool),
    Atom(usize),
    String(usize),
    Variable(usize),
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
            FValue::BuildIn(build_in) => Value::BuildIn(build_in)
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum Literal {
    Int(Result<i64,f64>),
    Float(f64),
    Atom(usize),
    String(usize),
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



#[derive(Debug,PartialEq)]
pub enum MatchPattern {
    Literal(Literal), 
    Variable(usize),   
    Wildcard,          // The `_` pattern
    //Tuple(Vec<MatchPattern>), // Matching a tuple
}


#[derive(Debug,PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern, // The pattern to match
    pub result: MatchOut, // Result of the match arm (a Value or a block)
}


// Result type for match arm
#[derive(Debug,PartialEq)]
pub enum MatchOut {
    Value(Value),
    Block(FuncBlock),
}




#[derive(Debug,PartialEq)]
pub struct MatchStatment {
    pub val: Box<Value>,      // The expression being matched
    pub arms: Vec<MatchArm>,  // The match arms
    pub debug_span: Span,
}

//these are expressions that look like match fn {...} 
//and they make a lamda function that pattrn matches arguments like a regular funtion
//the intended use is for things like arrays with  arr = match fn {0 => a, 1 => y}; arr(0)==a; 
#[derive(Debug,PartialEq)]
pub struct MatchLambda {
    pub arms: Vec<MatchArm>,  
    pub debug_span: Span,
}


#[derive(Debug,PartialEq)]
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

#[derive(Debug,PartialEq)]
pub struct ImportFunc{
    pub path: usize,
    pub name: usize,
}

#[derive(Debug,PartialEq)]
pub enum OuterExp {
    ImportFunc(ImportFunc),
    FuncDec(FuncDec),
}

#[derive(Debug,PartialEq)]
pub struct StringTable<'input> {
    map: HashMap<&'input str, usize>,
    vec: Vec<&'input str>,
}

impl<'input> StringTable<'input> {
    pub fn new() -> Self {
        let mut table = Self {
            map: HashMap::new(),
            vec: Vec::new(),
        };

        // Preload the basic atoms
        table.get_id("main");
        table.get_id("system");

        table.get_id("_");       
        
        table.get_id(":nil");
        table.get_id(":bool");
        table.get_id(":string");
        table.get_id(":int");
        table.get_id(":float");
        table.get_id(":atom");
        table.get_id(":func");
        
        table
    }

    // Returns the ID of the string, inserting it if it doesn't exist.
    pub fn get_id(&mut self, s: &'input str) -> usize {
        if let Some(&id) = self.map.get(s) {
            id
        } else {
            let id = self.vec.len();
            self.vec.push(s);
            self.map.insert(s, id);
            id
        }
    }
    pub fn get_existing_id(&self, s: &'input str) -> usize {
        self.map[s]
    }

    // Returns the string corresponding to an ID, or an error if the ID is out of bounds.
    pub fn get_string(&self, id: usize) -> Option<&'input str> {
        self.vec.get(id).copied()
    }

    pub fn compare_to(&self, id: usize,s: &str) -> bool {
        match self.get_string(id) {
            None => unreachable!("attempting to compare to a non existing entry"),
            Some(x) => x==s,
        }
    }
}

#[test]
fn test_string_table() {
    let input = "hello world";
    let mut table = StringTable::new();

    let id_hello = table.get_id(&input[0..5]);
    let id_world = table.get_id(&input[6..11]);


    // Check that we can retrieve "hello" by its ID
    let retrieved_hello = table.get_string(id_hello).unwrap();
    assert_eq!(retrieved_hello, "hello");

    // Check that we can retrieve "world" by its ID
    let retrieved_world = table.get_string(id_world).unwrap();
    assert_eq!(retrieved_world, "world");

    // Ensure that inserting "hello" again returns the same ID
    let id_hello_again = table.get_id("hello");
    assert_eq!(id_hello_again, id_hello);
}
