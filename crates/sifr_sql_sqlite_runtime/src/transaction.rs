use crate::config::SqliteProfile;
use crate::pool::{ExecutionOptions, SqliteConnection};
use crate::stream::SqliteTransactionRowStream;
use crate::worker::{SqliteExecutionMetadata, SqliteRow};
use sifr_sql_runtime::{ExecutionRequest, ExecutionResult, SqlError, SqlErrorKind};

pub struct SqliteTransaction {
    connection: Option<SqliteConnection>,
    next_savepoint: u32,
}

impl SqliteTransaction {
    pub(crate) fn new(connection: SqliteConnection) -> Self {
        Self {
            connection: Some(connection),
            next_savepoint: 1,
        }
    }

    pub async fn execute(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<ExecutionResult<SqliteExecutionMetadata>, SqlError> {
        self.connection_mut()?.execute(request, options).await
    }

    pub async fn fetch_all(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<Vec<SqliteRow>, SqlError> {
        self.connection_mut()?.fetch_all(request, options).await
    }

    pub async fn stream(
        &mut self,
        request: ExecutionRequest<SqliteProfile>,
        options: ExecutionOptions,
    ) -> Result<SqliteTransactionRowStream<'_>, SqlError> {
        SqliteTransactionRowStream::open(self, request, options).await
    }

    pub async fn savepoint(&mut self) -> Result<SqliteSavepoint, SqlError> {
        let name = format!("sifr_sp_{}", self.next_savepoint);
        self.next_savepoint = self.next_savepoint.saturating_add(1);
        let statement = format!("SAVEPOINT {name}");
        let timeout = self.timeout()?;
        self.connection_mut()?
            .worker()?
            .control(statement, timeout)
            .await?;
        Ok(SqliteSavepoint { name, active: true })
    }

    pub async fn release_savepoint(
        &mut self,
        savepoint: &mut SqliteSavepoint,
    ) -> Result<(), SqlError> {
        savepoint.ensure_active()?;
        let statement = format!("RELEASE SAVEPOINT {}", savepoint.name);
        let timeout = self.timeout()?;
        self.connection_mut()?
            .worker()?
            .control(statement, timeout)
            .await?;
        savepoint.active = false;
        Ok(())
    }

    pub async fn rollback_to(&mut self, savepoint: &mut SqliteSavepoint) -> Result<(), SqlError> {
        savepoint.ensure_active()?;
        let rollback = format!("ROLLBACK TO SAVEPOINT {}", savepoint.name);
        let release = format!("RELEASE SAVEPOINT {}", savepoint.name);
        let timeout = self.timeout()?;
        self.connection_mut()?
            .worker()?
            .control(rollback, timeout)
            .await?;
        self.connection_mut()?
            .worker()?
            .control(release, timeout)
            .await?;
        savepoint.active = false;
        Ok(())
    }

    pub async fn commit(mut self) -> Result<(), SqlError> {
        let timeout = self.timeout()?;
        self.connection_mut()?
            .worker()?
            .control("COMMIT", timeout)
            .await?;
        self.connection
            .take()
            .ok_or_else(|| SqlError::new(SqlErrorKind::TransactionControl))?
            .release(None)
            .await
    }

    pub async fn rollback(mut self) -> Result<(), SqlError> {
        let timeout = self.timeout()?;
        self.connection_mut()?
            .worker()?
            .control("ROLLBACK", timeout)
            .await?;
        self.connection
            .take()
            .ok_or_else(|| SqlError::new(SqlErrorKind::TransactionControl))?
            .release(None)
            .await
    }

    pub(crate) fn connection_mut(&mut self) -> Result<&mut SqliteConnection, SqlError> {
        self.connection
            .as_mut()
            .ok_or_else(|| SqlError::new(SqlErrorKind::TransactionControl))
    }

    fn timeout(&self) -> Result<std::time::Duration, SqlError> {
        self.connection
            .as_ref()
            .map(|connection| connection.profile.limits().statement_timeout)
            .ok_or_else(|| SqlError::new(SqlErrorKind::TransactionControl))
    }
}

pub struct SqliteSavepoint {
    name: String,
    active: bool,
}

impl SqliteSavepoint {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    fn ensure_active(&self) -> Result<(), SqlError> {
        if self.active {
            Ok(())
        } else {
            Err(SqlError::new(SqlErrorKind::TransactionControl))
        }
    }
}
