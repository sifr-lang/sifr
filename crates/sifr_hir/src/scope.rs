//! Scope-based name resolution for Sifr.

use sifr_type_system::{Type, OwnershipKind};
use std::collections::HashMap;

/// Tracks variable state for ownership.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Type,
    pub is_moved: bool,
}

/// A scope for name resolution.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Stack of scope frames (innermost last).
    frames: Vec<HashMap<String, VarInfo>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
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
            frame.insert(name, VarInfo { ty, is_moved: false });
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
}
