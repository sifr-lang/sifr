use crate::ast::{
    SqliteColumnDefinition, SqliteConflictForm, SqliteCreateTable, SqliteExpression,
    SqliteForeignKeyDefinition, SqliteGeneratedColumn, SqliteKeyDefinition, SqliteNamedDdl,
    SqliteProjection, SqliteQuery, SqliteStatement, SqliteStatementKind, SqliteTypeName,
    SqliteWrite,
};
use crate::lexer::{Keyword, Token};
use crate::parser::{
    RawStatement, SqliteParseError, SqliteParser, is_keyword, normalize_tokens, parse_error,
};

mod token_utils;

use token_utils::{
    contains_sequence, find_keyword, find_top_level_keyword, identifier_path, key_columns,
    matching_right, option_value, optional_key_name, single_identifier, split_top_level,
    token_word, validate_parentheses, value_after,
};

pub(crate) fn lower_statement(
    raw: &RawStatement,
    parser: &SqliteParser,
) -> Result<SqliteStatement, SqliteParseError> {
    validate_parentheses(&raw.tokens, raw.span.start as usize)?;
    let kind = match raw.tokens.first() {
        Some(Token::Keyword(Keyword::Select | Keyword::With)) => {
            SqliteStatementKind::Query(lower_query(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Insert | Keyword::Replace)) => {
            SqliteStatementKind::Insert(lower_insert(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Update)) => {
            SqliteStatementKind::Update(lower_update(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Delete)) => {
            SqliteStatementKind::Delete(lower_delete(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Create)) => lower_create(&raw.tokens, parser)?,
        Some(Token::Keyword(Keyword::Alter)) => {
            SqliteStatementKind::AlterTable(lower_named_ddl(&raw.tokens, Keyword::Table, true)?)
        }
        Some(Token::Keyword(Keyword::Drop)) => {
            let marker = [
                Keyword::Table,
                Keyword::Index,
                Keyword::View,
                Keyword::Trigger,
            ]
            .into_iter()
            .find(|marker| find_keyword(&raw.tokens, *marker, 1).is_some())
            .ok_or_else(|| parse_error(0, "DROP needs an object kind"))?;
            SqliteStatementKind::Drop(lower_named_ddl(&raw.tokens, marker, false)?)
        }
        _ => {
            return Err(parse_error(
                raw.span.start as usize,
                "unsupported SQLite statement",
            ));
        }
    };
    Ok(SqliteStatement {
        kind,
        span: raw.span,
    })
}

fn lower_query(tokens: &[Token]) -> Result<SqliteQuery, SqliteParseError> {
    let select = find_keyword(tokens, Keyword::Select, 0)
        .ok_or_else(|| parse_error(0, "WITH must contain a SELECT query"))?;
    let from = find_top_level_keyword(tokens, Keyword::From, select + 1);
    let projection_end = from.unwrap_or_else(|| first_query_clause(tokens, select + 1));
    let projection_tokens = &tokens[select + 1..projection_end];
    let distinct = projection_tokens
        .first()
        .is_some_and(|token| is_keyword(token, Keyword::Distinct));
    let projections = split_top_level(
        if distinct {
            &projection_tokens[1..]
        } else {
            projection_tokens
        },
        &Token::Comma,
    )
    .into_iter()
    .map(lower_projection)
    .collect::<Result<Vec<_>, _>>()?;
    if projections.is_empty() {
        return Err(parse_error(0, "SELECT needs at least one projection"));
    }

    let relation_end = from.map_or(projection_end, |from_index| {
        next_clause(tokens, from_index + 1).unwrap_or(tokens.len())
    });
    let mut relations = Vec::new();
    let mut joins = Vec::new();
    if let Some(from_index) = from {
        collect_relations(
            &tokens[from_index + 1..relation_end],
            &mut relations,
            &mut joins,
        );
    }
    let predicate = clause_expression(
        tokens,
        Keyword::Where,
        &[
            Keyword::Group,
            Keyword::Having,
            Keyword::Order,
            Keyword::Limit,
            Keyword::For,
            Keyword::Union,
        ],
    );
    let group_by = clause_list(
        tokens,
        Keyword::Group,
        Some(Keyword::By),
        &[
            Keyword::Having,
            Keyword::Order,
            Keyword::Limit,
            Keyword::For,
            Keyword::Union,
        ],
    );
    let having = clause_expression(
        tokens,
        Keyword::Having,
        &[Keyword::Order, Keyword::Limit, Keyword::For, Keyword::Union],
    );
    let order_by = clause_list(
        tokens,
        Keyword::Order,
        Some(Keyword::By),
        &[Keyword::Limit, Keyword::For, Keyword::Union],
    );
    let (limit, comma_offset) = limit_clause(tokens);
    let offset = numeric_after(tokens, Keyword::Offset).or(comma_offset);
    let common_tables = if select > 0 {
        tokens[1..select]
            .iter()
            .filter_map(Token::identifier)
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };
    Ok(SqliteQuery {
        common_tables,
        projections,
        relations,
        joins,
        predicate,
        group_by,
        having,
        order_by,
        limit,
        offset,
        distinct,
        windowed: tokens
            .iter()
            .any(|token| is_keyword(token, Keyword::Window) || is_keyword(token, Keyword::Over)),
        for_update: find_keyword(tokens, Keyword::For, 0).is_some_and(|index| {
            tokens
                .get(index + 1)
                .is_some_and(|token| is_keyword(token, Keyword::Update))
        }),
    })
}

fn lower_projection(tokens: &[Token]) -> Result<SqliteProjection, SqliteParseError> {
    if tokens.is_empty() {
        return Err(parse_error(0, "empty SELECT projection"));
    }
    let alias_index = find_top_level_keyword(tokens, Keyword::As, 0);
    let (expression, alias) = if let Some(index) = alias_index {
        let alias = tokens
            .get(index + 1)
            .and_then(Token::identifier)
            .ok_or_else(|| parse_error(0, "projection AS needs an identifier"))?;
        (&tokens[..index], Some(alias.to_string()))
    } else {
        (tokens, None)
    };
    Ok(SqliteProjection {
        expression: lower_expression(expression),
        alias,
    })
}

fn lower_insert(tokens: &[Token]) -> Result<SqliteWrite, SqliteParseError> {
    let into = find_keyword(tokens, Keyword::Into, 0).unwrap_or(0);
    let (relation, next) = identifier_path(tokens, into + 1)
        .ok_or_else(|| parse_error(0, "INSERT needs a target table"))?;
    let mut columns = Vec::new();
    let mut cursor = next;
    if tokens.get(cursor) == Some(&Token::LeftParen) {
        let end = matching_right(tokens, cursor)
            .ok_or_else(|| parse_error(0, "INSERT column list is not closed"))?;
        columns = split_top_level(&tokens[cursor + 1..end], &Token::Comma)
            .into_iter()
            .map(single_identifier)
            .collect::<Result<Vec<_>, _>>()?;
        cursor = end + 1;
    }
    let expressions = tokens[cursor..]
        .iter()
        .filter(|token| {
            matches!(
                token,
                Token::Parameter(_) | Token::String(_) | Token::Number(_)
            )
        })
        .map(|token| lower_expression(std::slice::from_ref(token)))
        .collect();
    let conflict = if contains_sequence(tokens, &[Keyword::On, Keyword::Conflict])
        && contains_sequence(tokens, &[Keyword::Do, Keyword::Nothing])
    {
        SqliteConflictForm::UpsertDoNothing
    } else if contains_sequence(tokens, &[Keyword::On, Keyword::Conflict])
        && contains_sequence(tokens, &[Keyword::Do, Keyword::Update])
    {
        SqliteConflictForm::UpsertDoUpdate
    } else if is_keyword(&tokens[0], Keyword::Replace) || insert_prefix_is(tokens, Keyword::Replace)
    {
        SqliteConflictForm::Replace
    } else if insert_prefix_is(tokens, Keyword::Ignore) {
        SqliteConflictForm::Ignore
    } else if insert_prefix_is(tokens, Keyword::Rollback) {
        SqliteConflictForm::Rollback
    } else if insert_prefix_is(tokens, Keyword::Abort) {
        SqliteConflictForm::Abort
    } else if insert_prefix_is(tokens, Keyword::Fail) {
        SqliteConflictForm::Fail
    } else {
        SqliteConflictForm::None
    };
    let assignments = assignment_columns(tokens);
    Ok(SqliteWrite {
        relation,
        columns,
        assignments,
        expressions,
        conflict,
        returning: returning_projections(tokens)?,
    })
}

fn insert_prefix_is(tokens: &[Token], action: Keyword) -> bool {
    tokens
        .first()
        .is_some_and(|token| is_keyword(token, Keyword::Insert))
        && tokens
            .get(1)
            .is_some_and(|token| is_keyword(token, Keyword::Or))
        && tokens.get(2).is_some_and(|token| is_keyword(token, action))
}

fn lower_update(tokens: &[Token]) -> Result<SqliteWrite, SqliteParseError> {
    let relation_start = if tokens
        .get(1)
        .is_some_and(|token| is_keyword(token, Keyword::Or))
    {
        3
    } else {
        1
    };
    let (relation, _) = identifier_path(tokens, relation_start)
        .ok_or_else(|| parse_error(0, "UPDATE needs a target table"))?;
    let assignments = assignment_columns(tokens);
    let expressions = collect_update_expressions(tokens);
    Ok(SqliteWrite {
        relation,
        columns: assignments.clone(),
        assignments,
        expressions,
        conflict: if contains_sequence(tokens, &[Keyword::Or, Keyword::Rollback]) {
            SqliteConflictForm::Rollback
        } else if contains_sequence(tokens, &[Keyword::Or, Keyword::Abort]) {
            SqliteConflictForm::Abort
        } else if contains_sequence(tokens, &[Keyword::Or, Keyword::Fail]) {
            SqliteConflictForm::Fail
        } else if contains_sequence(tokens, &[Keyword::Or, Keyword::Ignore]) {
            SqliteConflictForm::Ignore
        } else if contains_sequence(tokens, &[Keyword::Or, Keyword::Replace]) {
            SqliteConflictForm::Replace
        } else {
            SqliteConflictForm::None
        },
        returning: returning_projections(tokens)?,
    })
}

fn lower_delete(tokens: &[Token]) -> Result<SqliteWrite, SqliteParseError> {
    let from = find_keyword(tokens, Keyword::From, 0)
        .ok_or_else(|| parse_error(0, "DELETE needs FROM"))?;
    let (relation, _) = identifier_path(tokens, from + 1)
        .ok_or_else(|| parse_error(0, "DELETE needs a target table"))?;
    Ok(SqliteWrite {
        relation,
        columns: Vec::new(),
        assignments: Vec::new(),
        expressions: collect_write_expressions(tokens),
        conflict: SqliteConflictForm::None,
        returning: returning_projections(tokens)?,
    })
}

fn returning_projections(tokens: &[Token]) -> Result<Vec<SqliteProjection>, SqliteParseError> {
    let Some(index) = find_top_level_keyword(tokens, Keyword::Returning, 0) else {
        return Ok(Vec::new());
    };
    split_top_level(&tokens[index + 1..], &Token::Comma)
        .into_iter()
        .map(lower_projection)
        .collect()
}

fn lower_create(
    tokens: &[Token],
    parser: &SqliteParser,
) -> Result<SqliteStatementKind, SqliteParseError> {
    match tokens.get(1) {
        Some(Token::Keyword(Keyword::Table)) => {
            lower_create_table(tokens, parser).map(SqliteStatementKind::CreateTable)
        }
        Some(Token::Keyword(Keyword::View)) => {
            lower_named_ddl(tokens, Keyword::View, false).map(SqliteStatementKind::CreateView)
        }
        Some(Token::Keyword(Keyword::Index | Keyword::Unique)) => {
            lower_named_ddl(tokens, Keyword::Index, true).map(SqliteStatementKind::CreateIndex)
        }
        Some(Token::Keyword(Keyword::Trigger)) => {
            lower_named_ddl(tokens, Keyword::Trigger, true).map(SqliteStatementKind::CreateTrigger)
        }
        _ => Err(parse_error(0, "unsupported SQLite CREATE form")),
    }
}

fn lower_create_table(
    tokens: &[Token],
    parser: &SqliteParser,
) -> Result<SqliteCreateTable, SqliteParseError> {
    let name_start = if contains_sequence(
        tokens.get(2..5).unwrap_or_default(),
        &[Keyword::If, Keyword::Not, Keyword::Exists],
    ) {
        5
    } else {
        2
    };
    let (name, open) = identifier_path(tokens, name_start)
        .ok_or_else(|| parse_error(0, "CREATE TABLE needs a table name"))?;
    if tokens.get(open) != Some(&Token::LeftParen) {
        return Err(parse_error(0, "CREATE TABLE needs a definition list"));
    }
    let close = matching_right(tokens, open)
        .ok_or_else(|| parse_error(0, "CREATE TABLE definition is not closed"))?;
    let mut table = SqliteCreateTable {
        name,
        columns: Vec::new(),
        primary_key: Vec::new(),
        unique_keys: Vec::new(),
        foreign_keys: Vec::new(),
        checks: Vec::new(),
        indexes: Vec::new(),
        strict: false,
        without_rowid: false,
    };
    for definition in split_top_level(&tokens[open + 1..close], &Token::Comma) {
        lower_table_definition(definition, parser, &mut table)?;
    }
    if table.columns.is_empty() {
        return Err(parse_error(0, "CREATE TABLE needs at least one column"));
    }
    let options = &tokens[close + 1..];
    table.strict = options
        .iter()
        .any(|token| is_keyword(token, Keyword::Strict));
    table.without_rowid = contains_sequence(options, &[Keyword::Without, Keyword::Rowid]);
    Ok(table)
}

fn lower_table_definition(
    tokens: &[Token],
    _parser: &SqliteParser,
    table: &mut SqliteCreateTable,
) -> Result<(), SqliteParseError> {
    let mut cursor = 0;
    let constraint_name = if tokens
        .first()
        .is_some_and(|token| is_keyword(token, Keyword::Constraint))
    {
        let value = tokens
            .get(1)
            .and_then(Token::identifier)
            .ok_or_else(|| parse_error(0, "CONSTRAINT needs a name"))?;
        cursor = 2;
        Some(value.to_string())
    } else {
        None
    };
    match tokens.get(cursor) {
        Some(Token::Keyword(Keyword::Primary)) => {
            table.primary_key = key_columns(tokens, cursor + 1)?;
        }
        Some(Token::Keyword(Keyword::Unique)) => {
            table.unique_keys.push(SqliteKeyDefinition {
                name: constraint_name.or_else(|| optional_key_name(tokens, cursor + 1)),
                columns: key_columns(tokens, cursor + 1)?,
            });
        }
        Some(Token::Keyword(Keyword::Foreign)) => {
            let columns = key_columns(tokens, cursor + 1)?;
            let references = find_keyword(tokens, Keyword::References, cursor)
                .ok_or_else(|| parse_error(0, "FOREIGN KEY needs REFERENCES"))?;
            let (referenced_table, after_table) = identifier_path(tokens, references + 1)
                .ok_or_else(|| parse_error(0, "REFERENCES needs a table"))?;
            let referenced_columns = key_columns(tokens, after_table)?;
            table.foreign_keys.push(SqliteForeignKeyDefinition {
                name: constraint_name,
                columns,
                referenced_table,
                referenced_columns,
            });
        }
        Some(Token::Keyword(Keyword::Check)) => {
            table.checks.push(normalize_tokens(&tokens[cursor + 1..]));
        }
        Some(
            Token::Identifier(_)
            | Token::QuotedIdentifier(_)
            | Token::Keyword(Keyword::Key | Keyword::Index),
        ) => {
            table.columns.push(lower_column(tokens)?);
        }
        _ => return Err(parse_error(0, "unsupported CREATE TABLE definition")),
    }
    Ok(())
}

fn lower_column(tokens: &[Token]) -> Result<SqliteColumnDefinition, SqliteParseError> {
    let name = token_word(tokens.first()).ok_or_else(|| parse_error(0, "column needs a name"))?;
    let mut cursor = 1;
    let mut declared_type = Vec::new();
    while let Some(token) = tokens.get(cursor) {
        if matches!(
            token,
            Token::LeftParen
                | Token::Keyword(
                    Keyword::Primary
                        | Keyword::Not
                        | Keyword::Null
                        | Keyword::Unique
                        | Keyword::Check
                        | Keyword::Default
                        | Keyword::Collate
                        | Keyword::References
                        | Keyword::Generated
                )
        ) {
            break;
        }
        let Some(word) = token_word(Some(token)) else {
            break;
        };
        declared_type.push(word);
        cursor += 1;
    }
    let type_name = declared_type.join(" ");
    let mut parameters = Vec::new();
    if tokens.get(cursor) == Some(&Token::LeftParen) {
        let end = matching_right(tokens, cursor)
            .ok_or_else(|| parse_error(0, "type parameters are not closed"))?;
        parameters = split_top_level(&tokens[cursor + 1..end], &Token::Comma)
            .into_iter()
            .map(normalize_tokens)
            .collect();
        cursor = end + 1;
    }
    let tail = &tokens[cursor..];
    let generated = find_keyword(tail, Keyword::Generated, 0).map(|index| {
        let expression = tail
            .get(index + 1..)
            .and_then(|tokens| {
                tokens
                    .iter()
                    .position(|token| *token == Token::LeftParen)
                    .map(|open| (tokens, open))
            })
            .and_then(|(tokens, open)| {
                matching_right(tokens, open).map(|close| normalize_tokens(&tokens[open + 1..close]))
            })
            .unwrap_or_default();
        SqliteGeneratedColumn {
            expression,
            stored: tail.iter().any(|token| is_keyword(token, Keyword::Stored)),
        }
    });
    Ok(SqliteColumnDefinition {
        name,
        ty: SqliteTypeName {
            name: type_name.clone(),
            parameters,
        },
        nullable: !contains_sequence(tail, &[Keyword::Not, Keyword::Null]),
        primary_key: contains_sequence(tail, &[Keyword::Primary, Keyword::Key]),
        primary_key_desc: contains_sequence(tail, &[Keyword::Primary, Keyword::Key])
            && tail.iter().any(|token| is_keyword(token, Keyword::Desc)),
        auto_increment: tail
            .iter()
            .any(|token| is_keyword(token, Keyword::AutoIncrement)),
        generated,
        default: value_after(tail, Keyword::Default),
        collation: option_value(tail, Keyword::Collate),
    })
}

fn lower_named_ddl(
    tokens: &[Token],
    marker: Keyword,
    relation_required: bool,
) -> Result<SqliteNamedDdl, SqliteParseError> {
    let marker_index = find_keyword(tokens, marker, 0)
        .or_else(|| {
            (marker == Keyword::Index)
                .then(|| find_keyword(tokens, Keyword::Unique, 0))
                .flatten()
        })
        .ok_or_else(|| parse_error(0, "DDL object kind is missing"))?;
    let mut name_start = marker_index + 1;
    if contains_sequence(
        tokens
            .get(name_start..name_start.saturating_add(3))
            .unwrap_or_default(),
        &[Keyword::If, Keyword::Not, Keyword::Exists],
    ) {
        name_start += 3;
    } else if contains_sequence(
        tokens
            .get(name_start..name_start.saturating_add(2))
            .unwrap_or_default(),
        &[Keyword::If, Keyword::Exists],
    ) {
        name_start += 2;
    }
    let (name, after_name) = identifier_path(tokens, name_start)
        .ok_or_else(|| parse_error(0, "DDL object needs a name"))?;
    let relation = find_keyword(tokens, Keyword::On, after_name)
        .and_then(|index| identifier_path(tokens, index + 1).map(|(path, _)| path));
    if relation_required && relation.is_none() && marker != Keyword::Table {
        return Err(parse_error(0, "DDL object needs an owning table"));
    }
    Ok(SqliteNamedDdl {
        name,
        relation,
        definition: normalize_tokens(tokens),
    })
}

fn collect_relations(
    tokens: &[Token],
    relations: &mut Vec<Vec<String>>,
    joins: &mut Vec<Vec<String>>,
) {
    let mut cursor = 0;
    let mut is_join = false;
    while cursor < tokens.len() {
        if is_keyword(&tokens[cursor], Keyword::Join) {
            is_join = true;
            cursor += 1;
            continue;
        }
        if matches!(tokens[cursor], Token::Comma) {
            is_join = false;
            cursor += 1;
            continue;
        }
        if matches!(tokens[cursor], Token::Keyword(Keyword::On)) {
            break;
        }
        if let Some((path, next)) = identifier_path(tokens, cursor) {
            if is_join {
                joins.push(path);
            } else {
                relations.push(path);
            }
            cursor = next;
            while cursor < tokens.len()
                && !matches!(
                    tokens[cursor],
                    Token::Comma | Token::Keyword(Keyword::Join | Keyword::On)
                )
            {
                cursor += 1;
            }
        } else {
            cursor += 1;
        }
    }
}

fn lower_expression(tokens: &[Token]) -> SqliteExpression {
    if tokens == [Token::Operator("*".to_string())] {
        return SqliteExpression::Star {
            qualifier: Vec::new(),
        };
    }
    if tokens.len() == 1 {
        return match &tokens[0] {
            Token::Parameter(marker) => SqliteExpression::Parameter {
                marker: marker.clone(),
            },
            Token::String(value) | Token::Number(value) => SqliteExpression::Literal {
                value: value.clone(),
            },
            Token::Identifier(value) | Token::QuotedIdentifier(value) => SqliteExpression::Column {
                path: vec![value.clone()],
            },
            Token::Keyword(Keyword::Key | Keyword::Index) => SqliteExpression::Column {
                path: vec![tokens[0].normalized().to_ascii_lowercase()],
            },
            token => SqliteExpression::Raw {
                normalized: token.normalized(),
                columns: Vec::new(),
                parameters: match token {
                    Token::Parameter(marker) => vec![marker.clone()],
                    _ => Vec::new(),
                },
            },
        };
    }
    if let Some((path, consumed)) = identifier_path(tokens, 0)
        && consumed == tokens.len()
    {
        return SqliteExpression::Column { path };
    }
    SqliteExpression::Raw {
        normalized: normalize_tokens(tokens),
        columns: expression_columns(tokens),
        parameters: tokens
            .iter()
            .filter_map(|token| match token {
                Token::Parameter(marker) => Some(marker.clone()),
                _ => None,
            })
            .collect(),
    }
}

fn expression_columns(tokens: &[Token]) -> Vec<Vec<String>> {
    let mut columns = Vec::new();
    let mut cursor = 0;
    while cursor < tokens.len() {
        if let Some((path, next)) = identifier_path(tokens, cursor) {
            let function = tokens.get(next) == Some(&Token::LeftParen);
            if !function {
                columns.push(path);
            }
            cursor = next;
        } else {
            cursor += 1;
        }
    }
    columns
}

fn collect_write_expressions(tokens: &[Token]) -> Vec<SqliteExpression> {
    let start = find_keyword(tokens, Keyword::Where, 0)
        .or_else(|| find_keyword(tokens, Keyword::Values, 0));
    start
        .map(|index| vec![lower_expression(&tokens[index + 1..])])
        .unwrap_or_default()
}

fn collect_update_expressions(tokens: &[Token]) -> Vec<SqliteExpression> {
    let mut expressions = Vec::new();
    if let Some(start) = find_keyword(tokens, Keyword::Set, 0) {
        let end = [Keyword::Where, Keyword::Returning]
            .into_iter()
            .filter_map(|keyword| find_top_level_keyword(tokens, keyword, start + 1))
            .min()
            .unwrap_or(tokens.len());
        for assignment in split_top_level(&tokens[start + 1..end], &Token::Comma) {
            let value = assignment
                .iter()
                .position(|token| *token == Token::Operator("=".to_string()))
                .map(|index| &assignment[index + 1..])
                .unwrap_or_default();
            if !value.is_empty() {
                expressions.push(lower_expression(value));
            }
        }
    }
    if let Some(predicate) = clause_expression(tokens, Keyword::Where, &[Keyword::Returning]) {
        expressions.push(predicate);
    }
    expressions
}

fn assignment_columns(tokens: &[Token]) -> Vec<String> {
    let start = find_keyword(tokens, Keyword::Update, 0)
        .and_then(|_| find_keyword(tokens, Keyword::Set, 0))
        .or_else(|| {
            contains_sequence(
                tokens,
                &[
                    Keyword::On,
                    Keyword::Duplicate,
                    Keyword::Key,
                    Keyword::Update,
                ],
            )
            .then(|| find_keyword(tokens, Keyword::Update, 1))
            .flatten()
        });
    start.map_or_else(Vec::new, |start| {
        let end = [Keyword::Where, Keyword::Returning]
            .into_iter()
            .filter_map(|keyword| find_top_level_keyword(tokens, keyword, start + 1))
            .min()
            .unwrap_or(tokens.len());
        split_top_level(&tokens[start + 1..end], &Token::Comma)
            .into_iter()
            .filter_map(|assignment| assignment.first().and_then(Token::identifier))
            .map(str::to_string)
            .collect()
    })
}

fn clause_expression(
    tokens: &[Token],
    start: Keyword,
    ends: &[Keyword],
) -> Option<SqliteExpression> {
    let start = find_top_level_keyword(tokens, start, 0)?;
    let end = ends
        .iter()
        .filter_map(|keyword| find_top_level_keyword(tokens, *keyword, start + 1))
        .min()
        .unwrap_or(tokens.len());
    Some(lower_expression(&tokens[start + 1..end]))
}

fn clause_list(
    tokens: &[Token],
    start: Keyword,
    required_second: Option<Keyword>,
    ends: &[Keyword],
) -> Vec<SqliteExpression> {
    let Some(start) = find_top_level_keyword(tokens, start, 0) else {
        return Vec::new();
    };
    let mut content = start + 1;
    if let Some(required) = required_second {
        if !tokens
            .get(content)
            .is_some_and(|token| is_keyword(token, required))
        {
            return Vec::new();
        }
        content += 1;
    }
    let end = ends
        .iter()
        .filter_map(|keyword| find_top_level_keyword(tokens, *keyword, content))
        .min()
        .unwrap_or(tokens.len());
    split_top_level(&tokens[content..end], &Token::Comma)
        .into_iter()
        .map(lower_expression)
        .collect()
}

fn first_query_clause(tokens: &[Token], start: usize) -> usize {
    [
        Keyword::Where,
        Keyword::Group,
        Keyword::Having,
        Keyword::Order,
        Keyword::Limit,
        Keyword::For,
        Keyword::Union,
    ]
    .into_iter()
    .filter_map(|keyword| find_top_level_keyword(tokens, keyword, start))
    .min()
    .unwrap_or(tokens.len())
}

fn next_clause(tokens: &[Token], start: usize) -> Option<usize> {
    [
        Keyword::Where,
        Keyword::Group,
        Keyword::Having,
        Keyword::Order,
        Keyword::Limit,
        Keyword::For,
        Keyword::Union,
    ]
    .into_iter()
    .filter_map(|keyword| find_top_level_keyword(tokens, keyword, start))
    .min()
}

fn numeric_after(tokens: &[Token], keyword: Keyword) -> Option<u64> {
    let index = find_top_level_keyword(tokens, keyword, 0)?;
    match tokens.get(index + 1) {
        Some(Token::Number(value)) => value.parse().ok(),
        _ => None,
    }
}

fn limit_clause(tokens: &[Token]) -> (Option<u64>, Option<u64>) {
    let Some(index) = find_keyword(tokens, Keyword::Limit, 0) else {
        return (None, None);
    };
    let first = tokens.get(index + 1).and_then(|token| match token {
        Token::Number(value) => value.parse().ok(),
        _ => None,
    });
    if tokens.get(index + 2) == Some(&Token::Comma) {
        let count = tokens.get(index + 3).and_then(|token| match token {
            Token::Number(value) => value.parse().ok(),
            _ => None,
        });
        (count, first)
    } else {
        (first, None)
    }
}
