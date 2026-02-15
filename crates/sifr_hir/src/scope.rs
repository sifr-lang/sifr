//! Scope-based name resolution for Sifr.
//!
//! Supports type narrowing: variables can have a `narrowed_type` that
//! differs from their `declared_type` within control flow branches.

use sifr_type_system::{Type, OwnershipKind};
use std::collections::HashMap;

/// Tracks variable state for ownership and narrowing.
#[derive(Debug, Clone)]
pub struct VarInfo {
    /// The declared type (from annotation or inference).
    pub ty: Type,
    /// The narrowed type (from control flow analysis). None means not narrowed.
    pub narrowed_type: Option<Type>,
    /// Whether the variable has been moved.
    pub is_moved: bool,
}

impl VarInfo {
    /// Get the effective type (narrowed if available, otherwise declared).
    pub fn effective_type(&self) -> &Type {
        self.narrowed_type.as_ref().unwrap_or(&self.ty)
    }
}

/// A snapshot of the narrowing state for all variables in scope.
/// Used to save/restore narrowing at branch points.
pub type NarrowingSnapshot = Vec<(String, Option<Type>)>;

/// A snapshot of the moved state for all variables in scope.
/// Used to save/restore moved state at branch points and loop boundaries.
pub type MovedSnapshot = Vec<(String, bool)>;

/// A scope for name resolution.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Stack of scope frames (innermost last).
    frames: Vec<HashMap<String, VarInfo>>,
    /// Type alias registry.
    type_aliases: HashMap<String, Type>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
            type_aliases: HashMap::new(),
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

    /// Define a variable in the current (innermost) scope.
    pub fn define(&mut self, name: String, ty: Type) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, VarInfo { ty, narrowed_type: None, is_moved: false });
        }
    }

    /// Look up a variable, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&VarInfo> {
        for frame in self.frames.iter().rev() {
            if let Some(info) = frame.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Mark a variable as moved (for ownership tracking).
    pub fn mark_moved(&mut self, name: &str) -> bool {
        for frame in self.frames.iter_mut().rev() {
            if let Some(info) = frame.get_mut(name) {
                if info.ty.ownership() == OwnershipKind::Move {
                    info.is_moved = true;
                    return true;
                }
                return false; // Copy type, don't mark as moved
            }
        }
        false
    }

    /// Check if a variable has been moved.
    pub fn is_moved(&self, name: &str) -> bool {
        for frame in self.frames.iter().rev() {
            if let Some(info) = frame.get(name) {
                return info.is_moved;
            }
        }
        false
    }

    /// Reset moved state (e.g., when variable is reassigned).
    pub fn reset_moved(&mut self, name: &str) {
        for frame in self.frames.iter_mut().rev() {
            if let Some(info) = frame.get_mut(name) {
                info.is_moved = false;
                return;
            }
        }
    }

    // --- Narrowing support ---

    /// Set the narrowed type for a variable.
    pub fn narrow_var(&mut self, name: &str, narrowed_type: Type) {
        for frame in self.frames.iter_mut().rev() {
            if let Some(info) = frame.get_mut(name) {
                info.narrowed_type = Some(narrowed_type);
                return;
            }
        }
    }

    /// Clear the narrowed type for a variable (restore to declared type).
    pub fn clear_narrowing(&mut self, name: &str) {
        for frame in self.frames.iter_mut().rev() {
            if let Some(info) = frame.get_mut(name) {
                info.narrowed_type = None;
                return;
            }
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
            for frame in self.frames.iter_mut().rev() {
                if let Some(info) = frame.get_mut(name) {
                    info.narrowed_type = narrowed.clone();
                    break;
                }
            }
        }
    }

    /// Get the effective type of a variable (narrowed if available, otherwise declared).
    pub fn effective_type(&self, name: &str) -> Option<&Type> {
        self.lookup(name).map(|info| info.effective_type())
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
            for frame in self.frames.iter_mut().rev() {
                if let Some(info) = frame.get_mut(name) {
                    info.is_moved = *was_moved;
                    break;
                }
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

    // --- M3: Narrowing tests ---

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
