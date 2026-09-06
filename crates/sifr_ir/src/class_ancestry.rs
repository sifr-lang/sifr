use crate::HirClass;
use sifr_type_system::Type;

impl HirClass {
    /// Nominal ancestry for type checking, separate from the stored data parent.
    /// The builtin Error marker has no embedded parent field.
    #[must_use]
    pub fn semantic_parent_chain(&self) -> Option<String> {
        if !self.is_error_type {
            return self.parent_class.clone();
        }
        if let Some(Type::Class {
            identity,
            name,
            parent_class,
            ..
        }) = self.parent_type.as_ref().map(Type::resolve_alias)
        {
            let parent = identity.as_ref().unwrap_or(name);
            return Some(match parent_class {
                Some(chain) => format!("{parent}|{chain}"),
                None => parent.clone(),
            });
        }
        Some(
            self.parent_class
                .clone()
                .unwrap_or_else(|| "Error".to_string()),
        )
    }
}
