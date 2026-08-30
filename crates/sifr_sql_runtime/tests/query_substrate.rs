#![allow(clippy::expect_used)]

use sifr_sql_runtime::{
    BoundParameters, BoundQuery, EncodeParameters, ExecutionMetadata, ExecutionMode, OwnedSqlValue,
    QueryTemplate, RuntimeCardinality, RuntimeCodecIdentity, RuntimeEffect, RuntimeEffectContract,
    SqlError, SqlErrorKind,
};
use static_assertions::{assert_impl_all, assert_not_impl_any};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
struct CloneCapture(BoundParameters);

impl EncodeParameters for CloneCapture {
    fn encode(self) -> Result<BoundParameters, SqlError> {
        Ok(self.0)
    }
}

struct LinearCapture(BoundParameters);

impl EncodeParameters for LinearCapture {
    fn encode(self) -> Result<BoundParameters, SqlError> {
        Ok(self.0)
    }
}

assert_impl_all!(BoundQuery<(), CloneCapture>: Clone);
assert_not_impl_any!(BoundQuery<(), LinearCapture>: Clone);

#[test]
fn binding_evaluates_once_left_to_right_and_owns_values() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let first_order = Rc::clone(&order);
    let second_order = Rc::clone(&order);
    let bound = template(RuntimeCardinality::new(0, None).expect("valid many cardinality"))
        .bind_encoded_with(|encoder| {
            encoder.capture(codec(), || {
                first_order.borrow_mut().push("first");
                Ok(OwnedSqlValue::Text("owned-first".to_string()))
            })?;
            encoder.capture(codec(), || {
                second_order.borrow_mut().push("second");
                Ok(OwnedSqlValue::Bytes(Arc::from(&b"owned-second"[..])))
            })
        })
        .expect("ordered binding");
    assert_eq!(&*order.borrow(), &["first", "second"]);

    let debug = format!("{bound:?}");
    assert!(!debug.contains("owned-first"));
    assert!(!debug.contains("owned-second"));
    let request = bound
        .into_execution_request(ExecutionMode::FetchAll { maximum_rows: 10 })
        .expect("consuming execution lowering");
    assert_eq!(request.parameters.as_slice().len(), 2);
}

#[test]
fn failed_capture_stops_before_later_expression() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let first = Rc::clone(&order);
    let second = Rc::clone(&order);
    let result = template(RuntimeCardinality::new(0, None).expect("valid many cardinality"))
        .bind_encoded_with(|encoder| {
            encoder.capture(codec(), || {
                first.borrow_mut().push("first");
                Err(SqlError::new(SqlErrorKind::Encode))
            })?;
            encoder.capture(codec(), || {
                second.borrow_mut().push("second");
                Ok(OwnedSqlValue::Bool(true))
            })
        });
    assert!(result.is_err());
    assert_eq!(&*order.borrow(), &["first"]);
}

#[test]
fn execution_consumes_bound_query_and_preserves_cardinality_and_effects() {
    let bound = template(RuntimeCardinality::new(0, None).expect("valid many cardinality"))
        .bind(CloneCapture(BoundParameters::default()))
        .expect_at_most_one();
    let cloned = bound.clone();
    let request = cloned
        .into_execution_request(ExecutionMode::FetchOptional)
        .expect("narrowed query supports optional fetch");
    assert_eq!(request.cardinality.minimum, 0);
    assert_eq!(request.cardinality.maximum, Some(1));
    assert_eq!(request.effects.effect, RuntimeEffect::Read);
    assert_eq!(
        request.effects.referenced_objects.as_ref(),
        ["public.users"]
    );

    let original = bound
        .into_execution_request(ExecutionMode::FetchOptional)
        .expect("clone leaves original valid");
    assert_eq!(original.metadata.schema_fingerprint, "d".repeat(64));
}

fn template(cardinality: RuntimeCardinality) -> QueryTemplate<()> {
    QueryTemplate::new(
        Arc::new(()),
        "SELECT active FROM users WHERE active = $1",
        cardinality,
        RuntimeEffectContract::new(
            RuntimeEffect::Read,
            vec!["public.users".to_string()],
            Vec::new(),
        )
        .expect("valid read effect"),
        true,
        ExecutionMetadata {
            normalized_statement_fingerprint: "a".repeat(64),
            parameter_type_fingerprint: "b".repeat(64),
            result_type_fingerprint: "c".repeat(64),
            schema_fingerprint: "d".repeat(64),
        },
    )
    .expect("valid template")
}

fn codec() -> RuntimeCodecIdentity {
    RuntimeCodecIdentity::new("postgresql.bool.v1").expect("valid codec")
}
