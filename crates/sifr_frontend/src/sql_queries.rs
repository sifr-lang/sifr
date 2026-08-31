use sifr_ir::{
    HirExpr, HirSqlBoundQuery, HirSqlCardinality, HirSqlEffectContract, HirSqlEffectKind,
    HirSqlExecution, HirSqlExecutionMethod, HirSqlParameterSlot, HirSqlQueryAdapter,
    HirSqlQueryTemplate,
};
use sifr_sql_contract::{
    Cardinality, CodecRegistry, EffectContract, FetchMethod, ProfileModuleRegistry,
    ProviderAnalysis, QueryAdapter, QueryContractError, QueryContractErrorKind, QueryEffect,
    QueryOrigin, QueryTemplateContract, QueryTemplateDraft, RegisteredProfileModule, SifrType,
};
use sifr_type_system::{FixedIntType, StructuralRecordType, Type};

pub struct QueryCompilationInput<'a> {
    pub profile_name: &'a str,
    pub origin: QueryOrigin,
    pub analysis: ProviderAnalysis,
    pub codecs: &'a CodecRegistry,
    pub parameter_types: Vec<SifrType>,
    pub deterministic_order: bool,
    pub fragment_identities: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CompiledSqlQuery {
    pub contract: QueryTemplateContract,
    pub hir: HirSqlQueryTemplate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SqlExecutionResourceKind {
    Pool,
    Connection,
    Transaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedSqlExecutionResource {
    profile_identity: String,
    pub kind: SqlExecutionResourceKind,
}

impl VerifiedSqlExecutionResource {
    #[must_use]
    pub fn from_profile(profile: &RegisteredProfileModule, kind: SqlExecutionResourceKind) -> Self {
        Self {
            profile_identity: profile.authority().nominal_identity.clone(),
            kind,
        }
    }

    #[must_use]
    pub fn profile_identity(&self) -> &str {
        &self.profile_identity
    }
}

/// Production frontend consumer for generated profile modules and common SQL
/// query contracts. Dialect components produce `ProviderAnalysis`; this type
/// resolves the selected profile and lowers the validated result to closed HIR.
pub struct SqlQueryCompiler<'a> {
    profiles: &'a ProfileModuleRegistry,
}

/// Unify branch or generic query values through the ordinary Sifr union rules.
/// Query profiles, rows, cardinalities, and effects remain ordinary nominal or
/// structural type arguments. No dynamic or erased SQL query type is created.
#[must_use]
pub fn unify_query_value_types(left: &Type, right: &Type) -> Type {
    sifr_type_system::make_union(vec![left.clone(), right.clone()])
}

impl<'a> SqlQueryCompiler<'a> {
    #[must_use]
    pub fn new(profiles: &'a ProfileModuleRegistry) -> Self {
        Self { profiles }
    }

    pub fn profile(&self, name: &str) -> Result<&'a RegisteredProfileModule, QueryContractError> {
        self.profiles.profile(name).map_err(|error| {
            QueryContractError::new(QueryContractErrorKind::ProfileMismatch, error.message)
        })
    }

    pub fn compile(
        &self,
        input: QueryCompilationInput<'_>,
    ) -> Result<CompiledSqlQuery, QueryContractError> {
        let contract = QueryTemplateContract::compile(
            self.profiles,
            input.profile_name,
            QueryTemplateDraft {
                origin: input.origin,
                analysis: input.analysis,
                parameter_types: input.parameter_types,
                deterministic_order: input.deterministic_order,
                fragment_identities: input.fragment_identities,
            },
            input.codecs,
        )?;
        if !contract.effects.application_safe() {
            return Err(QueryContractError::new(
                QueryContractErrorKind::Effect,
                "application query APIs accept only read, write, and read-write effects",
            ));
        }
        let hir = lower_template(&contract)?;
        Ok(CompiledSqlQuery { contract, hir })
    }

    pub fn bind(
        &self,
        query: &CompiledSqlQuery,
        captures: Vec<HirExpr>,
    ) -> Result<HirSqlBoundQuery, QueryContractError> {
        if captures.len() != query.hir.parameters.len()
            || captures
                .iter()
                .zip(&query.hir.parameters)
                .any(|(capture, parameter)| capture.ty() != &parameter.ty)
        {
            return Err(QueryContractError::new(
                QueryContractErrorKind::BindingOrder,
                "query captures must match parameter slots exactly in source order",
            ));
        }
        Ok(HirSqlBoundQuery {
            template_identity: query.hir.identity.clone(),
            profile_identity: query.hir.profile_identity.clone(),
            profile_fingerprint: query.hir.profile_fingerprint.clone(),
            schema_fingerprint: query.hir.schema_fingerprint.clone(),
            captures,
            cardinality: query.hir.cardinality.clone(),
            effects: query.hir.effects.clone(),
            ty: bound_query_type(&query.hir),
        })
    }

    pub fn execution(
        &self,
        query: HirSqlBoundQuery,
        resource: &VerifiedSqlExecutionResource,
        method: HirSqlExecutionMethod,
    ) -> Result<HirSqlExecution, QueryContractError> {
        if query.profile_identity != resource.profile_identity {
            return Err(QueryContractError::new(
                QueryContractErrorKind::ProfileMismatch,
                "only a verified pool, connection, or transaction with the proving profile can execute this query",
            ));
        }
        let fetch = match method {
            HirSqlExecutionMethod::Execute => FetchMethod::Execute,
            HirSqlExecutionMethod::FetchOne => FetchMethod::FetchOne,
            HirSqlExecutionMethod::FetchOptional => FetchMethod::FetchOptional,
            HirSqlExecutionMethod::FetchAll { maximum_rows } if maximum_rows > 0 => {
                FetchMethod::FetchAll
            }
            HirSqlExecutionMethod::Stream => FetchMethod::Stream,
            HirSqlExecutionMethod::FetchAll { .. } => {
                return Err(QueryContractError::new(
                    QueryContractErrorKind::Cardinality,
                    "fetch_all requires a positive explicit row bound",
                ));
            }
        };
        let cardinality = cardinality_from_hir(&query.cardinality)?;
        let returns_rows = !matches!(method, HirSqlExecutionMethod::Execute);
        if !cardinality.supports(fetch, returns_rows) {
            return Err(QueryContractError::new(
                QueryContractErrorKind::Cardinality,
                "execution method conflicts with the query cardinality",
            ));
        }
        Ok(HirSqlExecution {
            runtime_cardinality: query.cardinality.clone(),
            runtime_effects: query.effects.clone(),
            query,
            method,
        })
    }
}

fn lower_template(
    contract: &QueryTemplateContract,
) -> Result<HirSqlQueryTemplate, QueryContractError> {
    let parameters = contract
        .parameters
        .iter()
        .map(|parameter| {
            Ok(HirSqlParameterSlot {
                slot: parameter.slot,
                ty: lower_sifr_type(&parameter.sifr_type)?,
            })
        })
        .collect::<Result<Vec<_>, QueryContractError>>()?;
    let row_type = Type::StructuralRecord(StructuralRecordType::new(
        contract
            .result_fields
            .iter()
            .map(|field| Ok((field.name.clone(), lower_sifr_type(&field.sifr_type)?)))
            .collect::<Result<Vec<_>, QueryContractError>>()?,
    ));
    let cardinality = lower_cardinality(contract.cardinality);
    let effects = lower_effects(&contract.effects);
    let ty = query_template_type(contract, &parameters, row_type.clone());
    Ok(HirSqlQueryTemplate {
        identity: contract.identity.as_str().to_string(),
        module: contract.origin.module.clone(),
        symbol: contract.origin.symbol.clone(),
        profile_identity: contract.profile_identity.clone(),
        profile_fingerprint: contract.profile_fingerprint.clone(),
        schema_fingerprint: contract.schema_fingerprint.clone(),
        normalized_statement: contract.normalized_statement.clone(),
        parameters,
        row_type,
        cardinality,
        effects,
        deterministic_order: contract.deterministic_order,
        fragment_identities: contract.fragment_identities.clone(),
        adapters: contract
            .adapters
            .iter()
            .copied()
            .map(lower_adapter)
            .collect(),
        ty,
    })
}

fn lower_cardinality(cardinality: Cardinality) -> HirSqlCardinality {
    match cardinality {
        Cardinality::Empty => HirSqlCardinality {
            empty: true,
            minimum: 0,
            maximum: Some(0),
        },
        Cardinality::Interval { minimum, maximum } => HirSqlCardinality {
            empty: false,
            minimum,
            maximum,
        },
    }
}

fn cardinality_from_hir(
    cardinality: &HirSqlCardinality,
) -> Result<Cardinality, QueryContractError> {
    if cardinality.empty {
        if cardinality.minimum != 0 || cardinality.maximum != Some(0) {
            return Err(QueryContractError::new(
                QueryContractErrorKind::Cardinality,
                "empty query cardinality has an invalid runtime representation",
            ));
        }
        return Ok(Cardinality::Empty);
    }
    Cardinality::new(cardinality.minimum, cardinality.maximum).map_err(|error| {
        QueryContractError::new(QueryContractErrorKind::Cardinality, error.to_string())
    })
}

fn lower_effects(effects: &EffectContract) -> HirSqlEffectContract {
    HirSqlEffectContract {
        effect: match effects.effect {
            QueryEffect::Read => HirSqlEffectKind::Read,
            QueryEffect::Write => HirSqlEffectKind::Write,
            QueryEffect::ReadWrite => HirSqlEffectKind::ReadWrite,
            QueryEffect::SchemaChange => HirSqlEffectKind::SchemaChange,
            QueryEffect::SessionChange => HirSqlEffectKind::SessionChange,
            QueryEffect::TransactionControl => HirSqlEffectKind::TransactionControl,
        },
        referenced_objects: effects
            .referenced_objects
            .iter()
            .map(ToString::to_string)
            .collect(),
        affected_objects: effects
            .affected_objects
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

fn lower_adapter(adapter: QueryAdapter) -> HirSqlQueryAdapter {
    match adapter {
        QueryAdapter::ExpectAtMostOne => HirSqlQueryAdapter::ExpectAtMostOne,
        QueryAdapter::First => HirSqlQueryAdapter::First,
    }
}

fn query_template_type(
    contract: &QueryTemplateContract,
    parameters: &[HirSqlParameterSlot],
    row_type: Type,
) -> Type {
    nominal_type(
        "sifr.sql.QueryTemplate",
        vec![
            nominal_type(&contract.profile_identity, Vec::new()),
            Type::Tuple(
                parameters
                    .iter()
                    .map(|parameter| parameter.ty.clone())
                    .collect(),
            ),
            row_type,
            nominal_type(&cardinality_identity(&contract.cardinality), Vec::new()),
            nominal_type(&effect_identity(contract.effects.effect), Vec::new()),
        ],
    )
}

fn bound_query_type(template: &HirSqlQueryTemplate) -> Type {
    nominal_type(
        "sifr.sql.BoundQuery",
        vec![
            nominal_type(&template.profile_identity, Vec::new()),
            template.row_type.clone(),
            nominal_type(&cardinality_hir_identity(&template.cardinality), Vec::new()),
            nominal_type(&effect_hir_identity(template.effects.effect), Vec::new()),
        ],
    )
}

fn lower_sifr_type(ty: &SifrType) -> Result<Type, QueryContractError> {
    Ok(match ty {
        SifrType::Bool => Type::Bool,
        SifrType::FixedInteger { sign, width } => Type::FixedInt(match (sign, width) {
            (sifr_sql_contract::IntegerSign::Signed, sifr_sql_contract::IntegerWidth::Bits8) => {
                FixedIntType::I8
            }
            (sifr_sql_contract::IntegerSign::Signed, sifr_sql_contract::IntegerWidth::Bits16) => {
                FixedIntType::I16
            }
            (sifr_sql_contract::IntegerSign::Signed, sifr_sql_contract::IntegerWidth::Bits32) => {
                FixedIntType::I32
            }
            (sifr_sql_contract::IntegerSign::Signed, sifr_sql_contract::IntegerWidth::Bits64) => {
                FixedIntType::I64
            }
            (sifr_sql_contract::IntegerSign::Unsigned, sifr_sql_contract::IntegerWidth::Bits8) => {
                FixedIntType::U8
            }
            (sifr_sql_contract::IntegerSign::Unsigned, sifr_sql_contract::IntegerWidth::Bits16) => {
                FixedIntType::U16
            }
            (sifr_sql_contract::IntegerSign::Unsigned, sifr_sql_contract::IntegerWidth::Bits32) => {
                FixedIntType::U32
            }
            (sifr_sql_contract::IntegerSign::Unsigned, sifr_sql_contract::IntegerWidth::Bits64) => {
                FixedIntType::U64
            }
        }),
        SifrType::ExactInteger => Type::Int,
        SifrType::Decimal => Type::Decimal,
        SifrType::BigDecimal => Type::BigDecimal,
        SifrType::Numeric => nominal_type("sifr.sql.Numeric", Vec::new()),
        SifrType::Float => Type::Float,
        SifrType::Str => Type::Str,
        SifrType::Bytes => Type::Bytes,
        SifrType::None => Type::None,
        SifrType::List { element } => Type::List(Box::new(lower_sifr_type(element)?)),
        SifrType::SqlArray { element } => {
            nominal_type("sifr.sql.SqlArray", vec![lower_sifr_type(element)?])
        }
        SifrType::Nominal { identity } => nominal_type(identity.as_str(), Vec::new()),
        SifrType::Range {
            element,
            multirange,
        } => nominal_type(
            if *multirange {
                "sifr.sql.MultiRange"
            } else {
                "sifr.sql.Range"
            },
            vec![lower_sifr_type(element)?],
        ),
        SifrType::Custom { identity } => nominal_type(identity, Vec::new()),
        SifrType::Union { members } => Type::Union(
            members
                .iter()
                .map(lower_sifr_type)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        SifrType::Date => nominal_type("sifr.datetime.date", Vec::new()),
        SifrType::LocalTime => nominal_type("sifr.datetime.time", Vec::new()),
        SifrType::OffsetTime => nominal_type("sifr.datetime.offset_time", Vec::new()),
        SifrType::LocalDateTime => nominal_type("sifr.datetime.datetime", Vec::new()),
        SifrType::Instant => nominal_type("sifr.datetime.instant", Vec::new()),
        SifrType::CalendarInterval => nominal_type("sifr.sql.CalendarInterval", Vec::new()),
        SifrType::Uuid => nominal_type("sifr.uuid.UUID", Vec::new()),
        SifrType::JsonValue => nominal_type("sifr.json.JsonValue", Vec::new()),
        SifrType::IpAddress => nominal_type("sifr.ipaddress.IPAddress", Vec::new()),
        SifrType::IpNetwork => nominal_type("sifr.ipaddress.IPNetwork", Vec::new()),
        SifrType::MacAddress => nominal_type("sifr.sql.MacAddress", Vec::new()),
    })
}

fn nominal_type(identity: &str, type_args: Vec<Type>) -> Type {
    Type::Class {
        identity: Some(identity.to_string()),
        type_args,
        name: identity.rsplit('.').next().unwrap_or(identity).to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }
}

fn cardinality_identity(cardinality: &Cardinality) -> String {
    match cardinality {
        Cardinality::Empty => "sifr.sql.cardinality.Empty".to_string(),
        Cardinality::Interval { minimum, maximum } => format!(
            "sifr.sql.cardinality.Interval_{minimum}_{}",
            maximum.map_or_else(|| "many".to_string(), |value| value.to_string())
        ),
    }
}

fn cardinality_hir_identity(cardinality: &HirSqlCardinality) -> String {
    if cardinality.empty {
        "sifr.sql.cardinality.Empty".to_string()
    } else {
        format!(
            "sifr.sql.cardinality.Interval_{}_{}",
            cardinality.minimum,
            cardinality
                .maximum
                .map_or_else(|| "many".to_string(), |value| value.to_string())
        )
    }
}

const fn effect_name(effect: QueryEffect) -> &'static str {
    match effect {
        QueryEffect::Read => "Read",
        QueryEffect::Write => "Write",
        QueryEffect::ReadWrite => "ReadWrite",
        QueryEffect::SchemaChange => "SchemaChange",
        QueryEffect::SessionChange => "SessionChange",
        QueryEffect::TransactionControl => "TransactionControl",
    }
}

fn effect_identity(effect: QueryEffect) -> String {
    format!("sifr.sql.effect.{}", effect_name(effect))
}

const fn effect_hir_name(effect: HirSqlEffectKind) -> &'static str {
    match effect {
        HirSqlEffectKind::Read => "Read",
        HirSqlEffectKind::Write => "Write",
        HirSqlEffectKind::ReadWrite => "ReadWrite",
        HirSqlEffectKind::SchemaChange => "SchemaChange",
        HirSqlEffectKind::SessionChange => "SessionChange",
        HirSqlEffectKind::TransactionControl => "TransactionControl",
    }
}

fn effect_hir_identity(effect: HirSqlEffectKind) -> String {
    format!("sifr.sql.effect.{}", effect_hir_name(effect))
}
