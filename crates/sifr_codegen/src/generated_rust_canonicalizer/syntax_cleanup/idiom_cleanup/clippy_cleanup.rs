use std::collections::{HashMap, HashSet};

use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

include!("clippy_cleanup/expression_cleanup.rs");
include!("clippy_cleanup/typed_value_cleanup.rs");
include!("clippy_cleanup/copy_iterator_cleanup.rs");
include!("clippy_cleanup/residual_call_cleanup.rs");
include!("clippy_cleanup/liveness_cleanup.rs");
include!("clippy_cleanup/condition_clone_cleanup.rs");
include!("clippy_cleanup/loop_cleanup.rs");
