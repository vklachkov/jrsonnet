#![allow(clippy::type_complexity)]

use std::{path::Path, rc::Rc};

use gcmodule::Trace;

use crate::{
	error::Result,
	function::{parse_builtin_call, ArgsLike, Builtin, BuiltinParam, CallLocation},
	gc::TraceBox,
	Context, State, Val,
};

#[derive(Trace)]
pub struct NativeCallback {
	pub(crate) params: Vec<BuiltinParam>,
	handler: TraceBox<dyn NativeCallbackHandler>,
}
impl NativeCallback {
	#[deprecated = "prefer using builtins directly, use this interface only for bindings"]
	pub fn new(params: Vec<BuiltinParam>, handler: TraceBox<dyn NativeCallbackHandler>) -> Self {
		Self { params, handler }
	}
}

impl Builtin for NativeCallback {
	fn name(&self) -> &str {
		// TODO: standard natives gets their names from definition
		// But builitins should already have them
		"<native>"
	}

	fn params(&self) -> &[BuiltinParam] {
		&self.params
	}

	fn call(&self, s: State, ctx: Context, loc: CallLocation, args: &dyn ArgsLike) -> Result<Val> {
		let args = parse_builtin_call(s.clone(), ctx, &self.params, args, true)?;
		let mut out_args = Vec::with_capacity(self.params.len());
		for p in &self.params {
			out_args.push(args[&p.name].evaluate(s.clone())?);
		}
		self.handler.call(s, loc.0.map(|l| l.0.clone()), &out_args)
	}
}

pub trait NativeCallbackHandler: Trace {
	fn call(&self, s: State, from: Option<Rc<Path>>, args: &[Val]) -> Result<Val>;
}
