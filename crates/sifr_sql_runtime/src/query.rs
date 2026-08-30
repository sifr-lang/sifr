use crate::{
    BoundParameters, ExecutionMetadata, ExecutionMode, ExecutionRequest, OwnedParameter,
    OwnedSqlValue, ParameterError, RuntimeCardinality, RuntimeCodecIdentity, RuntimeEffectContract,
    SqlError, SqlErrorKind,
};
use std::fmt;
use std::sync::Arc;

/// Captured parameter storage generated for one reusable query call.
///
/// Code generation can retain typed owned values in a capture struct and defer
/// fallible encoding until execution. `BoundQuery` is cloneable only when this
/// capture type is cloneable.
pub trait EncodeParameters {
    fn encode(self) -> Result<BoundParameters, SqlError>;
}

impl EncodeParameters for BoundParameters {
    fn encode(self) -> Result<BoundParameters, SqlError> {
        Ok(self)
    }
}

pub struct QueryTemplate<P> {
    profile: Arc<P>,
    statement: Arc<str>,
    cardinality: RuntimeCardinality,
    effects: RuntimeEffectContract,
    returns_rows: bool,
    metadata: ExecutionMetadata,
}

impl<P> Clone for QueryTemplate<P> {
    fn clone(&self) -> Self {
        Self {
            profile: Arc::clone(&self.profile),
            statement: Arc::clone(&self.statement),
            cardinality: self.cardinality,
            effects: self.effects.clone(),
            returns_rows: self.returns_rows,
            metadata: self.metadata.clone(),
        }
    }
}

impl<P> QueryTemplate<P> {
    pub fn new(
        profile: Arc<P>,
        statement: impl Into<Arc<str>>,
        cardinality: RuntimeCardinality,
        effects: RuntimeEffectContract,
        returns_rows: bool,
        metadata: ExecutionMetadata,
    ) -> Result<Self, SqlError> {
        let template = Self {
            profile,
            statement: statement.into(),
            cardinality,
            effects,
            returns_rows,
            metadata,
        };
        template.validate_shape()?;
        Ok(template)
    }

    #[must_use]
    pub fn bind<C>(self, captures: C) -> BoundQuery<P, C> {
        BoundQuery {
            template: self,
            captures,
        }
    }

    /// Evaluate and own encoded parameters in explicit source order.
    ///
    /// The callback is the code-generation target for template holes. Each
    /// `capture` call evaluates one expression immediately, so a failure stops
    /// before any later expression runs.
    pub fn bind_encoded_with(
        self,
        bind: impl FnOnce(&mut OrderedParameterEncoder) -> Result<(), SqlError>,
    ) -> Result<BoundQuery<P, BoundParameters>, SqlError> {
        let mut encoder = OrderedParameterEncoder::default();
        bind(&mut encoder)?;
        let parameters = encoder.finish()?;
        Ok(self.bind(parameters))
    }

    fn validate_shape(&self) -> Result<(), SqlError> {
        if self.statement.trim().is_empty() {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        let mode = if self.returns_rows {
            ExecutionMode::FetchAll {
                maximum_rows: u64::MAX,
            }
        } else {
            ExecutionMode::Execute
        };
        let request = ExecutionRequest {
            profile: Arc::clone(&self.profile),
            statement: Arc::clone(&self.statement),
            parameters: BoundParameters::default(),
            cardinality: self.cardinality,
            effects: self.effects.clone(),
            returns_rows: self.returns_rows,
            metadata: self.metadata.clone(),
            mode,
        };
        request.validate()
    }
}

impl<P> fmt::Debug for QueryTemplate<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueryTemplate")
            .field("statement_len", &self.statement.len())
            .field("cardinality", &self.cardinality)
            .field("effects", &self.effects)
            .field("returns_rows", &self.returns_rows)
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

pub struct BoundQuery<P, C> {
    template: QueryTemplate<P>,
    captures: C,
}

impl<P, C> BoundQuery<P, C> {
    #[must_use]
    pub fn expect_at_most_one(mut self) -> Self {
        self.template.cardinality.maximum = Some(1);
        self.template.cardinality.minimum = self.template.cardinality.minimum.min(1);
        self
    }
}

impl<P, C: EncodeParameters> BoundQuery<P, C> {
    /// Consume the bound query and produce the only provider execution shape.
    pub fn into_execution_request(
        self,
        mode: ExecutionMode,
    ) -> Result<ExecutionRequest<P>, SqlError> {
        let request = ExecutionRequest {
            profile: self.template.profile,
            statement: self.template.statement,
            parameters: self.captures.encode()?,
            cardinality: self.template.cardinality,
            effects: self.template.effects,
            returns_rows: self.template.returns_rows,
            metadata: self.template.metadata,
            mode,
        };
        request.validate()?;
        Ok(request)
    }
}

impl<P, C: Clone> Clone for BoundQuery<P, C> {
    fn clone(&self) -> Self {
        Self {
            template: self.template.clone(),
            captures: self.captures.clone(),
        }
    }
}

impl<P, C> fmt::Debug for BoundQuery<P, C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundQuery")
            .field("template", &self.template)
            .field("captures", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct OrderedParameterEncoder {
    parameters: Vec<OwnedParameter>,
}

impl OrderedParameterEncoder {
    pub fn capture(
        &mut self,
        codec: RuntimeCodecIdentity,
        evaluate: impl FnOnce() -> Result<OwnedSqlValue, SqlError>,
    ) -> Result<(), SqlError> {
        let slot = u32::try_from(self.parameters.len())
            .map_err(|_| SqlError::new(SqlErrorKind::ResourceLimit))?;
        let value = evaluate()?;
        self.parameters.push(OwnedParameter { slot, codec, value });
        Ok(())
    }

    fn finish(self) -> Result<BoundParameters, SqlError> {
        BoundParameters::new(self.parameters).map_err(parameter_error)
    }
}

fn parameter_error(error: ParameterError) -> SqlError {
    let kind = match error {
        ParameterError::DuplicateSlot
        | ParameterError::InvalidExactInteger
        | ParameterError::InvalidTypeIdentity => SqlErrorKind::Encode,
    };
    SqlError::new(kind)
}
