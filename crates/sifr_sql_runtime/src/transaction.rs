use crate::{SqlError, SqlErrorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionState {
    Live,
    Committed,
    RolledBack,
    Poisoned,
    Dropped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionMachine {
    state: TransactionState,
    savepoint_depth: u32,
}

impl TransactionMachine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: TransactionState::Live,
            savepoint_depth: 0,
        }
    }

    #[must_use]
    pub const fn state(&self) -> TransactionState {
        self.state
    }

    #[must_use]
    pub const fn savepoint_depth(&self) -> u32 {
        self.savepoint_depth
    }

    pub fn ensure_live(&self) -> Result<(), SqlError> {
        if self.state == TransactionState::Live {
            Ok(())
        } else {
            Err(SqlError::new(SqlErrorKind::TransactionControl))
        }
    }

    pub fn push_savepoint(&mut self) -> Result<u32, SqlError> {
        self.ensure_live()?;
        self.savepoint_depth = self
            .savepoint_depth
            .checked_add(1)
            .ok_or_else(|| SqlError::new(SqlErrorKind::ResourceLimit))?;
        Ok(self.savepoint_depth)
    }

    pub fn pop_savepoint(&mut self) -> Result<u32, SqlError> {
        self.ensure_live()?;
        if self.savepoint_depth == 0 {
            return Err(SqlError::new(SqlErrorKind::TransactionControl));
        }
        let depth = self.savepoint_depth;
        self.savepoint_depth -= 1;
        Ok(depth)
    }

    pub fn committed(&mut self) -> Result<(), SqlError> {
        self.terminal(TransactionState::Committed)
    }

    pub fn rolled_back(&mut self) -> Result<(), SqlError> {
        self.terminal(TransactionState::RolledBack)
    }

    pub fn poison(&mut self) {
        if self.state == TransactionState::Live {
            self.state = TransactionState::Poisoned;
            self.savepoint_depth = 0;
        }
    }

    pub fn dropped(&mut self) {
        if self.state == TransactionState::Live {
            self.state = TransactionState::Dropped;
            self.savepoint_depth = 0;
        }
    }

    fn terminal(&mut self, state: TransactionState) -> Result<(), SqlError> {
        self.ensure_live()?;
        if self.savepoint_depth != 0 {
            return Err(SqlError::new(SqlErrorKind::TransactionControl));
        }
        self.state = state;
        Ok(())
    }
}

impl Default for TransactionMachine {
    fn default() -> Self {
        Self::new()
    }
}
