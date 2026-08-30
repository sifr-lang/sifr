#![allow(clippy::expect_used)]

use sifr_sql_runtime::{
    BoundParameters, Connection, ConstraintKind, ExecutionMetadata, ExecutionMode,
    ExecutionRequest, OwnedParameter, OwnedSqlValue, Pool, ProviderFuture, ProviderLeaseToken,
    ResourceLimitKind, ResourceUsage, RetryClassification, RuntimeCardinality, RuntimeCodec,
    RuntimeCodecIdentity, RuntimeEffect, RuntimeEffectContract, RuntimeLimits, SafeSqlIdentifier,
    SqlError, SqlErrorKind, SqlErrorMetadata, SqlState, Unverified, VerificationEvidence, Verified,
    catch_codec_boundary,
};
use static_assertions::{assert_impl_all, assert_not_impl_any};
use std::future::Future;
use std::panic;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

struct App;

assert_impl_all!(Pool<App, Verified>: Clone, Send, Sync);
assert_not_impl_any!(Connection<App, Verified>: Clone, Send, Sync);

fn verified_pool() -> Pool<App, Verified> {
    let profile = "a".repeat(64);
    let schema = "b".repeat(64);
    let pool = Pool::<App, Unverified>::new(profile.as_str())
        .expect("test profile identity should be valid");
    pool.verify(VerificationEvidence::new(profile, schema).expect("test evidence should be valid"))
        .expect("matching evidence should verify")
}

#[test]
fn verified_pool_is_share_safe_while_leases_are_task_scoped() {
    let pool = verified_pool();
    let clone = pool.clone();
    assert_eq!(clone.profile_fingerprint(), "a".repeat(64));
    assert_eq!(clone.schema_fingerprint(), "b".repeat(64));
    let mut connection =
        pool.attach_lease(ProviderLeaseToken::new("lease-1").expect("test lease should be valid"));
    {
        let transaction = connection.begin();
        assert!(transaction.is_active());
        assert_eq!(transaction.lease_id(), "lease-1");
        transaction.finish();
    }
    assert_eq!(connection.lease_id(), "lease-1");
}

#[test]
fn bound_parameters_are_owned_ordered_and_duplicate_free() {
    let codec = RuntimeCodecIdentity::new("postgresql.text.v1").expect("valid codec");
    let bound = BoundParameters::new(vec![
        OwnedParameter {
            slot: 1,
            codec: codec.clone(),
            value: OwnedSqlValue::Text("owned".to_string()),
        },
        OwnedParameter {
            slot: 0,
            codec,
            value: OwnedSqlValue::Bytes(Arc::from(&b"bytes"[..])),
        },
    ])
    .expect("distinct owned parameters should bind");
    assert_eq!(bound.as_slice()[0].slot, 0);
    assert_eq!(bound.as_slice()[1].slot, 1);
    assert!(
        BoundParameters::new(vec![OwnedParameter {
            slot: 1,
            codec: RuntimeCodecIdentity::new("postgresql.text.v1").expect("valid codec"),
            value: OwnedSqlValue::Text("gap".to_string()),
        }])
        .is_err()
    );
}

#[test]
fn error_display_is_stable_and_never_renders_provider_metadata() {
    let metadata = SqlErrorMetadata {
        sql_state: Some(SqlState::new("23505").expect("valid SQL state")),
        vendor_code: Some(1062),
        constraint_kind: Some(ConstraintKind::Unique),
        constraint_identity: Some(
            SafeSqlIdentifier::new("public.users_email_key").expect("safe identity"),
        ),
        table_identity: None,
        columns: Vec::new(),
        retry: RetryClassification::Never,
        resource_limit: None,
        cardinality: None,
    };
    let error = SqlError::with_metadata(SqlErrorKind::Constraint, metadata)
        .expect("printable metadata is structurally safe");
    assert_eq!(error.to_string(), "database constraint was violated");
    assert!(!error.to_string().contains("users_email_key"));
    assert!(SafeSqlIdentifier::new("postgresql://user:secret@private.example/db").is_err());
}

struct PanicCodec;

impl RuntimeCodec for PanicCodec {
    type Value = String;

    fn encode(&self, _value: &Self::Value) -> Result<OwnedSqlValue, SqlError> {
        panic!("provider secret")
    }

    fn decode(&self, _value: &OwnedSqlValue) -> Result<Self::Value, SqlError> {
        Err(SqlError::new(SqlErrorKind::Decode))
    }
}

struct TextCodec;

impl RuntimeCodec for TextCodec {
    type Value = String;

    fn encode(&self, value: &Self::Value) -> Result<OwnedSqlValue, SqlError> {
        Ok(OwnedSqlValue::Text(value.clone()))
    }

    fn decode(&self, value: &OwnedSqlValue) -> Result<Self::Value, SqlError> {
        let OwnedSqlValue::Text(value) = value else {
            return Err(SqlError::new(SqlErrorKind::Decode));
        };
        Ok(value.clone())
    }
}

#[test]
fn codec_panics_are_redacted_and_malformed_values_remain_typed_errors() {
    let codec = PanicCodec;
    let panic_error = catch_codec_boundary(|| codec.encode(&"secret".to_string()))
        .expect_err("panic must become a typed error");
    assert_eq!(panic_error.kind(), SqlErrorKind::Provider);
    let decode_error = catch_codec_boundary(|| codec.decode(&OwnedSqlValue::Null))
        .expect_err("malformed value must remain a typed error");
    assert_eq!(decode_error.kind(), SqlErrorKind::Decode);

    let codec = TextCodec;
    let encoded = catch_codec_boundary(|| codec.encode(&"round trip".to_string()))
        .expect("valid text should encode");
    assert_eq!(
        catch_codec_boundary(|| codec.decode(&encoded)).expect("valid text should decode"),
        "round trip",
    );
    assert_eq!(
        catch_codec_boundary(|| codec.decode(&OwnedSqlValue::Signed(1)))
            .expect_err("wrong wire value should fail")
            .kind(),
        SqlErrorKind::Decode,
    );
}

#[test]
fn asynchronous_provider_panics_are_caught_at_poll_time() {
    let mut future = ProviderFuture::<()>::new(async { panic!("driver panic with secret") });
    let mut context = Context::from_waker(Waker::noop());
    let result = Future::poll(Pin::new(&mut future), &mut context);
    let Poll::Ready(Err(error)) = result else {
        panic!("provider panic should immediately become an error");
    };
    assert_eq!(error.kind(), SqlErrorKind::Provider);
    assert_eq!(error.to_string(), "database provider failed");
}

#[test]
fn synchronous_provider_panics_are_caught_before_future_creation() {
    let mut future = ProviderFuture::<()>::from_factory(|| -> std::future::Ready<_> {
        panic!("provider factory secret")
    });
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Err(error)) = Future::poll(Pin::new(&mut future), &mut context) else {
        panic!("provider factory panic should become a typed error");
    };
    assert_eq!(error.kind(), SqlErrorKind::Provider);
}

#[test]
fn common_runtime_has_no_dialect_semantic_fallback() {
    let effects = RuntimeEffectContract::new(RuntimeEffect::Read, Vec::new(), Vec::new())
        .expect("empty read effect should validate");
    assert_eq!(effects.effect, RuntimeEffect::Read);
    assert_eq!(
        RuntimeCardinality::new(0, None).expect("valid"),
        RuntimeCardinality {
            minimum: 0,
            maximum: None,
        },
    );
    let caught = panic::catch_unwind(|| SqlState::new("secret"));
    assert!(caught.is_ok());
    assert!(caught.expect("constructor should not panic").is_err());
}

#[test]
fn execution_shape_requires_explicit_compatible_fetch_method() {
    let parameters = BoundParameters::new(vec![OwnedParameter {
        slot: 0,
        codec: RuntimeCodecIdentity::new("postgresql.text.v1").expect("valid codec"),
        value: OwnedSqlValue::Text("parameter-secret".to_string()),
    }])
    .expect("one parameter should bind");
    let request = ExecutionRequest {
        profile: Arc::new(App),
        statement: Arc::from("select 'statement-secret'"),
        parameters,
        cardinality: RuntimeCardinality::new(0, Some(1)).expect("valid cardinality"),
        effects: RuntimeEffectContract::new(RuntimeEffect::Read, Vec::new(), Vec::new())
            .expect("valid read effect"),
        returns_rows: true,
        metadata: ExecutionMetadata {
            normalized_statement_fingerprint: "c".repeat(64),
            parameter_type_fingerprint: "d".repeat(64),
            result_type_fingerprint: "e".repeat(64),
            schema_fingerprint: "f".repeat(64),
        },
        mode: ExecutionMode::FetchOptional,
    };
    assert!(request.validate().is_ok());
    let rendered = format!("{request:?}");
    assert!(!rendered.contains("parameter-secret"));
    assert!(!rendered.contains("statement-secret"));
    assert!(rendered.contains("Text { len: 16 }"));
    let incompatible = ExecutionRequest {
        mode: ExecutionMode::Execute,
        ..request
    };
    assert_eq!(
        incompatible
            .validate()
            .expect_err("execute cannot consume row results")
            .kind(),
        SqlErrorKind::Cardinality,
    );
}

#[test]
fn resource_accounting_fails_before_a_bound_is_exceeded() {
    let limits = RuntimeLimits {
        max_decoded_row_bytes: 4,
        max_collected_rows: 1,
        max_parameters: 1,
        ..RuntimeLimits::default()
    }
    .validate()
    .expect("positive limits should validate");
    let mut usage = ResourceUsage::default();
    usage
        .account_parameters(1, limits)
        .expect("one parameter should fit");
    usage.account_row(4, limits).expect("one row should fit");
    assert_eq!(usage.largest_decoded_row_bytes(), 4);
    let error = usage
        .account_row(1, limits)
        .expect_err("second row should exceed the bound");
    assert_eq!(error.kind(), SqlErrorKind::ResourceLimit);
    assert_eq!(
        error.metadata().resource_limit,
        Some(ResourceLimitKind::CollectedRows),
    );
}
