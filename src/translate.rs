use crate::ast::*;
use crate::ir;
use crate::ir::{VarScope,StaticVarScope,LazyVal};
use crate::basic_ops::get_buildin_function;

pub trait Translate<Output> {
    fn translate(&self, table: &StringTable) -> Output;
}

impl Translate<LazyVal> for Value {
	fn translate(&self, table: &StringTable) -> LazyVal {
		match self {
			Value::Variable(id) => LazyVal::Ref(*id),
			Value::Int(x) => LazyVal::Terminal(ir::Value::Int(*x)),
			Value::Float(x) => LazyVal::Terminal(ir::Value::Float(*x)),
			Value::Bool(x) => LazyVal::Terminal(ir::Value::Bool(*x)),
			Value::Atom(x) => LazyVal::Terminal(ir::Value::Atom(*x)),

			Value::Nil => LazyVal::Terminal(ir::Value::Nil),
			Value::BuildIn(op) => LazyVal::Terminal(
				ir::Value::Func(
				ir::FunctionHandle::FFI(
					get_buildin_function(*op)
				)
			)),

			_ => todo!()
		}
	}
}