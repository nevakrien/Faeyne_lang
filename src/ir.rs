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

pub type ScopeRet = GenericRet<MVar>;
pub type BlockReturn = GenericRet<Value>;

impl From<Value> for BlockReturn {
    fn from(value: Value) -> Self {
        BlockReturn::Local(value)
    }
}

pub type GcPointer<T> = Rc<T>;


#[derive(Debug,PartialEq,Clone)]
pub enum MVar{
    //this represents a value before computation
    //however early return complicate things 
    //so we can actually return from the parent scope
    //sometimes evaluating a var causes an excepsion


    Terminal(Value),
    Ref(usize),
    FuncCall(Call),
    //this is scoped for now but in some cases that makes no sense
    Match{var: Box<MVar>,statment: MatchStatment}
}

impl MVar {
    pub fn eval(&self,scope: &VarScope) -> Result<BlockReturn,Error> {
        match self {
            MVar::Terminal(v) => Ok(v.clone().into()),
            MVar::Ref(id) => match scope.get(*id) {
                None => Err(Error::Missing(UndefinedName{})),
                Some(v) => Ok(v.clone().into()),
            },
            MVar::FuncCall(call) => call.eval(scope),
            MVar::Match { var, statment } => 

                match var.eval(scope)? {    
                    BlockReturn::Unwind(v) => Ok(BlockReturn::Unwind(v)),
                    BlockReturn::Local(v) => statment.eval(v, scope),
                }

        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Statment {
    Assign(usize, MVar),
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
    Simple(MVar),
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
    handle:Box<FunctionHandle>,
    args: Vec<MVar>
}

impl Call {
    pub fn eval(&self,scope: &VarScope) -> Result<BlockReturn,Error> {
        let mut arg_values = Vec::with_capacity(self.args.len());
        for a in self.args.iter() {
            match a.eval(scope) {
                Err(e) => {return Err(e);},
                Ok(x) => {match x {
                    BlockReturn::Local(v) => {arg_values.push(v);},
                    BlockReturn::Unwind(_) => {return Ok(x);},
                }}
            };
        }
        self.handle.eval(arg_values).map(|v| v.into())
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
pub struct UndefinedName {
    //placeholder
}

