use super::{
    CompiledMigrationStep, MigrationCompileError, MigrationCompileErrorKind, MigrationStateIdentity,
};
use crate::ObjectId;
use std::collections::BTreeSet;
use std::marker::PhantomData;

pub trait MigrationState {
    const IDENTITY: &'static str;
}

/// An affine compiler plan. It deliberately implements neither `Clone` nor
/// `Copy`; each checked transition consumes the previous state.
pub struct MigrationPlan<S: MigrationState> {
    state: MigrationStateIdentity,
    compiled_steps: Vec<CompiledMigrationStep>,
    marker: PhantomData<S>,
}

impl<S: MigrationState> MigrationPlan<S> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: MigrationStateIdentity::new(S::IDENTITY),
            compiled_steps: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn transition<N: MigrationState>(
        mut self,
        step: CompiledMigrationStep,
    ) -> Result<MigrationPlan<N>, MigrationCompileError> {
        if step.input_state != self.state || step.output_state.as_str() != N::IDENTITY {
            return Err(MigrationCompileError::new(
                MigrationCompileErrorKind::InvalidStep,
                "migration plan transition does not match its nominal state types",
            ));
        }
        self.compiled_steps.push(step);
        Ok(MigrationPlan {
            state: MigrationStateIdentity::new(N::IDENTITY),
            compiled_steps: self.compiled_steps,
            marker: PhantomData,
        })
    }

    #[must_use]
    pub fn state(&self) -> &MigrationStateIdentity {
        &self.state
    }

    #[must_use]
    pub fn into_steps(self) -> Vec<CompiledMigrationStep> {
        self.compiled_steps
    }
}

impl<S: MigrationState> Default for MigrationPlan<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// A callback-scoped migration database. Its private lifetime and state marker
/// prevent storage outside the compiler-provided callback scope.
pub struct MigrationDb<'scope, S: MigrationState> {
    allowed_objects: &'scope BTreeSet<ObjectId>,
    marker: PhantomData<&'scope mut S>,
}

impl<'scope, S: MigrationState> MigrationDb<'scope, S> {
    #[must_use]
    pub fn new(allowed_objects: &'scope BTreeSet<ObjectId>) -> Self {
        Self {
            allowed_objects,
            marker: PhantomData,
        }
    }

    pub fn require_object(&self, object: &ObjectId) -> Result<(), MigrationCompileError> {
        if self.allowed_objects.contains(object) {
            Ok(())
        } else {
            Err(MigrationCompileError::new(
                MigrationCompileErrorKind::UnknownSchemaObject,
                format!(
                    "migration state '{}' does not contain '{object}'",
                    S::IDENTITY
                ),
            ))
        }
    }
}
