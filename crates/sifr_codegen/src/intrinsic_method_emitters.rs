use crate::{RustEmitter, RustExpr, intrinsics, methods};
use sifr_ir::{HirExpr, HirFStringPart};
use sifr_type_system::{ParamConvention, Type};

mod registry_helpers;
pub(crate) use registry_helpers::*;
mod borrowing_call_args;
mod builtin_core_methods;
mod builtin_numeric;
mod collection_methods;
mod collection_type_resolution;
mod defaultdict_iterable_mutations;
mod literal_and_intrinsic_exprs;
mod narrowing_helpers;
mod plain_call_args;
mod recursive_exprs;
mod recursive_method_calls;
mod registry_method_arg_conventions;
