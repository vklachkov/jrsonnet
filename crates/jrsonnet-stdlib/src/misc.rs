use std::{cell::RefCell, rc::Rc};

use jrsonnet_evaluator::{
	error::{Error::*, Result},
	function::{builtin, ArgLike, CallLocation, FuncVal},
	throw,
	typed::{Any, Either2, Either4},
	val::{equals, ArrValue},
	Either, IStr, ObjValue, State, Thunk, Val,
};

use crate::{extvar_source, Settings};

#[builtin]
pub fn builtin_length(x: Either![IStr, ArrValue, ObjValue, FuncVal]) -> Result<usize> {
	use Either4::*;
	Ok(match x {
		A(x) => x.chars().count(),
		B(x) => x.len(),
		C(x) => x.len(),
		D(f) => f.params_len(),
	})
}

#[builtin(fields(
	settings: Rc<RefCell<Settings>>,
))]
pub fn builtin_ext_var(this: &builtin_ext_var, s: State, x: IStr) -> Result<Any> {
	let ctx = s.create_default_context(extvar_source(&x, ""));
	Ok(Any(this
		.settings
		.borrow()
		.ext_vars
		.get(&x)
		.cloned()
		.ok_or_else(|| UndefinedExternalVariable(x))?
		.evaluate_arg(s.clone(), ctx, true)?
		.evaluate(s)?))
}

#[builtin(fields(
	settings: Rc<RefCell<Settings>>,
))]
pub fn builtin_native(this: &builtin_native, name: IStr) -> Result<Any> {
	Ok(Any(this
		.settings
		.borrow()
		.ext_natives
		.get(&name)
		.cloned()
		.map_or(Val::Null, |v| {
			Val::Func(FuncVal::Builtin(v.clone()))
		})))
}

#[builtin(fields(
	settings: Rc<RefCell<Settings>>,
))]
pub fn builtin_trace(
	this: &builtin_trace,
	s: State,
	loc: CallLocation,
	str: IStr,
	rest: Thunk<Val>,
) -> Result<Any> {
	this.settings
		.borrow()
		.trace_printer
		.print_trace(s.clone(), loc, str);
	Ok(Any(rest.evaluate(s)?))
}

#[allow(clippy::comparison_chain)]
#[builtin]
pub fn builtin_starts_with(
	s: State,
	a: Either![IStr, ArrValue],
	b: Either![IStr, ArrValue],
) -> Result<bool> {
	Ok(match (a, b) {
		(Either2::A(a), Either2::A(b)) => a.starts_with(b.as_str()),
		(Either2::B(a), Either2::B(b)) => {
			if b.len() > a.len() {
				return Ok(false);
			} else if b.len() == a.len() {
				return equals(s, &Val::Arr(a), &Val::Arr(b));
			} else {
				for (a, b) in a
					.slice(None, Some(b.len()), None)
					.iter(s.clone())
					.zip(b.iter(s.clone()))
				{
					let a = a?;
					let b = b?;
					if !equals(s.clone(), &a, &b)? {
						return Ok(false);
					}
				}
				true
			}
		}
		_ => throw!("both arguments should be of the same type"),
	})
}

#[allow(clippy::comparison_chain)]
#[builtin]
pub fn builtin_ends_with(
	s: State,
	a: Either![IStr, ArrValue],
	b: Either![IStr, ArrValue],
) -> Result<bool> {
	Ok(match (a, b) {
		(Either2::A(a), Either2::A(b)) => a.ends_with(b.as_str()),
		(Either2::B(a), Either2::B(b)) => {
			if b.len() > a.len() {
				return Ok(false);
			} else if b.len() == a.len() {
				return equals(s, &Val::Arr(a), &Val::Arr(b));
			} else {
				let a_len = a.len();
				for (a, b) in a
					.slice(Some(a_len - b.len()), None, None)
					.iter(s.clone())
					.zip(b.iter(s.clone()))
				{
					let a = a?;
					let b = b?;
					if !equals(s.clone(), &a, &b)? {
						return Ok(false);
					}
				}
				true
			}
		}
		_ => throw!("both arguments should be of the same type"),
	})
}
