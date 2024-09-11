use std::collections::HashMap;

//names are represented as a usize which is a key into our table names

#[derive(Debug)]
pub enum Statment {
    Assign(usize,Value),
    Call(FunctionCall),
}

#[derive(Debug)]
pub struct FuncDec {
    pub sig: FuncSig,
    pub body: FuncBlock,
}

#[derive(Debug)]
pub struct FuncSig {
    pub name: usize,     // Function name ID from the StringTable
    pub args: Vec<usize>, // names of args
}

#[derive(Debug)]
pub struct FuncBlock{
    pub body: Vec<Statment>, 
    pub ret: Option<Value>,
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: FValue,     //
    pub args: Vec<Value> // Arguments to the function call
}

#[derive(Debug)]
pub enum FValue {
    Name(usize),
    FuncCall(Box<FunctionCall>),
}

#[derive(Debug)]
pub enum Value {
    Int(Result<i64, f64>),
    Float(f64),
    Atom(usize),
    String(usize),
    Variable(usize),
    FuncCall(FunctionCall),
}

pub struct StringTable<'input> {
    map: HashMap<&'input str, usize>,
    vec: Vec<&'input str>,
}

impl<'input> StringTable<'input> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            vec: Vec::new(),
        }
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

    // Returns the string corresponding to an ID, or an error if the ID is out of bounds.
    pub fn get_string(&self, id: usize) -> Option<&'input str> {
        self.vec.get(id).copied()
    }
}

#[test]
fn test_string_table() {
    let input = "hello world";
    let mut table = StringTable::new();

    // Insert "hello" into the table and get its ID
    let id_hello = table.get_id(&input[0..5]);
    assert_eq!(id_hello, 0); // First insertion, ID should be 0

    // Insert "world" into the table and get its ID
    let id_world = table.get_id(&input[6..11]);
    assert_eq!(id_world, 1); // Second insertion, ID should be 1

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
