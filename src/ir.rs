#![allow(dead_code)]
use std::collections::HashMap;
use std::rc::Rc;
use codespan::Span;

pub use crate::basic_ops::{is_equal};

//this is used within functions. 
pub struct VarScope<'parent> {
	parent : Option<&'parent VarScope<'parent>>,
	vars : HashMap<usize,Value>
}

impl<'parent> VarScope<'parent>  {
	pub fn new() -> Self {
		VarScope{
			parent:None,
			vars:HashMap::new()
		}
	}
	pub fn make_subscope<'a>(&'a self) -> VarScope<'a> {
		VarScope{
			parent:Some(self),
			vars:HashMap::new()
		}
	}

	pub fn add(&mut self,id : usize , val: Value) {
		self.vars.insert(id,val);
	}

	pub fn get(&self,id:usize) -> Option<&Value> {
		match self.vars.get(&id) {
			Some(v) => Some(v),
			None => match self.parent {
				None => None,
				Some(p) => p.get(id),
			}
		}
	}
}

#[test]
fn test_scope_lifetimes(){
	let g = VarScope::new();
	let mut a = g.make_subscope();
	let _b = g.make_subscope();
	{
		let _c = a.make_subscope();
	}
	let _d = &mut a;
}

#[derive(Debug, PartialEq, Clone)]
pub enum GenericRet<T> {
    //holds the return value marks what to do with the call stack
    //this is shared between match statments and functions
    //HOWEVER only matchstatments may return the Unwind varient

    Local(T),
    Unwind(T),
}

impl<T> GenericRet<T> {
    pub fn as_local(self) -> Self {
        match self {
            GenericRet::Local(v) => GenericRet::Local(v),
            GenericRet::Unwind(v) => GenericRet::Local(v),
        }
    }
    pub fn into(self) -> T {
        match self {
            GenericRet::Local(v) | GenericRet::Unwind(v) => v,
        }
    }
}

// Define the types for ScopeRet and BlockReturn using the generic enum

pub type ScopeRet = GenericRet<LazyVal>;
pub type BlockReturn = GenericRet<Value>;

impl From<Value> for BlockReturn {
    fn from(value: Value) -> Self {
        BlockReturn::Local(value)
    }
}

pub type GcPointer<T> = Rc<T>;


#[derive(Debug,PartialEq,Clone)]
pub enum LazyVal{
    //this represents a value before computation
    //however early return complicate things 
    //so we can actually return from the parent scope
    //sometimes evaluating a var causes an excepsion


    Terminal(Value),
    Ref(usize),
    FuncCall(Call),
    //this is scoped for now but in some cases that makes no sense
    Match{var: Box<LazyVal>,statment: MatchStatment}
}

impl LazyVal {
    pub fn eval(&self,scope: &VarScope) -> Result<BlockReturn,Error> {
        match self {
            LazyVal::Terminal(v) => Ok(v.clone().into()),
            LazyVal::Ref(id) => match scope.get(*id) {
                None => Err(Error::Missing(UndefinedName{})),
                Some(v) => Ok(v.clone().into()),
            },
            LazyVal::FuncCall(call) => call.eval(scope),
            LazyVal::Match { var, statment } => 

                match var.eval(scope)? {    
                    BlockReturn::Unwind(v) => Ok(BlockReturn::Unwind(v)),
                    BlockReturn::Local(v) => statment.eval(v, scope),
                }

        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Statment {
    Assign(usize, LazyVal),
    Call(Call),
    Match(MatchStatment),
}


#[derive(Debug,PartialEq,Clone)]
pub struct Func {
    sig:FuncSig,
    inner:Block,
}

#[derive(Debug,PartialEq,Clone)]
pub struct FuncSig{
    num_args: usize, //for now its all simple
}


#[derive(Debug,PartialEq,Clone)]
pub enum Block{
    Simple(LazyVal),
    Code{inner:Vec<Statment>,ret:ScopeRet},
}

impl Block {
    pub fn eval(&self,scope: &VarScope) -> Result<BlockReturn,Error>{
        match self {
            Block::Simple(v) => v.eval(scope),
            _=>todo!(),
        }
    }
}



#[derive(Debug,PartialEq,Clone)]
pub enum MatchCond {
    Literal(Value),
    Any
}

impl MatchCond {
    pub fn matches(&self,v: &Value) -> bool{
        match self {
            MatchCond::Any => true,
            MatchCond::Literal(x) => is_equal(v,x),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct MatchStatment {
    arms: Vec<MatchCond>,
    vals: Vec<Block>,
    debug_span: Span,
}

impl MatchStatment {
    pub fn eval(&self, x:Value ,scope: &VarScope) -> Result<BlockReturn,Error> {
        for (i,a) in self.arms.iter().enumerate() {
            if a.matches(&x) {
                return self.vals[i].eval(scope);
            }
        }
        return Err(Error::Match(MatchError{span:self.debug_span}));
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum FunctionHandle{
    FFI(fn(Vec<Value>)->Result<Value,Error>),
	StaticDef(&'static Func),
    MatchLambda(GcPointer<MatchStatment>),
    Lambda(GcPointer<Func>),
}

impl FunctionHandle{
    pub fn eval(&self,args: Vec<Value>) -> Result<Value,Error> {
        match *self {
            FunctionHandle::FFI(f) => f(args),
            _=>todo!()
        }
        
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct Call{
    //handle:Box<FunctionHandle>,
    called: Box<LazyVal>,
    args: Vec<LazyVal>
}

impl Call {
    pub fn eval(&self,scope: &VarScope) -> Result<BlockReturn,Error> {
        let mut arg_values = Vec::with_capacity(self.args.len());
        let handle = match self.called.eval(scope)? {
            BlockReturn::Unwind(v) => {return Ok(BlockReturn::Unwind(v));}
            BlockReturn::Local(v) => match v {
                Value::Func(f) => f,
                _ => {return Err(Error::NoneCallble(NoneCallble{}));}
            }
        };

        for a in self.args.iter() {
            match a.eval(scope) {
                Err(e) => {return Err(e);},
                Ok(x) => {match x {
                    BlockReturn::Local(v) => {arg_values.push(v);},
                    BlockReturn::Unwind(_) => {return Ok(x);},
                }}
            };
        }
        handle.eval(arg_values).map(|v| v.into())
    }
}


#[derive(Debug,PartialEq,Clone)]
pub enum Value {
	Nil,
	Bool(bool),
	Atom(usize),
	String(GcPointer<String>),
	Int(i64),
	Float(f64),
	Func(FunctionHandle),
}

#[derive(Debug,PartialEq)]
pub enum Error {
    Match(MatchError),
    Sig(SigError),
    Missing(UndefinedName),
    NoneCallble(NoneCallble)
    //UndocumentedError,
}

#[derive(Debug,PartialEq)]
pub struct MatchError {
    span: Span
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

use std::cell::RefCell;

#[test]
fn test_system_println() {
    // Define a static mutable log buffer, scoped within this module for the test
    thread_local! {
        static LOG_PRINT: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
        static LOG_SYSTEM: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
    }

    // Inlined private function for `println`
    fn ffi_println(args: Vec<Value>) -> Result<Value, Error> {
        // Log the arguments into the static log buffer for println
        LOG_PRINT.with(|log| {
            log.borrow_mut().push(args.clone());
        });
        Ok(Value::Nil)
    }

    // Inlined private function for `system`
    fn ffi_system(args: Vec<Value>) -> Result<Value, Error> {
        // Log the arguments into the static log buffer for system
        LOG_SYSTEM.with(|log| {
            log.borrow_mut().push(args.clone());
        });

        // Return a function pointer to `ffi_println`
        Ok(Value::Func(FunctionHandle::FFI(ffi_println)))
    }

    // Create a scope and add the system function as a variable
    let mut scope = VarScope::new();
    let system_var = 1; // Mock ID for system variable
    scope.add(system_var, Value::Func(FunctionHandle::FFI(ffi_system)));

    // Inner call for accessing the `system` variable from the scope
    let system_call = Call {
        called: Box::new(LazyVal::Ref(system_var)), // Accessing system from the scope
        args: vec![LazyVal::Terminal(Value::Atom(2))], // Argument is :println (Atom with ID 2)
    };

    // Outer call for `system(:println)("hello world")`
    let outer_call = Call {
        called: Box::new(LazyVal::FuncCall(system_call)), // The function returned by system_call (println)
        args: vec![LazyVal::Terminal(Value::String(Rc::new("hello world".to_string())))], // Argument to println
    };

    // Evaluate the outer call (first system, then println)
    let ans = outer_call.eval(&scope).unwrap();

    // Check that system was called once with the correct argument (Atom 2 for :println)
    LOG_SYSTEM.with(|log| {
        assert_eq!(log.borrow().len(), 1);
        assert_eq!(log.borrow()[0], vec![Value::Atom(2)]); // :println is Atom(2)
    });

    // Check that println was called once with the correct argument (GcPointer<String> with "hello world")
    LOG_PRINT.with(|log| {
        assert_eq!(log.borrow().len(), 1);
        assert_eq!(log.borrow()[0], vec![Value::String(Rc::new("hello world".to_string()))]);
    });
}
