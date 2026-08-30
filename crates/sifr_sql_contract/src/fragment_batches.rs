use crate::{ObjectId, QueryParameterSlot, SifrType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictBatchBehavior {
    AbortBatch,
    IgnoreConflicts,
    UpdateConflicts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderParameterLimit {
    pub maximum: u32,
}

impl ProviderParameterLimit {
    pub fn new(maximum: u32) -> Result<Self, FragmentBatchError> {
        if maximum == 0 {
            return Err(FragmentBatchError::new(
                "a provider parameter limit must be positive",
            ));
        }
        Ok(Self { maximum })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchChunk {
    pub first_row: u32,
    pub row_count: u32,
    pub parameter_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValuesFragmentContract {
    pub columns: Vec<(ObjectId, SifrType)>,
    pub rows: u32,
    pub parameters_per_row: u32,
    pub conflict: ConflictBatchBehavior,
    pub chunks: Vec<BatchChunk>,
}

impl ValuesFragmentContract {
    pub fn checked(
        columns: Vec<(ObjectId, SifrType)>,
        rows: u32,
        parameters_per_row: u32,
        conflict: ConflictBatchBehavior,
        provider_limit: ProviderParameterLimit,
        requested_chunk_rows: Option<u32>,
    ) -> Result<Self, FragmentBatchError> {
        if columns.is_empty() || rows == 0 || parameters_per_row == 0 {
            return Err(FragmentBatchError::new(
                "a values fragment needs columns, rows, and parameters",
            ));
        }
        if parameters_per_row > provider_limit.maximum {
            return Err(FragmentBatchError::new(
                "one values row exceeds the provider parameter limit",
            ));
        }
        let provider_rows = provider_limit.maximum / parameters_per_row;
        let chunk_rows = requested_chunk_rows.unwrap_or(rows);
        if chunk_rows == 0 || chunk_rows > provider_rows {
            return Err(FragmentBatchError::new(
                "the explicit values chunk size exceeds the provider parameter limit",
            ));
        }
        let chunks = (0..rows)
            .step_by(usize::try_from(chunk_rows).map_err(|_| {
                FragmentBatchError::new("the values chunk size is not representable")
            })?)
            .map(|first_row| {
                let row_count = chunk_rows.min(rows - first_row);
                Ok(BatchChunk {
                    first_row,
                    row_count,
                    parameter_count: row_count.checked_mul(parameters_per_row).ok_or_else(
                        || FragmentBatchError::new("values parameter count overflows"),
                    )?,
                })
            })
            .collect::<Result<Vec<_>, FragmentBatchError>>()?;
        if requested_chunk_rows.is_none() && chunks.len() != 1 {
            return Err(FragmentBatchError::new(
                "values exceed the provider parameter limit; select an explicit chunk size",
            ));
        }
        Ok(Self {
            columns,
            rows,
            parameters_per_row,
            conflict,
            chunks,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentFragmentContract {
    pub relation: ObjectId,
    pub assignments: BTreeMap<ObjectId, QueryParameterSlot>,
}

impl AssignmentFragmentContract {
    pub fn checked(
        relation: ObjectId,
        assignments: BTreeMap<ObjectId, QueryParameterSlot>,
        writable_columns: &BTreeSet<ObjectId>,
        provider_limit: ProviderParameterLimit,
    ) -> Result<Self, FragmentBatchError> {
        if assignments.is_empty()
            || assignments.len() > usize::try_from(provider_limit.maximum).unwrap_or(usize::MAX)
            || !assignments
                .keys()
                .all(|column| writable_columns.contains(column))
            || assignments
                .values()
                .enumerate()
                .any(|(slot, parameter)| usize::try_from(parameter.slot) != Ok(slot))
        {
            return Err(FragmentBatchError::new(
                "dynamic assignments must be non-empty, writable, bounded, and in slot order",
            ));
        }
        Ok(Self {
            relation,
            assignments,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentBatchError {
    pub message: String,
}

impl FragmentBatchError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FragmentBatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FragmentBatchError {}
