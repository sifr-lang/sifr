use crate::{intrinsics, methods, RustEmitter, RustExpr};
use sifr_hir::{HirExpr, HirFStringPart};
use sifr_type_system::{ParamConvention, Type};

mod registry_helpers;
pub(crate) use registry_helpers::*;
mod builtin_core_methods;
mod builtin_numeric;
mod collection_methods;
mod literal_and_intrinsic_exprs;
mod narrowing_helpers;
mod plain_call_args;
mod recursive_exprs;
