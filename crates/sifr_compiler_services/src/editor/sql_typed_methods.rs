//! Typed SQL expressions at the outer compiler-service boundary.
use crate::sql_editor::PreparedSqlProfiles;
use sifr_compiler_component::{ComponentHost, ComponentHostLimits};
use sifr_frontend::{
    QueryCompilationInput, SqlEditorDocumentView, SqlQueryCompiler, SqlQueryDeclaration,
};
use sifr_ir::{HirExpr, HirSqlExecutionMethod};
use sifr_lowering::{TypedMethodProcessor, TypedMethodRequest};
use sifr_sql_contract::{QueryOrigin, component_codec_registry, provider_analysis_from_response};
use sifr_type_system::Type;

#[derive(Debug)]
pub(crate) struct SqlTypedMethods {
    profiles: PreparedSqlProfiles,
    methods: std::sync::Mutex<PreparedMethods>,
}

impl SqlTypedMethods {
    pub(crate) fn new(profiles: PreparedSqlProfiles) -> Self {
        Self {
            profiles,
            methods: std::sync::Mutex::new(PreparedMethods::default()),
        }
    }

    fn profile_name<'a>(&'a self, ty: &Type) -> Option<&'a str> {
        let Type::Class {
            identity: Some(identity),
            ..
        } = ty.resolve_alias()
        else {
            return None;
        };
        self.profiles
            .registry()
            .entries()
            .find_map(|(name, registered)| {
                (identity == &format!("sifr.sql.schemas.{name}")
                    && registered.authority().profile.schema.dialect.family == "sqlite")
                    .then_some(name)
            })
    }

    fn query(
        &self,
        module: &str,
        owner: &str,
        profile: &str,
        args: &[HirExpr],
    ) -> Result<HirExpr, String> {
        let [HirExpr::TemplateString(template)] = args else {
            return Err("SQL construction requires one literal typed template".into());
        };
        let document = SqlEditorDocumentView::from_hir(template).with_profile(profile.to_string());
        let parameter_types = template
            .interpolations
            .iter()
            .map(|i| sifr_frontend::sql_contract_type(&i.value_type))
            .collect::<Result<Vec<_>, _>>()?;
        let declaration = SqlQueryDeclaration {
            symbol: owner.into(),
            profile_name: profile.into(),
            exported: !owner.starts_with('_'),
            document,
            parameter_types: parameter_types.clone(),
        };
        let registered = self
            .profiles
            .registry()
            .profile(profile)
            .map_err(|e| e.to_string())?;
        let component = self
            .profiles
            .query_component(profile)
            .ok_or("SQL query component is missing")?;
        let context = self
            .profiles
            .schema_context(profile)
            .ok_or("SQL schema context is missing")?;
        let request = super::sql_application_queries::request_for_declaration(
            module,
            &declaration,
            registered,
            component.registration.clone(),
            context.clone(),
        )
        .map_err(|diagnostics| format!("{diagnostics:?}"))?;
        let mut host = ComponentHost::new(
            ComponentHostLimits {
                fuel: 100_000_000,
                ..ComponentHostLimits::default()
            },
            None,
        )
        .map_err(|e| e.to_string())?;
        let response = host
            .analyze(&component.registration, &component.bytes, &request)
            .map_err(|e| e.to_string())?
            .response;
        if !response.plan.diagnostics.is_empty() {
            return Err(response
                .plan
                .diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; "));
        }
        let analysis = provider_analysis_from_response(&response).map_err(|e| e.to_string())?;
        let codecs =
            component_codec_registry(&analysis, &parameter_types).map_err(|e| e.to_string())?;
        if analysis.effects.effect != sifr_sql_contract::QueryEffect::Read {
            return Err("SQLite application execution currently supports read queries".into());
        }
        let compiler = SqlQueryCompiler::new(self.profiles.registry());
        let query = compiler
            .compile(QueryCompilationInput {
                profile_name: profile,
                origin: QueryOrigin::new(
                    module,
                    owner,
                    template.source_range.start().to_u32(),
                    template.source_range.end().to_u32(),
                )
                .map_err(|e| e.to_string())?,
                deterministic_order: analysis.semantic_flags.contains("deterministic-order"),
                analysis,
                codecs: &codecs,
                parameter_types,
                fragment_identities: Vec::new(),
            })
            .map_err(|e| e.to_string())?;
        // Only codecs with executable native value representations enter this path.
        for slot in &query.hir.parameters {
            validate_scalar(&slot.ty)?;
        }
        if let Type::StructuralRecord(record) = &query.hir.row_type {
            for field in record.fields() {
                validate_scalar(field.ty())?;
            }
        }
        let captures = template
            .interpolations
            .iter()
            .map(|i| (*i.value).clone())
            .collect();
        let bound = compiler.bind(&query, captures).map_err(|e| e.to_string())?;
        let mut ty = bound.ty;
        if let Type::Class { type_args, .. } = &mut ty {
            type_args.push(nominal(&query.hir.identity, Vec::new()));
        }
        let descriptor = serde_json::json!({
            "statement": query.hir.normalized_statement,
            "profile": query.hir.profile_fingerprint,
            "schema": query.hir.schema_fingerprint,
            "minimum": query.hir.cardinality.minimum,
            "maximum": query.hir.cardinality.maximum,
            "effect": format!("{:?}", query.hir.effects.effect),
            "referenced": query.hir.effects.referenced_objects,
            "affected": query.hir.effects.affected_objects,
            "parameter_types": query.hir.parameters.iter().map(|slot| slot.ty.to_string()).collect::<Vec<_>>(),
            "result_types": query.contract.result_fields,
        }).to_string();
        Ok(HirExpr::ConstructorCall {
            class_name: "__sifr_sql_bound".into(),
            args: vec![HirExpr::StringLiteral(descriptor), args[0].clone()],
            ty,
        })
    }

    fn connect(&self, profile: &str, args: &[HirExpr]) -> Result<HirExpr, String> {
        if args.len() != 1 || args[0].ty() != &Type::Str {
            return Err("connect requires one database path string".into());
        }
        let authority = self
            .profiles
            .registry()
            .profile(profile)
            .map_err(|e| e.to_string())?
            .authority();
        if authority.profile.evidence != sifr_sql_contract::SchemaEvidence::Introspection
            || authority.profile.strictness != sifr_sql_contract::SchemaStrictness::Exact
        {
            return Err("SQLite application connect requires exact introspection evidence".into());
        }
        if authority.profile.schema.objects.values().any(|object| {
            !matches!(
                object.kind,
                sifr_sql_contract::SchemaObjectKind::Namespace
                    | sifr_sql_contract::SchemaObjectKind::Catalog
                    | sifr_sql_contract::SchemaObjectKind::DialectMetadata
                    | sifr_sql_contract::SchemaObjectKind::ServerCapability
            )
        }) {
            return Err("SQLite connect currently supports empty application catalogs; populated profiles require provider runtime introspection".into());
        }
        let ty = Type::Awaitable(Box::new(Type::Result(
            Box::new(nominal(
                "sifr.sql.Pool",
                vec![
                    nominal(&authority.nominal_identity, Vec::new()),
                    nominal("sifr.sql.Verified", Vec::new()),
                ],
            )),
            Box::new(sql_error()),
        )));
        Ok(HirExpr::ConstructorCall {
            class_name: "__sifr_sql_connect".into(),
            args: vec![
                args[0].clone(),
                HirExpr::StringLiteral(authority.profile_fingerprint.as_str().into()),
                HirExpr::StringLiteral(authority.schema_fingerprint.as_str().into()),
            ],
            ty,
        })
    }

    fn execute(object: HirExpr, method: &str, mut args: Vec<HirExpr>) -> Result<HirExpr, String> {
        let Type::Class {
            type_args: pool_args,
            ..
        } = object.ty().resolve_alias()
        else {
            return Err("SQL execution requires a verified pool".into());
        };
        if pool_args.len() != 2
            || !matches!(&pool_args[1], Type::Class {
            identity: Some(identity), ..
        } if identity == "sifr.sql.Verified")
        {
            return Err("SQL execution requires a verified pool".into());
        }
        let Some(query) = args.first() else {
            return Err("SQL execution requires a bound query".into());
        };
        let Type::Class {
            identity: Some(identity),
            type_args,
            ..
        } = query.ty().resolve_alias()
        else {
            return Err("SQL execution requires a bound query".into());
        };
        if identity != "sifr.sql.BoundQuery" || type_args.len() != 5 {
            return Err("SQL execution requires a compiler-approved bound query".into());
        }
        if pool_args.first() != type_args.first() {
            return Err("SQL query and verified pool profile identities differ".into());
        }
        let Type::Class {
            identity: Some(query_id),
            ..
        } = &type_args[4]
        else {
            return Err("SQL query identity is missing".into());
        };
        if query_id.len() != 64 || !query_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("SQL query identity is invalid".into());
        }
        let mode = match method {
            "fetch_one" => HirSqlExecutionMethod::FetchOne,
            "fetch_optional" => HirSqlExecutionMethod::FetchOptional,
            "fetch_all" if args.len() == 2 => {
                let HirExpr::IntLiteral(value) = &args[1] else {
                    return Err("fetch_all requires a positive literal max_rows bound".into());
                };
                let bound = value
                    .to_string()
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or("fetch_all requires a positive literal max_rows bound")?;
                HirSqlExecutionMethod::FetchAll {
                    maximum_rows: bound,
                }
            }
            _ => return Err("unsupported SQL execution method or missing row bound".into()),
        };
        if args.len() != if method == "fetch_all" { 2 } else { 1 } {
            return Err("invalid SQL execution arguments".into());
        }
        let cardinality = sifr_frontend::sql_cardinality_from_type(&type_args[2])?;
        if !matches!(&type_args[3], Type::Class { identity: Some(identity), .. } if identity == "sifr.sql.effect.Read")
        {
            return Err("SQLite application execution currently supports read queries".into());
        }
        let fetch = match mode {
            HirSqlExecutionMethod::FetchOne => sifr_sql_contract::FetchMethod::FetchOne,
            HirSqlExecutionMethod::FetchOptional => sifr_sql_contract::FetchMethod::FetchOptional,
            _ => sifr_sql_contract::FetchMethod::FetchAll,
        };
        if !cardinality.supports(fetch, true) {
            return Err("execution method conflicts with the query cardinality".into());
        }
        let row = type_args[1].clone();
        let result = match mode {
            HirSqlExecutionMethod::FetchOptional => Type::Union(vec![row, Type::None]),
            HirSqlExecutionMethod::FetchAll { .. } => Type::List(Box::new(row)),
            _ => row,
        };
        let ty = Type::Awaitable(Box::new(Type::Result(
            Box::new(result),
            Box::new(sql_error()),
        )));
        args.insert(0, object);
        Ok(HirExpr::ConstructorCall {
            class_name: format!("__sifr_sql_{method}"),
            args,
            ty,
        })
    }
}

#[derive(Debug, Default)]
struct PreparedMethods {
    pending: std::collections::BTreeMap<String, OwnedMethodRequest>,
    expressions: std::collections::BTreeMap<String, Result<HirExpr, String>>,
}
#[derive(Debug)]
struct OwnedMethodRequest {
    module: String,
    owner: String,
    object: HirExpr,
    method: String,
    args: Vec<HirExpr>,
    keywords: Vec<Option<String>>,
}

impl TypedMethodProcessor for SqlTypedMethods {
    fn handles(&self, receiver: &Type, method: &str) -> bool {
        self.profile_name(receiver).is_some() && matches!(method, "sql" | "connect")
            || matches!(receiver.resolve_alias(), Type::Class { identity: Some(identity), .. } if identity == "sifr.sql.Pool")
    }
    fn compile(&self, request: TypedMethodRequest<'_>) -> Result<HirExpr, String> {
        // Include the complete typed source expression, not merely its range: edited
        // templates and dependent inferred types must never reuse stale facts.
        let key = format!("{request:?}");
        let mut methods = self
            .methods
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(expression) = methods.expressions.get(&key) {
            return expression.clone();
        }
        methods
            .pending
            .entry(key)
            .or_insert_with(|| OwnedMethodRequest {
                module: request.module.into(),
                owner: request.owner.into(),
                object: request.object,
                method: request.method.into(),
                args: request.args,
                keywords: request.keywords,
            });
        Err("typed method awaits outer frontend preparation".into())
    }
    fn prepare_pending(&self) -> bool {
        let mut methods = self
            .methods
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if methods.pending.is_empty() {
            return false;
        }
        let requests = std::mem::take(&mut methods.pending);
        for (key, request) in requests {
            methods
                .expressions
                .insert(key, self.prepare_method(request));
        }
        true
    }
}

impl SqlTypedMethods {
    fn prepare_method(&self, request: OwnedMethodRequest) -> Result<HirExpr, String> {
        let OwnedMethodRequest {
            module,
            owner,
            object,
            method,
            args,
            keywords,
        } = request;
        if keywords.iter().enumerate().any(|(index, keyword)| {
            keyword
                .as_ref()
                .is_some_and(|name| method != "fetch_all" || index != 1 || name != "max_rows")
        }) {
            return Err("unsupported SQL keyword argument".into());
        }
        if let Some(profile) = self.profile_name(object.ty()) {
            return match method.as_str() {
                "sql" => self.query(&module, &owner, profile, &args),
                "connect" => self.connect(profile, &args),
                _ => Err("unsupported SQL constructor".into()),
            };
        }
        Self::execute(object, &method, args)
    }
}

fn nominal(identity: &str, type_args: Vec<Type>) -> Type {
    Type::Class {
        identity: Some(identity.into()),
        type_args,
        name: identity.rsplit('.').next().unwrap_or(identity).into(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}
fn sql_error() -> Type {
    nominal("sifr.sql.SqlError", Vec::new())
}
fn validate_scalar(ty: &Type) -> Result<(), String> {
    match ty.resolve_alias() {
        Type::Bool
        | Type::FixedInt(sifr_type_system::FixedIntType::I64)
        | Type::Str
        | Type::Bytes
        | Type::Float
        | Type::None => Ok(()),
        Type::Union(members)
            if members.iter().any(|member| member == &Type::None) && members.len() == 2 =>
        {
            for member in members {
                validate_scalar(member)?;
            }
            Ok(())
        }
        _ => Err(format!(
            "SQL native execution codec for '{ty}' is not supported"
        )),
    }
}
