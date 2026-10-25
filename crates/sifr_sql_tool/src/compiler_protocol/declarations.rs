//! Explicit wire DTOs keep validation separate from canonical compiler types.
use serde::{Deserialize, Serialize};
use sifr_sql_contract::{
    MigrationNodeId, MigrationSourceDeclaration, MigrationSourceStep, MigrationSourceStepKind,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Declaration {
    id: String,
    parents: Vec<String>,
    function: String,
    author: String,
    created_at: String,
    steps: Vec<Step>,
    rollback: Option<Vec<Step>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    kind: Kind,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Kind {
    Ddl { statement: String },
    SqlData { statement: String },
    Assertion { statement: String },
    RecoveryPoint { name: String },
    Begin {},
    Commit {},
}

impl From<&MigrationSourceDeclaration> for Declaration {
    fn from(value: &MigrationSourceDeclaration) -> Self {
        Self {
            id: value.id.to_string(),
            parents: value.parents.iter().map(ToString::to_string).collect(),
            function: value.function.clone(),
            author: value.author.clone(),
            created_at: value.created_at.clone(),
            steps: value.steps.iter().map(Into::into).collect(),
            rollback: value
                .rollback
                .as_ref()
                .map(|steps| steps.iter().map(Into::into).collect()),
        }
    }
}

impl From<&MigrationSourceStep> for Step {
    fn from(value: &MigrationSourceStep) -> Self {
        Self {
            id: value.id.to_string(),
            kind: match &value.kind {
                MigrationSourceStepKind::Ddl { statement } => Kind::Ddl {
                    statement: statement.clone(),
                },
                MigrationSourceStepKind::SqlData { statement } => Kind::SqlData {
                    statement: statement.clone(),
                },
                MigrationSourceStepKind::Assertion { statement } => Kind::Assertion {
                    statement: statement.clone(),
                },
                MigrationSourceStepKind::RecoveryPoint { name } => {
                    Kind::RecoveryPoint { name: name.clone() }
                }
                MigrationSourceStepKind::Begin => Kind::Begin {},
                MigrationSourceStepKind::Commit => Kind::Commit {},
            },
        }
    }
}

fn id(value: String) -> Result<MigrationNodeId, String> {
    MigrationNodeId::new(value).map_err(|error| error.to_string())
}

impl TryFrom<Declaration> for MigrationSourceDeclaration {
    type Error = String;
    fn try_from(value: Declaration) -> Result<Self, String> {
        Ok(Self {
            id: id(value.id)?,
            parents: value
                .parents
                .into_iter()
                .map(id)
                .collect::<Result<_, _>>()?,
            function: value.function,
            author: value.author,
            created_at: value.created_at,
            steps: value
                .steps
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            rollback: value
                .rollback
                .map(|steps| steps.into_iter().map(TryInto::try_into).collect())
                .transpose()?,
        })
    }
}

impl TryFrom<Step> for MigrationSourceStep {
    type Error = String;
    fn try_from(value: Step) -> Result<Self, String> {
        Ok(Self {
            id: id(value.id)?,
            kind: match value.kind {
                Kind::Ddl { statement } => MigrationSourceStepKind::Ddl { statement },
                Kind::SqlData { statement } => MigrationSourceStepKind::SqlData { statement },
                Kind::Assertion { statement } => MigrationSourceStepKind::Assertion { statement },
                Kind::RecoveryPoint { name } => MigrationSourceStepKind::RecoveryPoint { name },
                Kind::Begin {} => MigrationSourceStepKind::Begin,
                Kind::Commit {} => MigrationSourceStepKind::Commit,
            },
        })
    }
}
