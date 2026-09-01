use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use crate::project::ProjectLowering;
use ruff_text_size::TextRange;
use sifr_compiler_component::{
    AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ComponentHost, ComponentHostLimits,
    EmbeddedAnalysisRequest, HoleDescriptor, PlanKind, SourceSpan, TemplatePart,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{QueryCompilationInput, SqlQueryCompiler, TemplateSourceMapKind};
use sifr_sql_contract::{
    QueryOrigin, QuerySignatureRegistry, QuerySymbol, QuerySymbolKind, component_codec_registry,
    provider_analysis_from_response,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct PlannedQuery {
    module_name: String,
    declaration: sifr_frontend::SqlQueryDeclaration,
    portable: Option<PortableProofPlan>,
}

#[derive(Clone)]
struct PortableProofPlan {
    requirement_name: String,
}

pub(super) fn compile_application_queries(
    project: &mut ProjectLowering,
    profiles: &super::sql_profiles::PreparedSqlProfiles,
) -> Result<QuerySignatureRegistry, Vec<RenderedDiagnostic>> {
    let profile_names = profiles
        .registry()
        .entries()
        .map(|(name, _)| name.to_string())
        .collect::<BTreeSet<_>>();
    let mut declarations = Vec::new();
    let mut portable_plans = BTreeMap::new();
    let mut imported_portable_plans = BTreeMap::new();
    let mut local_profile_names = BTreeMap::new();
    for module_name in &project.compile_order {
        let Some(module) = project.hir_modules.get(module_name) else {
            return Err(error("compiled module is missing from SQL query discovery"));
        };
        let profile_locals = sifr_frontend::sql_profile_local_names(module, &profile_names);
        for mut declaration in sifr_frontend::sql_query_declarations(module).map_err(error)? {
            declaration.profile_name = profile_locals
                .get(&declaration.profile_name)
                .cloned()
                .ok_or_else(|| {
                    error(format!(
                        "SQL query '{}' names unknown profile namespace '{}'",
                        declaration.symbol, declaration.profile_name
                    ))
                })?;
            declarations.push(PlannedQuery {
                module_name: module_name.clone(),
                declaration,
                portable: None,
            });
        }
        let (portable_declarations, specializations) =
            sifr_frontend::portable_sql_query_plan(module, &profile_names).map_err(error)?;
        portable_plans.insert(
            module_name.clone(),
            (portable_declarations, specializations),
        );
        local_profile_names.insert(
            module_name.clone(),
            profile_locals.keys().cloned().collect::<BTreeSet<_>>(),
        );
    }
    let portable_declarations = portable_plans
        .iter()
        .map(|(module, (declarations, _))| (module.clone(), declarations.clone()))
        .collect::<BTreeMap<_, _>>();
    for module_name in &project.compile_order {
        let module = project
            .hir_modules
            .get(module_name)
            .ok_or_else(|| error("compiled module is missing from SQL import discovery"))?;
        let imported = sifr_frontend::imported_portable_sql_query_plan(
            module,
            &portable_declarations,
            &profile_names,
        )
        .map_err(error)?;
        for item in &imported {
            let (_, owner_specializations) = portable_plans
                .get_mut(&item.owner_module)
                .ok_or_else(|| error("portable SQL import names an unknown project module"))?;
            owner_specializations.push(item.specialization.clone());
        }
        imported_portable_plans.insert(module_name.clone(), imported);
    }
    for (module_name, (portable_declarations, specializations)) in &mut portable_plans {
        specializations.sort();
        specializations.dedup();
        for specialization in specializations.iter() {
            let declaration = portable_declarations
                .iter()
                .find(|candidate| candidate.symbol == specialization.symbol)
                .ok_or_else(|| error("portable SQL specialization lost its declaration"))?;
            declarations.push(PlannedQuery {
                module_name: module_name.clone(),
                declaration: sifr_frontend::SqlQueryDeclaration {
                    symbol: specialization.specialized_symbol.clone(),
                    profile_name: specialization.profile_name.clone(),
                    exported: false,
                    document: declaration
                        .document
                        .clone()
                        .with_profile(specialization.profile_name.clone()),
                    parameter_types: declaration.parameter_types.clone(),
                },
                portable: Some(PortableProofPlan {
                    requirement_name: canonical_requirement_name(
                        profiles,
                        &specialization.requirement_name,
                    )?,
                }),
            });
        }
    }
    for module_name in &project.compile_order {
        let module = project
            .hir_modules
            .get(module_name)
            .ok_or_else(|| error("compiled module is missing from SQL witness validation"))?;
        let local = portable_plans
            .get(module_name)
            .map(|(_, specializations)| specializations.as_slice())
            .unwrap_or_default();
        let imported = imported_portable_plans
            .get(module_name)
            .map(Vec::as_slice)
            .unwrap_or_default();
        sifr_frontend::validate_profile_witness_consumption(
            module,
            local,
            imported,
            &profile_names,
        )
        .map_err(error)?;
    }
    if declarations.is_empty() {
        for (module_name, module) in &mut project.hir_modules {
            if let Some(imported) = imported_portable_plans.get(module_name) {
                sifr_frontend::apply_imported_portable_sql_query_plan(module, imported)
                    .map_err(error)?;
            }
            if let Some((portable_declarations, specializations)) = portable_plans.get(module_name)
            {
                sifr_frontend::apply_portable_sql_query_plan(
                    module,
                    portable_declarations,
                    specializations,
                )
                .map_err(error)?;
            }
            let names = local_profile_names
                .get(module_name)
                .unwrap_or(&profile_names);
            sifr_frontend::erase_compiler_sql_surfaces(module, names);
        }
        return Ok(QuerySignatureRegistry::default());
    }
    let limits = ComponentHostLimits {
        fuel: 100_000_000,
        ..ComponentHostLimits::default()
    };
    let mut host = ComponentHost::new(limits, None)
        .map_err(|failure| error(format!("cannot initialize SQL query compiler: {failure}")))?;
    let compiler = SqlQueryCompiler::new(profiles.registry());
    let mut signatures = QuerySignatureRegistry::default();
    for planned in declarations {
        let PlannedQuery {
            module_name,
            declaration,
            portable,
        } = planned;
        let registered = profiles
            .registry()
            .profile(&declaration.profile_name)
            .map_err(|failure| error(failure.to_string()))?;
        let component = profiles
            .query_component(&declaration.profile_name)
            .ok_or_else(|| error("SQL profile has no resolved query component"))?;
        let context = profiles
            .schema_context(&declaration.profile_name)
            .ok_or_else(|| error("SQL profile has no normalized schema context"))?;
        let request = request_for_declaration(
            &module_name,
            &declaration,
            registered,
            component.registration.clone(),
            context.clone(),
        )?;
        let response = host
            .analyze(&component.registration, &component.bytes, &request)
            .map_err(|failure| error(format!("SQL query component failed: {failure}")))?
            .response;
        if !response.plan.diagnostics.is_empty() {
            let messages = response
                .plan
                .diagnostics
                .iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(error(messages));
        }
        let analysis = provider_analysis_from_response(&response)
            .map_err(|failure| error(failure.to_string()))?;
        let codecs = component_codec_registry(&analysis, &declaration.parameter_types)
            .map_err(|failure| error(failure.to_string()))?;
        let start = declaration.document.template.source_range.start().to_u32();
        let end = declaration.document.template.source_range.end().to_u32();
        let input = QueryCompilationInput {
            profile_name: &declaration.profile_name,
            origin: QueryOrigin::new(module_name.clone(), declaration.symbol.clone(), start, end)
                .map_err(|failure| error(failure.to_string()))?,
            deterministic_order: analysis.semantic_flags.contains("deterministic-order"),
            analysis,
            codecs: &codecs,
            parameter_types: declaration.parameter_types,
            fragment_identities: Vec::new(),
        };
        let contract = if let Some(portable) = portable {
            let witness = sifr_frontend::SqlSchemaWitness::from_profile_export(registered);
            sifr_frontend::SchemaPolymorphicQueryCompiler::new(
                profiles.registry(),
                profiles.requirements(),
            )
            .specialize(sifr_frontend::SchemaSpecializationInput {
                requirement_name: &portable.requirement_name,
                profile_name: &declaration.profile_name,
                witness: &witness,
                query: input,
            })
            .map_err(|failure| error(failure.message))?
            .query
            .contract
        } else {
            compiler
                .compile(input)
                .map_err(|failure| error(failure.to_string()))?
                .contract
        };
        signatures
            .register(QuerySymbol {
                module: module_name,
                name: declaration.symbol,
                kind: QuerySymbolKind::TopLevelReusable,
                exported: declaration.exported,
                template: contract,
            })
            .map_err(|failure| error(failure.to_string()))?;
    }
    for (module_name, module) in &mut project.hir_modules {
        if let Some(imported) = imported_portable_plans.get(module_name) {
            sifr_frontend::apply_imported_portable_sql_query_plan(module, imported)
                .map_err(error)?;
        }
        if let Some((portable_declarations, specializations)) = portable_plans.get(module_name) {
            sifr_frontend::apply_portable_sql_query_plan(
                module,
                portable_declarations,
                specializations,
            )
            .map_err(error)?;
        }
        let names = local_profile_names
            .get(module_name)
            .unwrap_or(&profile_names);
        sifr_frontend::erase_compiler_sql_surfaces(module, names);
    }
    Ok(signatures)
}

fn canonical_requirement_name(
    profiles: &super::sql_profiles::PreparedSqlProfiles,
    source_name: &str,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let candidates = profiles
        .requirements()
        .entries()
        .filter(|(canonical, requirement)| {
            *canonical == source_name || requirement.identity.name == source_name
        })
        .map(|(canonical, _)| canonical.to_string())
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [canonical] => Ok(canonical.clone()),
        [] => Err(error(format!(
            "schema requirement '{source_name}' is not registered"
        ))),
        _ => Err(error(format!(
            "schema requirement '{source_name}' is ambiguous; import a unique reachable requirement"
        ))),
    }
}

fn request_for_declaration(
    module_name: &str,
    declaration: &sifr_frontend::SqlQueryDeclaration,
    profile: &sifr_sql_contract::RegisteredProfileModule,
    registration: sifr_compiler_component::ComponentRegistration,
    context_artifact: sifr_compiler_component::ContextArtifact,
) -> Result<EmbeddedAnalysisRequest, Vec<RenderedDiagnostic>> {
    let parts = template_parts(module_name, &declaration.document.template)?;
    let holes = declaration
        .document
        .parameter_protocol_types
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            Ok(HoleDescriptor {
                index: u32::try_from(index)
                    .map_err(|_| error("SQL query has too many interpolation holes"))?,
                ty: ty
                    .clone()
                    .ok_or_else(|| error("SQL interpolation type is not component-safe"))?,
                fragment_identity: None,
            })
        })
        .collect::<Result<Vec<_>, Vec<RenderedDiagnostic>>>()?;
    let authority = profile.authority();
    let schema = &authority.profile.schema;
    Ok(EmbeddedAnalysisRequest {
        protocol_major: COMPONENT_PROTOCOL_MAJOR,
        component: registration.identity,
        provider_diagnostics: registration.diagnostics,
        compiler_semantic_version: env!("CARGO_PKG_VERSION").to_string(),
        parts,
        holes,
        context: AnalysisContext {
            schema_profile: Some(authority.nominal_identity.clone()),
            schema_fingerprint: Some(authority.schema_fingerprint.as_str().to_string()),
            semantic_profile: BTreeMap::from([
                (
                    "server-version".to_string(),
                    schema.dialect.server_version.clone(),
                ),
                (
                    "modes".to_string(),
                    serde_json::to_string(&schema.dialect.modes)
                        .map_err(|failure| error(failure.to_string()))?,
                ),
                (
                    "features".to_string(),
                    serde_json::to_string(&schema.dialect.features)
                        .map_err(|failure| error(failure.to_string()))?,
                ),
                (
                    "strictness".to_string(),
                    format!("{:?}", authority.profile.strictness),
                ),
                (
                    "session".to_string(),
                    serde_json::to_string(&authority.profile.session)
                        .map_err(|failure| error(failure.to_string()))?,
                ),
            ]),
            imported_signatures: Vec::new(),
            artifacts: vec![context_artifact],
        },
        plan_kind: PlanKind::Expression,
    })
}

fn template_parts(
    document: &str,
    template: &sifr_frontend::TemplateDocumentView,
) -> Result<Vec<TemplatePart>, Vec<RenderedDiagnostic>> {
    let mut mappings = template.mappings.clone();
    mappings.sort_by_key(|mapping| mapping.virtual_range.start());
    mappings
        .into_iter()
        .map(|mapping| {
            let span = SourceSpan {
                document: document.to_string(),
                start: mapping.source_range.start().to_u32(),
                end: mapping.source_range.end().to_u32(),
            };
            match mapping.kind {
                TemplateSourceMapKind::Static => Ok(TemplatePart::Static {
                    text: virtual_text(&template.source, mapping.virtual_range)
                        .ok_or_else(|| error("SQL template source map is out of bounds"))?
                        .to_string(),
                    span,
                }),
                TemplateSourceMapKind::Interpolation { index } => Ok(TemplatePart::Hole {
                    index: u32::try_from(index)
                        .map_err(|_| error("SQL query has too many interpolation holes"))?,
                    span,
                }),
            }
        })
        .collect()
}

fn virtual_text(source: &str, range: TextRange) -> Option<&str> {
    source.get(
        usize::try_from(range.start().to_u32()).ok()?
            ..usize::try_from(range.end().to_u32()).ok()?,
    )
}

fn error(message: impl Into<String>) -> Vec<RenderedDiagnostic> {
    vec![diagnostic_with_code(
        message,
        DiagnosticCode::COMPONENT_EXECUTION,
    )]
}
