use crate::Type;

fn task_owner_type(name: &str) -> Type {
    Type::Class {
        identity: Some(format!("sifr.task.{name}")),
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Default::default(),
        methods: Default::default(),
        parent_class: None,
    }
}

/// Canonical compiler-owned scope receiver, distinct from user classes.
pub fn task_scope_type() -> Type {
    task_owner_type("TaskScope")
}

/// Canonical compiler-owned group receiver, distinct from user classes.
pub fn task_group_type() -> Type {
    task_owner_type("TaskGroup")
}

pub fn is_task_owner_type(ty: &Type) -> bool {
    let ty = ty.resolve_alias();
    ty == &task_scope_type() || ty == &task_group_type()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_owner_admission_requires_the_complete_canonical_type() {
        for owner in [task_scope_type(), task_group_type()] {
            assert!(is_task_owner_type(&owner));
            let mut collision = owner.clone();
            if let Type::Class { identity, .. } = &mut collision {
                *identity = Some("user.TaskScope".to_string());
            }
            assert!(!is_task_owner_type(&collision));
            let mut generic = owner.clone();
            if let Type::Class { type_args, .. } = &mut generic {
                type_args.push(Type::Int);
            }
            assert!(!is_task_owner_type(&generic));
            let mut field = owner;
            if let Type::Class { fields, .. } = &mut field {
                *fields = vec![("value".to_string(), Type::Int)].into();
            }
            assert!(!is_task_owner_type(&field));
        }
        assert!(!is_task_owner_type(&Type::Int));
    }
}
