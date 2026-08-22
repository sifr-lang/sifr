//! Scope-based name resolution for Sifr.
//!
//! Supports type narrowing: variables can have a `narrowed_type` that
//! differs from their `declared_type` within control flow branches.

use num_bigint::BigInt;
use sifr_ir::BindingId;
use sifr_type_system::{OwnershipKind, ParamConvention, ReceiverConvention, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EphemeralOrigin {
    Iteration,
    Comprehension,
    MatchCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Local,
    ModuleConstant,
    Parameter,
    Receiver,
    EphemeralLocal(EphemeralOrigin),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingTypeSource {
    Explicit,
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ErrorTaint(());

impl ErrorTaint {
    pub(crate) const fn emitted() -> Self {
        Self(())
    }
}

/// Tracks variable state for ownership and narrowing.
#[derive(Debug, Clone)]
pub struct VarInfo {
    /// Stable identity for this binding.
    pub binding_id: BindingId,
    /// The declared type (from annotation or inference).
    pub ty: Type,
    /// The narrowed type (from control flow analysis). None means not narrowed.
    pub narrowed_type: Option<Type>,
    /// Whether the variable has been moved.
    pub is_moved: bool,
    /// Whether a specialized lowered representation requires move tracking
    /// even though its surface type is ordinarily copyable.
    requires_move_tracking: bool,
    /// Whether the variable is currently mutably borrowed.
    pub is_mut_borrowed: bool,
    /// Whether the binding itself is mutable.
    pub mutability: BindingMutability,
    /// Whether this binding originated from a function parameter.
    pub binding_kind: BindingKind,
    /// Parameter convention, when this binding is an ordinary parameter.
    pub parameter_convention: Option<ParamConvention>,
    /// Final receiver convention, when this binding is an instance receiver.
    pub receiver_convention: Option<ReceiverConvention>,
    /// Whether the binding type was provided explicitly (annotation/parameter).
    pub type_source: BindingTypeSource,
    /// Compile-time exact integer value known for this binding, if any.
    pub const_integer_value: Option<BigInt>,
    /// Proof that this binding only exists to suppress cascades after an emitted error.
    error_taint: Option<ErrorTaint>,
}

/// Immutable binding facts retained after the defining scope frame is popped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedBindingFact {
    pub name: String,
    pub ty: Type,
    pub binding_kind: BindingKind,
    pub mutability: BindingMutability,
    pub parameter_convention: Option<ParamConvention>,
    pub receiver_convention: Option<ReceiverConvention>,
}

impl VarInfo {
    /// Get the effective type (narrowed if available, otherwise declared).
    pub fn effective_type(&self) -> &Type {
        self.narrowed_type.as_ref().unwrap_or(&self.ty)
    }

    pub fn is_parameter_binding(&self) -> bool {
        matches!(self.binding_kind, BindingKind::Parameter)
    }

    pub fn is_mutable_binding(&self) -> bool {
        matches!(self.mutability, BindingMutability::Mutable)
    }

    pub fn is_inferred_local_binding(&self) -> bool {
        matches!(self.binding_kind, BindingKind::Local)
            && matches!(self.type_source, BindingTypeSource::Inferred)
    }

    pub fn is_poisoned_binding(&self) -> bool {
        self.error_taint.is_some()
    }

    pub(crate) fn error_taint(&self) -> Option<ErrorTaint> {
        self.error_taint
    }
}

/// A snapshot of the narrowing state for all variables in scope.
/// Used to save/restore narrowing at branch points.
pub type NarrowingSnapshot = Vec<(String, Option<Type>)>;

/// A snapshot of the moved state for all variables in scope.
/// Used to save/restore moved state at branch points and loop boundaries.
pub(crate) type MovedSnapshot = Vec<(String, bool)>;

/// A snapshot of known compile-time integer values for all variables in scope.
pub(crate) type ConstIntegerSnapshot = Vec<(String, Option<BigInt>)>;

/// A scope for name resolution.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Stack of scope frames (innermost last).
    frames: Vec<HashMap<String, VarInfo>>,
    /// Type alias registry.
    type_aliases: HashMap<String, Type>,
    /// Generic type alias registry: name -> (`type_params`, `body_type`).
    generic_type_aliases: HashMap<String, (Vec<String>, Type)>,
    next_binding_id: u32,
    retained_bindings: HashMap<BindingId, RetainedBindingFact>,
}

impl Scope {
    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        self.frames.iter().rev().find_map(|frame| frame.get(name))
    }

    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        self.frames
            .iter_mut()
            .rev()
            .find_map(|frame| frame.get_mut(name))
    }

    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
            type_aliases: HashMap::new(),
            generic_type_aliases: HashMap::new(),
            next_binding_id: 0,
            retained_bindings: HashMap::new(),
        }
    }

    /// Push a new scope frame (e.g., entering a function or block).
    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Pop the innermost scope frame.
    pub fn pop(&mut self) {
        self.frames.pop();
    }

    /// Return the current number of scope frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn visible_local_bindings(&self) -> Vec<(String, Type)> {
        let mut bindings = HashMap::new();
        for frame in self.frames.iter().skip(1) {
            for (name, info) in frame {
                bindings.insert(name.clone(), info.effective_type().clone());
            }
        }
        bindings.into_iter().collect()
    }

    pub(crate) fn resolves_to_module_binding(&self, name: &str) -> bool {
        self.frames
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, frame)| frame.contains_key(name).then_some(index == 0))
            .unwrap_or(false)
    }

    /// Define a variable in the current (innermost) scope.
    pub fn define(&mut self, name: String, ty: Type) {
        self.define_binding(name, ty, true, BindingKind::Local, false);
    }

    /// Define an immutable module-level value that codegen re-materializes on access.
    pub fn define_module_constant(&mut self, name: String, ty: Type) {
        self.define_binding(name, ty, false, BindingKind::ModuleConstant, true);
    }

    /// Define an explicitly-typed local variable in the current scope.
    pub fn define_explicit_local(&mut self, name: String, ty: Type) {
        self.define_binding(name, ty, true, BindingKind::Local, true);
    }

    /// Define a short-lived iteration, comprehension, or pattern binding.
    pub fn define_ephemeral(&mut self, name: String, ty: Type, origin: EphemeralOrigin) {
        self.define_binding(name, ty, true, BindingKind::EphemeralLocal(origin), false);
    }

    pub(crate) fn define_poisoned_local(
        &mut self,
        name: String,
        ty: Type,
        has_explicit_type: bool,
        error_taint: ErrorTaint,
    ) {
        self.define_binding_with_taint(
            name,
            ty,
            true,
            BindingKind::Local,
            has_explicit_type,
            Some(error_taint),
        );
    }

    /// Define a function parameter in the current (innermost) scope.
    pub fn define_parameter(&mut self, name: String, ty: Type, convention: ParamConvention) {
        self.define_binding_with_convention(
            name,
            ty,
            convention.is_mutable(),
            BindingKind::Parameter,
            true,
            Some(convention),
            None,
        );
    }

    /// Define the implicit receiver binding for a regular instance method.
    pub fn define_receiver(
        &mut self,
        name: String,
        ty: Type,
        convention: ReceiverConvention,
    ) -> BindingId {
        self.define_binding_with_convention(
            name,
            ty,
            convention.is_mutable(),
            BindingKind::Receiver,
            true,
            None,
            Some(convention),
        )
    }

    fn define_binding(
        &mut self,
        name: String,
        ty: Type,
        is_mutable_binding: bool,
        binding_kind: BindingKind,
        has_explicit_type: bool,
    ) {
        self.define_binding_with_taint(
            name,
            ty,
            is_mutable_binding,
            binding_kind,
            has_explicit_type,
            None,
        );
    }

    fn define_binding_with_taint(
        &mut self,
        name: String,
        ty: Type,
        is_mutable_binding: bool,
        binding_kind: BindingKind,
        has_explicit_type: bool,
        error_taint: Option<ErrorTaint>,
    ) -> BindingId {
        self.define_binding_with_convention_and_taint(
            name,
            ty,
            is_mutable_binding,
            binding_kind,
            has_explicit_type,
            None,
            None,
            error_taint,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn define_binding_with_convention(
        &mut self,
        name: String,
        ty: Type,
        is_mutable_binding: bool,
        binding_kind: BindingKind,
        has_explicit_type: bool,
        parameter_convention: Option<ParamConvention>,
        receiver_convention: Option<ReceiverConvention>,
    ) -> BindingId {
        self.define_binding_with_convention_and_taint(
            name,
            ty,
            is_mutable_binding,
            binding_kind,
            has_explicit_type,
            parameter_convention,
            receiver_convention,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn define_binding_with_convention_and_taint(
        &mut self,
        name: String,
        ty: Type,
        is_mutable_binding: bool,
        binding_kind: BindingKind,
        has_explicit_type: bool,
        parameter_convention: Option<ParamConvention>,
        receiver_convention: Option<ReceiverConvention>,
        error_taint: Option<ErrorTaint>,
    ) -> BindingId {
        let binding_id = BindingId(self.next_binding_id);
        assert!(
            self.next_binding_id < u32::MAX,
            "binding id space exhausted"
        );
        self.next_binding_id += 1;
        let mutability = if is_mutable_binding {
            BindingMutability::Mutable
        } else {
            BindingMutability::Immutable
        };
        self.retained_bindings.insert(
            binding_id,
            RetainedBindingFact {
                name: name.clone(),
                ty: ty.clone(),
                binding_kind,
                mutability,
                parameter_convention,
                receiver_convention,
            },
        );
        if let Some(frame) = self.frames.last_mut() {
            let type_source = if has_explicit_type {
                BindingTypeSource::Explicit
            } else {
                BindingTypeSource::Inferred
            };
            frame.insert(
                name,
                VarInfo {
                    binding_id,
                    ty,
                    narrowed_type: None,
                    is_moved: false,
                    requires_move_tracking: false,
                    is_mut_borrowed: false,
                    mutability,
                    binding_kind,
                    parameter_convention,
                    receiver_convention,
                    type_source,
                    const_integer_value: None,
                    error_taint,
                },
            );
        }
        binding_id
    }

    pub fn retained_binding(&self, id: BindingId) -> Option<&RetainedBindingFact> {
        self.retained_bindings.get(&id)
    }

    pub fn patch_receiver_convention(&mut self, id: BindingId, convention: ReceiverConvention) {
        if let Some(fact) = self.retained_bindings.get_mut(&id) {
            fact.receiver_convention = Some(convention);
            fact.mutability = if convention.is_mutable() {
                BindingMutability::Mutable
            } else {
                BindingMutability::Immutable
            };
        }
        for frame in &mut self.frames {
            for info in frame.values_mut() {
                if info.binding_id == id {
                    info.receiver_convention = Some(convention);
                    info.mutability = if convention.is_mutable() {
                        BindingMutability::Mutable
                    } else {
                        BindingMutability::Immutable
                    };
                }
            }
        }
    }

    /// Permit later lowering checks to continue after the declaration-site
    /// diagnostic for a receiver that requires explicit `mut` syntax.
    pub fn patch_receiver_mutability_for_recovery(&mut self, id: BindingId) {
        if let Some(fact) = self.retained_bindings.get_mut(&id) {
            fact.mutability = BindingMutability::Mutable;
        }
        for frame in &mut self.frames {
            for info in frame.values_mut() {
                if info.binding_id == id {
                    info.mutability = BindingMutability::Mutable;
                }
            }
        }
    }

    /// Update the declared type for an existing variable.
    pub fn set_type(&mut self, name: &str, ty: Type) -> bool {
        let Some(info) = self.lookup_var_mut(name) else {
            return false;
        };
        let binding_id = info.binding_id;
        info.ty = ty.clone();
        info.narrowed_type = None;
        info.const_integer_value = None;
        if let Some(fact) = self.retained_bindings.get_mut(&binding_id) {
            fact.ty = ty;
        }
        true
    }

    pub(crate) fn set_const_integer_value(&mut self, name: &str, value: BigInt) -> bool {
        if let Some(info) = self.lookup_var_mut(name) {
            info.const_integer_value = Some(value);
            return true;
        }
        false
    }

    pub(crate) fn clear_const_integer_value(&mut self, name: &str) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.const_integer_value = None;
        }
    }

    pub(crate) fn const_integer_value(&self, name: &str) -> Option<&BigInt> {
        self.lookup_var(name)
            .and_then(|info| info.const_integer_value.as_ref())
    }

    pub(crate) fn save_const_integer_state(&self) -> ConstIntegerSnapshot {
        let mut snapshot = Vec::new();
        for frame in &self.frames {
            for (name, info) in frame {
                snapshot.push((name.clone(), info.const_integer_value.clone()));
            }
        }
        snapshot
    }

    pub(crate) fn restore_const_integer_state(&mut self, snapshot: &ConstIntegerSnapshot) {
        for (name, const_integer_value) in snapshot {
            if let Some(info) = self.lookup_var_mut(name) {
                info.const_integer_value.clone_from(const_integer_value);
            }
        }
    }

    /// Look up a variable, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&VarInfo> {
        self.lookup_var(name)
    }
    pub(crate) fn current_frame_binding(&self, name: &str) -> Option<&VarInfo> {
        self.frames.last().and_then(|frame| frame.get(name))
    }
    pub(crate) fn restore_captured_binding(&mut self, name: String, info: VarInfo) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, info);
        }
    }

    pub(crate) fn active_bindings(&self) -> Vec<(String, Type)> {
        let mut bindings = Vec::new();
        for frame in &self.frames {
            for (name, info) in frame {
                if info.is_moved || info.is_poisoned_binding() {
                    continue;
                }
                bindings.push((name.clone(), info.effective_type().clone()));
            }
        }
        bindings
    }

    /// Look up a variable within an inclusive frame range.
    pub fn lookup_in_frame_range(&self, name: &str, start: usize, end: usize) -> Option<&VarInfo> {
        if start > end || end >= self.frames.len() {
            return None;
        }
        for frame_index in (start..=end).rev() {
            if let Some(info) = self.frames[frame_index].get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Mark a variable as moved (for ownership tracking).
    pub fn mark_moved(&mut self, name: &str) -> bool {
        if let Some(info) = self.lookup_var_mut(name) {
            if info.ty.ownership() == OwnershipKind::Move || info.requires_move_tracking {
                info.is_moved = true;
                return true;
            }

            return false; // Copy type, don't mark as moved
        }
        false
    }

    /// Mark a binding moved even when its surface type is ordinarily copyable.
    ///
    /// This is used when lowering proves that a specialized representation,
    /// such as an owning retained closure, is consumed at the boundary.
    pub fn mark_binding_moved(&mut self, name: &str) -> bool {
        let Some(info) = self.lookup_var_mut(name) else {
            return false;
        };
        info.requires_move_tracking = true;
        info.is_moved = true;
        true
    }

    /// Check if a variable has been moved.
    pub fn is_moved(&self, name: &str) -> bool {
        self.lookup_var(name).is_some_and(|info| info.is_moved)
    }

    /// Reset moved state (e.g., when variable is reassigned).
    pub fn reset_moved(&mut self, name: &str) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.is_moved = false;
        }
    }

    // --- Mutable borrow tracking ---

    /// Mark a variable as mutably borrowed.
    pub fn mark_mut_borrowed(&mut self, name: &str) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.is_mut_borrowed = true;
        }
    }

    /// Check if a variable is currently mutably borrowed.
    pub fn is_mut_borrowed(&self, name: &str) -> bool {
        self.lookup_var(name)
            .is_some_and(|info| info.is_mut_borrowed)
    }

    /// Clear the mutable borrow on a variable (after the borrowing call returns).
    pub fn clear_mut_borrow(&mut self, name: &str) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.is_mut_borrowed = false;
        }
    }

    // --- Narrowing support ---

    /// Set the narrowed type for a variable.
    pub fn narrow_var(&mut self, name: &str, narrowed_type: Type) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.narrowed_type = Some(narrowed_type);
        }
    }

    /// Clear the narrowed type for a variable (restore to declared type).
    pub fn clear_narrowing(&mut self, name: &str) {
        if let Some(info) = self.lookup_var_mut(name) {
            info.narrowed_type = None;
        }
    }

    /// Save the current narrowing state for all variables in scope.
    /// Used before entering a branch.
    pub fn save_narrowing_state(&self) -> NarrowingSnapshot {
        let mut snapshot = Vec::new();
        for frame in &self.frames {
            for (name, info) in frame {
                snapshot.push((name.clone(), info.narrowed_type.clone()));
            }
        }
        snapshot
    }

    /// Restore narrowing state from a snapshot.
    /// Used after exiting a branch to restore the state before the branch.
    pub fn restore_narrowing_state(&mut self, snapshot: &NarrowingSnapshot) {
        for (name, narrowed) in snapshot {
            if let Some(info) = self.lookup_var_mut(name) {
                info.narrowed_type.clone_from(narrowed);
            }
        }
    }

    /// Get the effective type of a variable (narrowed if available, otherwise declared).
    pub fn effective_type(&self, name: &str) -> Option<&Type> {
        self.lookup(name).map(VarInfo::effective_type)
    }

    /// Check whether an existing binding is mutable.
    pub fn is_mutable_binding(&self, name: &str) -> Option<bool> {
        self.lookup(name).map(VarInfo::is_mutable_binding)
    }

    // --- Moved state snapshot support ---

    /// Save the current moved state for all variables in scope.
    /// Used before entering a branch or loop body.
    pub fn save_moved_state(&self) -> MovedSnapshot {
        let mut snapshot = Vec::new();
        for frame in &self.frames {
            for (name, info) in frame {
                snapshot.push((name.clone(), info.is_moved));
            }
        }
        snapshot
    }

    /// Restore moved state from a snapshot.
    /// Used after exiting a branch to restore the state before the branch.
    pub fn restore_moved_state(&mut self, snapshot: &MovedSnapshot) {
        for (name, was_moved) in snapshot {
            if let Some(info) = self.lookup_var_mut(name) {
                info.is_moved = *was_moved;
            }
        }
    }

    /// Return the names of variables that were newly moved since the snapshot.
    /// A variable is "newly moved" if it was not moved in the snapshot but is moved now.
    pub fn moved_since(&self, snapshot: &MovedSnapshot) -> Vec<String> {
        let mut newly_moved = Vec::new();
        for (name, was_moved) in snapshot {
            if !was_moved && self.is_moved(name) {
                newly_moved.push(name.clone());
            }
        }
        newly_moved
    }

    // --- Type alias support ---

    /// Register a type alias.
    pub fn define_type_alias(&mut self, name: String, ty: Type) {
        self.type_aliases.insert(name, ty);
    }

    /// Look up a type alias.
    pub fn lookup_type_alias(&self, name: &str) -> Option<&Type> {
        self.type_aliases.get(name)
    }

    pub(crate) fn type_aliases(&self) -> &HashMap<String, Type> {
        &self.type_aliases
    }

    /// Register a generic type alias (e.g., `type Pair[T] = tuple[T, T]`).
    pub fn define_generic_type_alias(&mut self, name: String, type_params: Vec<String>, ty: Type) {
        self.generic_type_aliases.insert(name, (type_params, ty));
    }

    /// Look up a generic type alias.
    pub fn lookup_generic_type_alias(&self, name: &str) -> Option<&(Vec<String>, Type)> {
        self.generic_type_aliases.get(name)
    }

    pub(crate) fn generic_type_aliases(&self) -> &HashMap<String, (Vec<String>, Type)> {
        &self.generic_type_aliases
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup() {
        let mut scope = Scope::new();
        scope.define("x".to_string(), Type::Int);
        let info = scope.lookup("x").unwrap();
        assert_eq!(info.ty, Type::Int);
        assert!(!info.is_moved);
    }

    #[test]
    fn test_nested_scopes() {
        let mut scope = Scope::new();
        scope.define("x".to_string(), Type::Int);
        scope.push();
        scope.define("y".to_string(), Type::Str);
        assert!(scope.lookup("x").is_some()); // visible from outer
        assert!(scope.lookup("y").is_some());
        scope.pop();
        assert!(scope.lookup("x").is_some());
        assert!(scope.lookup("y").is_none()); // no longer visible
    }

    #[test]
    fn binding_ids_distinguish_shadowed_names_and_outlive_frames() {
        let mut scope = Scope::new();
        scope.define("value".to_string(), Type::Int);
        let outer = scope.lookup("value").unwrap().binding_id;

        scope.push();
        scope.define("value".to_string(), Type::Str);
        let inner = scope.lookup("value").unwrap().binding_id;
        assert_ne!(outer, inner);
        scope.pop();

        assert_eq!(scope.lookup("value").unwrap().binding_id, outer);
        assert_eq!(scope.retained_binding(inner).unwrap().name, "value");
        assert_eq!(
            scope.retained_binding(inner).unwrap().binding_kind,
            BindingKind::Local
        );
    }

    #[test]
    fn module_constants_have_distinct_immutable_binding_facts() {
        let mut scope = Scope::new();
        scope.define_module_constant("VALUES".to_string(), Type::List(Box::new(Type::Int)));

        let info = scope.lookup("VALUES").unwrap();
        assert_eq!(info.binding_kind, BindingKind::ModuleConstant);
        assert_eq!(info.mutability, BindingMutability::Immutable);
        assert_eq!(
            scope
                .retained_binding(info.binding_id)
                .unwrap()
                .binding_kind,
            BindingKind::ModuleConstant
        );
    }

    #[test]
    fn retained_receiver_and_ephemeral_facts_keep_final_conventions() {
        let mut scope = Scope::new();
        scope.push();
        let receiver = scope.define_receiver(
            "self".to_string(),
            Type::Any,
            ReceiverConvention::SharedBorrow,
        );
        scope.define_ephemeral("item".to_string(), Type::Int, EphemeralOrigin::Iteration);
        let item = scope.lookup("item").unwrap().binding_id;
        scope.pop();

        scope.patch_receiver_convention(receiver, ReceiverConvention::MutableBorrow);
        let receiver_fact = scope.retained_binding(receiver).unwrap();
        assert_eq!(receiver_fact.binding_kind, BindingKind::Receiver);
        assert_eq!(
            receiver_fact.receiver_convention,
            Some(ReceiverConvention::MutableBorrow)
        );
        assert_eq!(receiver_fact.mutability, BindingMutability::Mutable);
        assert_eq!(
            scope.retained_binding(item).unwrap().binding_kind,
            BindingKind::EphemeralLocal(EphemeralOrigin::Iteration)
        );
    }

    #[test]
    fn test_move_tracking() {
        let mut scope = Scope::new();
        scope.define("s".to_string(), Type::Str);
        assert!(!scope.is_moved("s"));
        scope.mark_moved("s");
        assert!(scope.is_moved("s"));
    }

    #[test]
    fn test_copy_types_not_moved() {
        let mut scope = Scope::new();
        scope.define("x".to_string(), Type::Int);
        let moved = scope.mark_moved("x");
        assert!(!moved); // Int is Copy, not moved
        assert!(!scope.is_moved("x"));
    }

    #[test]
    fn test_specialized_copy_binding_can_be_marked_moved() {
        let mut scope = Scope::new();
        scope.define("handler".to_string(), Type::Int);

        assert!(scope.mark_binding_moved("handler"));
        assert!(scope.is_moved("handler"));
        scope.reset_moved("handler");
        assert!(scope.mark_moved("handler"));
        assert!(scope.is_moved("handler"));
    }

    // --- Narrowing tests ---

    #[test]
    fn test_narrowing() {
        let mut scope = Scope::new();
        let union_type = Type::Union(vec![Type::Int, Type::Str]);
        scope.define("x".to_string(), union_type.clone());

        // Before narrowing, effective type is declared type
        assert_eq!(scope.effective_type("x"), Some(&union_type));

        // Narrow to Int
        scope.narrow_var("x", Type::Int);
        assert_eq!(scope.effective_type("x"), Some(&Type::Int));

        // Clear narrowing
        scope.clear_narrowing("x");
        assert_eq!(scope.effective_type("x"), Some(&union_type));
    }

    #[test]
    fn test_narrowing_save_restore() {
        let mut scope = Scope::new();
        let union_type = Type::Union(vec![Type::Int, Type::Str]);
        scope.define("x".to_string(), union_type.clone());

        // Save state before branch
        let snapshot = scope.save_narrowing_state();

        // Narrow in branch
        scope.narrow_var("x", Type::Int);
        assert_eq!(scope.effective_type("x"), Some(&Type::Int));

        // Restore state
        scope.restore_narrowing_state(&snapshot);
        assert_eq!(scope.effective_type("x"), Some(&union_type));
    }

    #[test]
    fn test_type_alias() {
        let mut scope = Scope::new();
        scope.define_type_alias("UserId".to_string(), Type::Int);
        assert_eq!(scope.lookup_type_alias("UserId"), Some(&Type::Int));
        assert_eq!(scope.lookup_type_alias("Unknown"), None);
    }

    // --- Moved state snapshot tests ---

    #[test]
    fn test_save_restore_moved_state() {
        let mut scope = Scope::new();
        scope.define("s".to_string(), Type::Str);
        assert!(!scope.is_moved("s"));

        let snapshot = scope.save_moved_state();
        scope.mark_moved("s");
        assert!(scope.is_moved("s"));

        scope.restore_moved_state(&snapshot);
        assert!(!scope.is_moved("s"));
    }

    #[test]
    fn test_moved_since() {
        let mut scope = Scope::new();
        scope.define("s".to_string(), Type::Str);
        scope.define("x".to_string(), Type::Int);

        let snapshot = scope.save_moved_state();
        scope.mark_moved("s"); // Move type — should appear in moved_since
        scope.mark_moved("x"); // Copy type — should NOT appear

        let newly = scope.moved_since(&snapshot);
        assert_eq!(newly, vec!["s".to_string()]);
    }
}
