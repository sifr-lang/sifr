use crate::{
    DialectIdentity, EffectContract, ObjectId, QueryContractError, QueryContractErrorKind,
    QueryParameterSlot, QueryTemplateIdentity, SifrType,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentCategory {
    Expression,
    Predicate,
    Identifier,
    Relation,
    OrderBy,
    Join,
    SelectList,
    AssignmentList,
    Values,
    ReturningList,
    Query,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SqlPrecedence(u8);

impl SqlPrecedence {
    pub const QUERY: Self = Self(0);
    pub const OR: Self = Self(10);
    pub const AND: Self = Self(20);
    pub const NOT: Self = Self(30);
    pub const COMPARISON: Self = Self(40);
    pub const ADDITIVE: Self = Self(50);
    pub const MULTIPLICATIVE: Self = Self(60);
    pub const UNARY: Self = Self(70);
    pub const ATOM: Self = Self(80);

    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AliasIdentity(String);

impl AliasIdentity {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FragmentIdentity(String);

impl FragmentIdentity {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticFragmentOrigin {
    QueryDefinition,
    RuntimeBranch,
    RuntimeLoop,
    RuntimeContainer,
    ReturnedValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationAlias {
    pub identity: AliasIdentity,
    pub query_scope: String,
    pub source_name: String,
    pub relation: ObjectId,
}

#[derive(Clone, Debug)]
pub struct QueryDefinitionScope {
    identity: String,
    next_alias: u32,
    aliases: BTreeSet<AliasIdentity>,
}

impl QueryDefinitionScope {
    pub fn new(identity: impl Into<String>) -> Result<Self, QueryContractError> {
        let identity = identity.into();
        if identity.is_empty() || identity.len() > 160 || identity.chars().any(char::is_control) {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentScope,
                "query fragment scope identity is invalid",
            ));
        }
        Ok(Self {
            identity,
            next_alias: 0,
            aliases: BTreeSet::new(),
        })
    }

    pub fn relation_alias(
        &mut self,
        relation: ObjectId,
        source_name: impl Into<String>,
        origin: StaticFragmentOrigin,
    ) -> Result<RelationAlias, QueryContractError> {
        if origin != StaticFragmentOrigin::QueryDefinition {
            return Err(fragment_error(
                QueryContractErrorKind::AliasEscape,
                "relation aliases can be created only at a static query-definition site",
            ));
        }
        let source_name = source_name.into();
        if source_name.is_empty() || source_name.chars().any(char::is_control) {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentScope,
                "relation alias source name is invalid",
            ));
        }
        let identity = AliasIdentity(format!("{}:alias:{}", self.identity, self.next_alias));
        self.next_alias = self.next_alias.checked_add(1).ok_or_else(|| {
            fragment_error(
                QueryContractErrorKind::FragmentScope,
                "query contains too many relation aliases",
            )
        })?;
        self.aliases.insert(identity.clone());
        Ok(RelationAlias {
            identity,
            query_scope: self.identity.clone(),
            source_name,
            relation,
        })
    }

    #[must_use]
    pub fn aliases(&self) -> &BTreeSet<AliasIdentity> {
        &self.aliases
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResultTransformation {
    Preserve,
    Replace { fields: BTreeMap<String, SifrType> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectTransformation {
    Preserve,
    Replace { contract: EffectContract },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsafeSyntaxLint {
    Warn,
    Deny,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnsafeSyntaxAudit {
    pub package_identity: String,
    pub capability: String,
    pub lint: UnsafeSyntaxLint,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsafeSyntaxGrant {
    audit: UnsafeSyntaxAudit,
}

pub trait PackageCapabilityResolver {
    fn allows(&self, package_identity: &str, capability: &str) -> bool;
}

impl UnsafeSyntaxGrant {
    pub fn from_package_resolver(
        resolver: &impl PackageCapabilityResolver,
        package_identity: impl Into<String>,
        lint: UnsafeSyntaxLint,
        reason: impl Into<String>,
    ) -> Result<Self, QueryContractError> {
        let package_identity = package_identity.into();
        let audit = UnsafeSyntaxAudit {
            package_identity: package_identity.clone(),
            capability: "sql.unsafe-syntax".to_string(),
            lint,
            reason: reason.into(),
        };
        if audit.package_identity.is_empty()
            || !resolver.allows(&package_identity, "sql.unsafe-syntax")
            || audit.reason.trim().is_empty()
            || audit.lint == UnsafeSyntaxLint::Deny
        {
            return Err(fragment_error(
                QueryContractErrorKind::UnsafeSyntax,
                "unsafe SQL syntax requires the sql.unsafe-syntax capability, an audit reason, and a non-deny lint policy",
            ));
        }
        Ok(Self { audit })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlFragment {
    pub identity: FragmentIdentity,
    pub query_identity: QueryTemplateIdentity,
    pub profile_identity: String,
    pub dialect: DialectIdentity,
    pub category: FragmentCategory,
    pub input_scope: BTreeSet<AliasIdentity>,
    pub output_scope: BTreeSet<AliasIdentity>,
    pub required_aliases: BTreeSet<AliasIdentity>,
    pub introduced_aliases: BTreeSet<AliasIdentity>,
    pub free_identifiers: BTreeSet<ObjectId>,
    pub parameters: Vec<QueryParameterSlot>,
    pub result: ResultTransformation,
    pub effect: EffectTransformation,
    pub precedence: SqlPrecedence,
    pub canonical_syntax: String,
    pub unsafe_audit: Option<UnsafeSyntaxAudit>,
}

#[derive(Clone, Debug)]
pub struct FragmentDraft {
    pub query_identity: QueryTemplateIdentity,
    pub profile_identity: String,
    pub dialect: DialectIdentity,
    pub category: FragmentCategory,
    pub input_scope: BTreeSet<AliasIdentity>,
    pub output_scope: BTreeSet<AliasIdentity>,
    pub required_aliases: BTreeSet<AliasIdentity>,
    pub introduced_aliases: BTreeSet<AliasIdentity>,
    pub free_identifiers: BTreeSet<ObjectId>,
    pub parameters: Vec<QueryParameterSlot>,
    pub result: ResultTransformation,
    pub effect: EffectTransformation,
    pub precedence: SqlPrecedence,
    pub canonical_syntax: String,
    pub origin: StaticFragmentOrigin,
}

impl SqlFragment {
    pub fn checked(draft: FragmentDraft) -> Result<Self, QueryContractError> {
        if draft.origin != StaticFragmentOrigin::QueryDefinition {
            return Err(fragment_error(
                QueryContractErrorKind::AliasEscape,
                "fragment identities cannot be created by runtime control flow or escape a query definition",
            ));
        }
        validate_draft(&draft)?;
        let identity = fragment_identity(&draft, None)?;
        Ok(Self {
            identity,
            query_identity: draft.query_identity,
            profile_identity: draft.profile_identity,
            dialect: draft.dialect,
            category: draft.category,
            input_scope: draft.input_scope,
            output_scope: draft.output_scope,
            required_aliases: draft.required_aliases,
            introduced_aliases: draft.introduced_aliases,
            free_identifiers: draft.free_identifiers,
            parameters: draft.parameters,
            result: draft.result,
            effect: draft.effect,
            precedence: draft.precedence,
            canonical_syntax: draft.canonical_syntax,
            unsafe_audit: None,
        })
    }

    pub fn unsafe_checked(
        draft: FragmentDraft,
        grant: &UnsafeSyntaxGrant,
    ) -> Result<Self, QueryContractError> {
        validate_draft(&draft)?;
        if draft.origin != StaticFragmentOrigin::QueryDefinition {
            return Err(fragment_error(
                QueryContractErrorKind::AliasEscape,
                "unsafe fragments remain static query-definition values",
            ));
        }
        let identity = fragment_identity(&draft, Some(&grant.audit))?;
        Ok(Self {
            identity,
            query_identity: draft.query_identity,
            profile_identity: draft.profile_identity,
            dialect: draft.dialect,
            category: draft.category,
            input_scope: draft.input_scope,
            output_scope: draft.output_scope,
            required_aliases: draft.required_aliases,
            introduced_aliases: draft.introduced_aliases,
            free_identifiers: draft.free_identifiers,
            parameters: draft.parameters,
            result: draft.result,
            effect: draft.effect,
            precedence: draft.precedence,
            canonical_syntax: draft.canonical_syntax,
            unsafe_audit: Some(grant.audit.clone()),
        })
    }

    pub fn validate_insertion(
        &self,
        expected: FragmentCategory,
        query_identity: &QueryTemplateIdentity,
        profile_identity: &str,
        dialect: &DialectIdentity,
        available_scope: &BTreeSet<AliasIdentity>,
    ) -> Result<(), QueryContractError> {
        if self.category != expected {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentCategory,
                format!(
                    "fragment category {:?} cannot fill a {:?} hole",
                    self.category, expected
                ),
            ));
        }
        if &self.query_identity != query_identity
            || self.profile_identity != profile_identity
            || &self.dialect != dialect
        {
            return Err(fragment_error(
                QueryContractErrorKind::ProfileMismatch,
                "fragment profile, dialect, or query identity does not match the insertion site",
            ));
        }
        if !self.input_scope.is_subset(available_scope)
            || !self.required_aliases.is_subset(available_scope)
            || !self.introduced_aliases.is_disjoint(available_scope)
        {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentScope,
                "fragment relation scope or hygienic aliases do not match the insertion site",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PredicateContext {
    pub query_identity: QueryTemplateIdentity,
    pub profile_identity: String,
    pub dialect: DialectIdentity,
    pub scope: BTreeSet<AliasIdentity>,
}

pub fn all_predicates(
    context: &PredicateContext,
    predicates: Vec<SqlFragment>,
) -> Result<SqlFragment, QueryContractError> {
    combine_predicates(context, predicates, "TRUE", " AND ", SqlPrecedence::AND)
}

pub fn any_predicates(
    context: &PredicateContext,
    predicates: Vec<SqlFragment>,
) -> Result<SqlFragment, QueryContractError> {
    combine_predicates(context, predicates, "FALSE", " OR ", SqlPrecedence::OR)
}

pub fn not_predicate(
    context: &PredicateContext,
    predicate: SqlFragment,
) -> Result<SqlFragment, QueryContractError> {
    predicate.validate_insertion(
        FragmentCategory::Predicate,
        &context.query_identity,
        &context.profile_identity,
        &context.dialect,
        &context.scope,
    )?;
    let syntax = if predicate.precedence < SqlPrecedence::NOT {
        format!("NOT ({})", predicate.canonical_syntax)
    } else {
        format!("NOT {}", predicate.canonical_syntax)
    };
    combine_predicates(context, vec![predicate], "FALSE", "", SqlPrecedence::NOT).and_then(
        |mut fragment| {
            fragment.canonical_syntax = syntax;
            fragment.identity = recompute_identity(&fragment)?;
            Ok(fragment)
        },
    )
}

fn combine_predicates(
    context: &PredicateContext,
    predicates: Vec<SqlFragment>,
    identity_syntax: &str,
    separator: &str,
    precedence: SqlPrecedence,
) -> Result<SqlFragment, QueryContractError> {
    for predicate in &predicates {
        predicate.validate_insertion(
            FragmentCategory::Predicate,
            &context.query_identity,
            &context.profile_identity,
            &context.dialect,
            &context.scope,
        )?;
    }
    let syntax = if predicates.is_empty() {
        identity_syntax.to_string()
    } else {
        predicates
            .iter()
            .map(|predicate| {
                if predicate.precedence < precedence {
                    format!("({})", predicate.canonical_syntax)
                } else {
                    predicate.canonical_syntax.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(separator)
    };
    let mut parameters = Vec::new();
    let mut required_aliases = BTreeSet::new();
    let mut free_identifiers = BTreeSet::new();
    for predicate in predicates {
        parameters.extend(predicate.parameters);
        required_aliases.extend(predicate.required_aliases);
        free_identifiers.extend(predicate.free_identifiers);
    }
    for (slot, parameter) in parameters.iter_mut().enumerate() {
        parameter.slot = u32::try_from(slot).map_err(|_| {
            fragment_error(
                QueryContractErrorKind::BindingOrder,
                "fragment composition has too many parameter slots",
            )
        })?;
    }
    SqlFragment::checked(FragmentDraft {
        query_identity: context.query_identity.clone(),
        profile_identity: context.profile_identity.clone(),
        dialect: context.dialect.clone(),
        category: FragmentCategory::Predicate,
        input_scope: context.scope.clone(),
        output_scope: context.scope.clone(),
        required_aliases,
        introduced_aliases: BTreeSet::new(),
        free_identifiers,
        parameters,
        result: ResultTransformation::Preserve,
        effect: EffectTransformation::Preserve,
        precedence,
        canonical_syntax: syntax,
        origin: StaticFragmentOrigin::QueryDefinition,
    })
}

fn validate_draft(draft: &FragmentDraft) -> Result<(), QueryContractError> {
    if draft.profile_identity.is_empty()
        || draft.canonical_syntax.trim().is_empty()
        || !draft.input_scope.is_subset(&draft.output_scope)
        || !draft.required_aliases.is_subset(&draft.input_scope)
        || !draft.introduced_aliases.is_subset(&draft.output_scope)
        || !draft.introduced_aliases.is_disjoint(&draft.input_scope)
        || draft
            .parameters
            .iter()
            .enumerate()
            .any(|(slot, parameter)| usize::try_from(parameter.slot) != Ok(slot))
    {
        return Err(fragment_error(
            QueryContractErrorKind::FragmentScope,
            "fragment scope, syntax, or parameter slots are not canonical",
        ));
    }
    if let EffectTransformation::Replace { contract } = &draft.effect {
        contract
            .validate()
            .map_err(|error| fragment_error(QueryContractErrorKind::Effect, error.to_string()))?;
    }
    match draft.category {
        FragmentCategory::Join
            if draft.introduced_aliases.is_empty()
                || draft.output_scope == draft.input_scope
                || !matches!(draft.result, ResultTransformation::Replace { .. }) =>
        {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentScope,
                "a join fragment must introduce relation scope and an exact result transformation",
            ));
        }
        FragmentCategory::SelectList | FragmentCategory::ReturningList
            if !matches!(draft.result, ResultTransformation::Replace { .. }) =>
        {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentCategory,
                "a select-list or returning-list fragment must declare its exact result record",
            ));
        }
        FragmentCategory::AssignmentList | FragmentCategory::Values
            if !matches!(draft.result, ResultTransformation::Preserve) =>
        {
            return Err(fragment_error(
                QueryContractErrorKind::FragmentCategory,
                "assignment and values fragments preserve the query result record",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn fragment_identity(
    draft: &FragmentDraft,
    unsafe_audit: Option<&UnsafeSyntaxAudit>,
) -> Result<FragmentIdentity, QueryContractError> {
    let canonical = serde_json::to_vec(&(
        &draft.query_identity,
        &draft.profile_identity,
        &draft.dialect,
        draft.category,
        &draft.input_scope,
        &draft.output_scope,
        &draft.required_aliases,
        &draft.introduced_aliases,
        &draft.free_identifiers,
        &draft.parameters,
        &draft.result,
        &draft.effect,
        draft.precedence,
        &draft.canonical_syntax,
        unsafe_audit,
    ))
    .map_err(|error| {
        fragment_error(
            QueryContractErrorKind::InvalidTemplate,
            format!("cannot serialize SQL fragment identity: {error}"),
        )
    })?;
    Ok(FragmentIdentity(lower_hex(&Sha256::digest(canonical))))
}

fn recompute_identity(fragment: &SqlFragment) -> Result<FragmentIdentity, QueryContractError> {
    let draft = FragmentDraft {
        query_identity: fragment.query_identity.clone(),
        profile_identity: fragment.profile_identity.clone(),
        dialect: fragment.dialect.clone(),
        category: fragment.category,
        input_scope: fragment.input_scope.clone(),
        output_scope: fragment.output_scope.clone(),
        required_aliases: fragment.required_aliases.clone(),
        introduced_aliases: fragment.introduced_aliases.clone(),
        free_identifiers: fragment.free_identifiers.clone(),
        parameters: fragment.parameters.clone(),
        result: fragment.result.clone(),
        effect: fragment.effect.clone(),
        precedence: fragment.precedence,
        canonical_syntax: fragment.canonical_syntax.clone(),
        origin: StaticFragmentOrigin::QueryDefinition,
    };
    fragment_identity(&draft, fragment.unsafe_audit.as_ref())
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        },
    )
}

fn fragment_error(kind: QueryContractErrorKind, message: impl Into<String>) -> QueryContractError {
    QueryContractError::new(kind, message)
}
