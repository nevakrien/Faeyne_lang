#![allow(dead_code)]
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::LinkedList;
use crate::reporting::*;

use std::rc::Rc;
use codespan::Span;

pub type GcPointer<T> = Rc<T>;

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

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Scopble<'parent>{
    SubScope(&'parent VarScope<'parent>),
    Static(&'parent StaticScope),
}

impl<'parent> Scopble<'parent> {
    pub fn get(&self,n :usize) -> Option<&Value> {
        match self {
            Scopble::SubScope(s) => s.get(n),
            Scopble::Static(s) => s.get(n),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct StaticScope {
    vars : Vec<Option<Value>>,
    names: Vec<usize>,
}

impl StaticScope {
    pub fn new(names : Vec<usize>) -> Self {
        StaticScope{
            vars:vec![None;names.len()],
            names,
        }
    }

    pub fn get(&self,n :usize) -> Option<&Value> {
        self.get(n)
    }
}

#[derive(Debug,PartialEq,Clone)]
pub struct VarScope<'parent> {
    parent : Scopble<'parent>,//Option<&'parent VarScope<'parent>>,
    vars : Vec<Option<Value>>,
    names: Vec<usize>,
}

impl<'parent> VarScope<'parent>  {
    pub fn new(parent:Scopble<'parent>,names : Vec<usize>) -> Self {
        VarScope{
            parent,
            vars:vec![None;names.len()],
            names,
        }
    }

    pub fn get(&self,n :usize) -> Option<&Value> {
        if n < self.vars.len() {
            self.vars[n].as_ref()
        }
        else {
            self.parent.get(n+self.vars.len())
        }
    }
}   

#[derive(Debug,PartialEq,Clone)]
pub enum FunctionHandle{
    FFI(fn(Vec<Value>)->Result<Value,ErrList>),
    // StaticDef(&'static Func),
    // Lambda(GcPointer<Func>),
    
}

