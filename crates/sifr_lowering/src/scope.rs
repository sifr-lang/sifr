//! Scope-based name resolution for Sifr.
//!
//! Supports type narrowing: variables can have a `narrowed_type` that
//! differs from their `declared_type` within control flow branches.

use num_bigint::BigInt;
use sifr_type_system::{OwnershipKind, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Local,
    Parameter,
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
    /// The declared type (from annotation or inference).
    pub ty: Type,
    /// The narrowed type (from control flow analysis). None means not narrowed.
    pub narrowed_type: Option<Type>,
    /// Whether the variable has been moved.
    pub is_moved: bool,
    /// Whether the variable is currently mutably borrowed.
    pub is_mut_borrowed: bool,
    /// Whether the binding itself is mutable.
    pub mutability: BindingMutability,
    /// Whether this binding originated from a function parameter.
    pub binding_kind: BindingKind,
    /// Whether the binding type was provided explicitly (annotation/parameter).
    pub type_source: BindingTypeSource,
    /// Compile-time exact integer value known for this binding, if any.
    pub const_integer_value: Option<BigInt>,
    /// Proof that this binding only exists to suppress cascades after an emitted error.
    error_taint: Option<ErrorTaint>,
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

    /// Define an explicitly-typed local variable in the current scope.
    pub fn define_explicit_local(&mut self, name: String, ty: Type) {
        self.define_binding(name, ty, true, BindingKind::Local, true);
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
    pub fn define_parameter(&mut self, name: String, ty: Type, is_mutable_binding: bool) {
        self.define_binding(name, ty, is_mutable_binding, BindingKind::Parameter, true);
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
    ) {
        if let Some(frame) = self.frames.last_mut() {
            let mutability = if is_mutable_binding {
                BindingMutability::Mutable
            } else {
                BindingMutability::Immutable
            };
            let type_source = if has_explicit_type {
                BindingTypeSource::Explicit
            } else {
                BindingTypeSource::Inferred
            };
            frame.insert(
                name,
                VarInfo {
                    ty,
                    narrowed_type: None,
                    is_moved: false,
                    is_mut_borrowed: false,
                    mutability,
                    binding_kind,
                    type_source,
                    const_integer_value: None,
                    error_taint,
                },
            );
        }
    }

    /// Update the declared type for an existing variable.
    pub fn set_type(&mut self, name: &str, ty: Type) -> bool {
        if let Some(info) = self.lookup_var_mut(name) {
            info.ty = ty;
            info.narrowed_type = None;
            info.const_integer_value = None;
            return true;
        }
        false
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
            if info.ty.ownership() == OwnershipKind::Move {
                info.is_moved = true;
                return true;
            }

            return false; // Copy type, don't mark as moved
        }
        false
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

    /// Register a generic type alias (e.g., `type Pair[T] = tuple[T, T]`).
    pub fn define_generic_type_alias(&mut self, name: String, type_params: Vec<String>, ty: Type) {
        self.generic_type_aliases.insert(name, (type_params, ty));
    }

    /// Look up a generic type alias.
    pub fn lookup_generic_type_alias(&self, name: &str) -> Option<&(Vec<String>, Type)> {
        self.generic_type_aliases.get(name)
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
