use crate::ast::{
    MysqlColumnDefinition, MysqlConflictForm, MysqlCreateTable, MysqlExpression,
    MysqlForeignKeyDefinition, MysqlGeneratedColumn, MysqlKeyDefinition, MysqlNamedDdl,
    MysqlProjection, MysqlQuery, MysqlStatement, MysqlStatementKind, MysqlTypeName, MysqlWrite,
};
use crate::lexer::{Keyword, Token};
use crate::parser::{
    MysqlParseError, MysqlParser, RawStatement, is_keyword, normalize_tokens, parse_error,
};

pub(crate) fn lower_statement(
    raw: &RawStatement,
    parser: &MysqlParser,
) -> Result<MysqlStatement, MysqlParseError> {
    if raw
        .tokens
        .iter()
        .any(|token| is_keyword(token, Keyword::Returning))
    {
        return Err(parse_error(
            raw.span.start as usize,
            "RETURNING is not part of the supported MySQL grammar",
        ));
    }
    validate_parentheses(&raw.tokens, raw.span.start as usize)?;
    let kind = match raw.tokens.first() {
        Some(Token::Keyword(Keyword::Select | Keyword::With)) => {
            MysqlStatementKind::Query(lower_query(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Insert | Keyword::Replace)) => {
            MysqlStatementKind::Insert(lower_insert(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Update)) => {
            MysqlStatementKind::Update(lower_update(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Delete)) => {
            MysqlStatementKind::Delete(lower_delete(&raw.tokens)?)
        }
        Some(Token::Keyword(Keyword::Create)) => lower_create(&raw.tokens, parser)?,
        Some(Token::Keyword(Keyword::Alter)) => {
            MysqlStatementKind::AlterTable(lower_named_ddl(&raw.tokens, Keyword::Table, true)?)
        }
        Some(Token::Keyword(Keyword::Drop)) => {
            MysqlStatementKind::Drop(lower_named_ddl(&raw.tokens, Keyword::Table, false)?)
        }
        _ => {
            return Err(parse_error(
                raw.span.start as usize,
                "unsupported MySQL statement",
            ));
        }
    };
    Ok(MysqlStatement {
        kind,
        span: raw.span,
    })
}

fn lower_query(tokens: &[Token]) -> Result<MysqlQuery, MysqlParseError> {
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
    let limit = numeric_after(tokens, Keyword::Limit);
    let offset = numeric_after(tokens, Keyword::Offset);
    let common_tables = if select > 0 {
        tokens[1..select]
            .iter()
            .filter_map(Token::identifier)
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };
    Ok(MysqlQuery {
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

fn lower_projection(tokens: &[Token]) -> Result<MysqlProjection, MysqlParseError> {
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
    Ok(MysqlProjection {
        expression: lower_expression(expression),
        alias,
    })
}

fn lower_insert(tokens: &[Token]) -> Result<MysqlWrite, MysqlParseError> {
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
                Token::Parameter | Token::String(_) | Token::Number(_)
            )
        })
        .map(|token| lower_expression(std::slice::from_ref(token)))
        .collect();
    let conflict = if is_keyword(&tokens[0], Keyword::Replace) {
        MysqlConflictForm::Replace
    } else if tokens
        .iter()
        .any(|token| is_keyword(token, Keyword::Ignore))
    {
        MysqlConflictForm::Ignore
    } else if contains_sequence(
        tokens,
        &[
            Keyword::On,
            Keyword::Duplicate,
            Keyword::Key,
            Keyword::Update,
        ],
    ) {
        MysqlConflictForm::OnDuplicateKeyUpdate
    } else {
        MysqlConflictForm::None
    };
    let assignments = assignment_columns(tokens);
    Ok(MysqlWrite {
        relation,
        columns,
        assignments,
        expressions,
        conflict,
    })
}

fn lower_update(tokens: &[Token]) -> Result<MysqlWrite, MysqlParseError> {
    let (relation, _) =
        identifier_path(tokens, 1).ok_or_else(|| parse_error(0, "UPDATE needs a target table"))?;
    let assignments = assignment_columns(tokens);
    let expressions = collect_write_expressions(tokens);
    Ok(MysqlWrite {
        relation,
        columns: assignments.clone(),
        assignments,
        expressions,
        conflict: MysqlConflictForm::None,
    })
}

fn lower_delete(tokens: &[Token]) -> Result<MysqlWrite, MysqlParseError> {
    let from = find_keyword(tokens, Keyword::From, 0)
        .ok_or_else(|| parse_error(0, "DELETE needs FROM"))?;
    let (relation, _) = identifier_path(tokens, from + 1)
        .ok_or_else(|| parse_error(0, "DELETE needs a target table"))?;
    Ok(MysqlWrite {
        relation,
        columns: Vec::new(),
        assignments: Vec::new(),
        expressions: collect_write_expressions(tokens),
        conflict: MysqlConflictForm::None,
    })
}

fn lower_create(
    tokens: &[Token],
    parser: &MysqlParser,
) -> Result<MysqlStatementKind, MysqlParseError> {
    match tokens.get(1) {
        Some(Token::Keyword(Keyword::Table)) => {
            lower_create_table(tokens, parser).map(MysqlStatementKind::CreateTable)
        }
        Some(Token::Keyword(Keyword::View)) => {
            lower_named_ddl(tokens, Keyword::View, false).map(MysqlStatementKind::CreateView)
        }
        Some(Token::Keyword(Keyword::Index | Keyword::Unique)) => {
            lower_named_ddl(tokens, Keyword::Index, true).map(MysqlStatementKind::CreateIndex)
        }
        _ => Err(parse_error(0, "unsupported MySQL CREATE form")),
    }
}

fn lower_create_table(
    tokens: &[Token],
    parser: &MysqlParser,
) -> Result<MysqlCreateTable, MysqlParseError> {
    let (name, open) = identifier_path(tokens, 2)
        .ok_or_else(|| parse_error(0, "CREATE TABLE needs a table name"))?;
    if tokens.get(open) != Some(&Token::LeftParen) {
        return Err(parse_error(0, "CREATE TABLE needs a definition list"));
    }
    let close = matching_right(tokens, open)
        .ok_or_else(|| parse_error(0, "CREATE TABLE definition is not closed"))?;
    let mut table = MysqlCreateTable {
        name,
        columns: Vec::new(),
        primary_key: Vec::new(),
        unique_keys: Vec::new(),
        foreign_keys: Vec::new(),
        checks: Vec::new(),
        indexes: Vec::new(),
        default_character_set: None,
        default_collation: None,
    };
    for definition in split_top_level(&tokens[open + 1..close], &Token::Comma) {
        lower_table_definition(definition, parser, &mut table)?;
    }
    if table.columns.is_empty() {
        return Err(parse_error(0, "CREATE TABLE needs at least one column"));
    }
    let options = &tokens[close + 1..];
    table.default_character_set = option_value(options, Keyword::Charset)
        .or_else(|| option_value_after_sequence(options, Keyword::Character, Keyword::Set));
    table.default_collation = option_value(options, Keyword::Collate)
        .or_else(|| Some(parser.default_collation().to_string()));
    Ok(table)
}

fn lower_table_definition(
    tokens: &[Token],
    _parser: &MysqlParser,
    table: &mut MysqlCreateTable,
) -> Result<(), MysqlParseError> {
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
            table.unique_keys.push(MysqlKeyDefinition {
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
            table.foreign_keys.push(MysqlForeignKeyDefinition {
                name: constraint_name,
                columns,
                referenced_table,
                referenced_columns,
            });
        }
        Some(Token::Keyword(Keyword::Check)) => {
            table.checks.push(normalize_tokens(&tokens[cursor + 1..]));
        }
        Some(Token::Keyword(Keyword::Key | Keyword::Index)) => {
            table.indexes.push(MysqlKeyDefinition {
                name: optional_key_name(tokens, cursor + 1),
                columns: key_columns(tokens, cursor + 1)?,
            });
        }
        Some(Token::Identifier(_) | Token::QuotedIdentifier(_)) => {
            table.columns.push(lower_column(tokens)?);
        }
        _ => return Err(parse_error(0, "unsupported CREATE TABLE definition")),
    }
    Ok(())
}

fn lower_column(tokens: &[Token]) -> Result<MysqlColumnDefinition, MysqlParseError> {
    let name = tokens[0]
        .identifier()
        .ok_or_else(|| parse_error(0, "column needs a name"))?
        .to_string();
    let type_name =
        token_word(tokens.get(1)).ok_or_else(|| parse_error(0, "column needs a type"))?;
    let mut cursor = 2;
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
        MysqlGeneratedColumn {
            expression,
            stored: tail.iter().any(|token| is_keyword(token, Keyword::Stored)),
        }
    });
    Ok(MysqlColumnDefinition {
        name,
        ty: MysqlTypeName {
            name: type_name,
            parameters,
            unsigned: tail
                .iter()
                .any(|token| is_keyword(token, Keyword::Unsigned)),
            zerofill: tail
                .iter()
                .any(|token| is_keyword(token, Keyword::Zerofill)),
        },
        nullable: !contains_sequence(tail, &[Keyword::Not, Keyword::Null]),
        auto_increment: tail
            .iter()
            .any(|token| is_keyword(token, Keyword::AutoIncrement)),
        generated,
        default: value_after(tail, Keyword::Default),
        character_set: option_value_after_sequence(tail, Keyword::Character, Keyword::Set),
        collation: option_value(tail, Keyword::Collate),
    })
}

fn lower_named_ddl(
    tokens: &[Token],
    marker: Keyword,
    relation_required: bool,
) -> Result<MysqlNamedDdl, MysqlParseError> {
    let marker_index = find_keyword(tokens, marker, 0)
        .or_else(|| {
            (marker == Keyword::Index)
                .then(|| find_keyword(tokens, Keyword::Unique, 0))
                .flatten()
        })
        .ok_or_else(|| parse_error(0, "DDL object kind is missing"))?;
    let (name, after_name) = identifier_path(tokens, marker_index + 1)
        .ok_or_else(|| parse_error(0, "DDL object needs a name"))?;
    let relation = find_keyword(tokens, Keyword::On, after_name)
        .and_then(|index| identifier_path(tokens, index + 1).map(|(path, _)| path));
    if relation_required && relation.is_none() && marker != Keyword::Table {
        return Err(parse_error(0, "DDL object needs an owning table"));
    }
    Ok(MysqlNamedDdl {
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

fn lower_expression(tokens: &[Token]) -> MysqlExpression {
    if tokens == [Token::Operator("*".to_string())] {
        return MysqlExpression::Star {
            qualifier: Vec::new(),
        };
    }
    if tokens.len() == 1 {
        return match &tokens[0] {
            Token::Parameter => MysqlExpression::Parameter,
            Token::String(value) | Token::Number(value) => MysqlExpression::Literal {
                value: value.clone(),
            },
            Token::Identifier(value) | Token::QuotedIdentifier(value) => MysqlExpression::Column {
                path: vec![value.clone()],
            },
            token => MysqlExpression::Raw {
                normalized: token.normalized(),
                columns: Vec::new(),
                parameters: u32::from(matches!(token, Token::Parameter)),
            },
        };
    }
    if let Some((path, consumed)) = identifier_path(tokens, 0)
        && consumed == tokens.len()
    {
        return MysqlExpression::Column { path };
    }
    MysqlExpression::Raw {
        normalized: normalize_tokens(tokens),
        columns: expression_columns(tokens),
        parameters: u32::try_from(
            tokens
                .iter()
                .filter(|token| matches!(token, Token::Parameter))
                .count(),
        )
        .unwrap_or(u32::MAX),
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

fn collect_write_expressions(tokens: &[Token]) -> Vec<MysqlExpression> {
    let start = find_keyword(tokens, Keyword::Where, 0)
        .or_else(|| find_keyword(tokens, Keyword::Values, 0));
    start
        .map(|index| vec![lower_expression(&tokens[index + 1..])])
        .unwrap_or_default()
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
        split_top_level(&tokens[start + 1..], &Token::Comma)
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
) -> Option<MysqlExpression> {
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
) -> Vec<MysqlExpression> {
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

fn find_keyword(tokens: &[Token], keyword: Keyword, start: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| is_keyword(token, keyword).then_some(index))
}

fn find_top_level_keyword(tokens: &[Token], keyword: Keyword, start: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => depth = depth.saturating_sub(1),
            _ if depth == 0 && is_keyword(token, keyword) => return Some(index),
            _ => {}
        }
    }
    None
}

fn split_top_level<'tokens>(tokens: &'tokens [Token], separator: &Token) -> Vec<&'tokens [Token]> {
    let mut parts = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => depth = depth.saturating_sub(1),
            _ if depth == 0 && token == separator => {
                parts.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < tokens.len() {
        parts.push(&tokens[start..]);
    }
    parts
}

fn identifier_path(tokens: &[Token], start: usize) -> Option<(Vec<String>, usize)> {
    let mut path = vec![tokens.get(start)?.identifier()?.to_string()];
    let mut cursor = start + 1;
    while tokens.get(cursor) == Some(&Token::Dot) {
        path.push(tokens.get(cursor + 1)?.identifier()?.to_string());
        cursor += 2;
    }
    Some((path, cursor))
}

fn matching_right(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn validate_parentheses(tokens: &[Token], offset: usize) -> Result<(), MysqlParseError> {
    let mut depth = 0_i64;
    for token in tokens {
        match token {
            Token::LeftParen => depth += 1,
            Token::RightParen => {
                depth -= 1;
                if depth < 0 {
                    return Err(parse_error(offset, "unexpected closing parenthesis"));
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(parse_error(offset, "unclosed parenthesis"));
    }
    Ok(())
}

fn key_columns(tokens: &[Token], start: usize) -> Result<Vec<String>, MysqlParseError> {
    let open = tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| (*token == Token::LeftParen).then_some(index))
        .ok_or_else(|| parse_error(0, "key needs a column list"))?;
    let close = matching_right(tokens, open)
        .ok_or_else(|| parse_error(0, "key column list is not closed"))?;
    split_top_level(&tokens[open + 1..close], &Token::Comma)
        .into_iter()
        .map(single_identifier)
        .collect()
}

fn single_identifier(tokens: &[Token]) -> Result<String, MysqlParseError> {
    tokens
        .first()
        .and_then(Token::identifier)
        .map(str::to_string)
        .ok_or_else(|| parse_error(0, "expected an identifier"))
}

fn optional_key_name(tokens: &[Token], start: usize) -> Option<String> {
    tokens
        .iter()
        .skip(start)
        .take_while(|token| **token != Token::LeftParen)
        .find_map(Token::identifier)
        .map(str::to_string)
}

fn option_value(tokens: &[Token], keyword: Keyword) -> Option<String> {
    let index = find_keyword(tokens, keyword, 0)?;
    tokens
        .get(index + 1)
        .filter(|token| **token != Token::Operator("=".to_string()))
        .or_else(|| tokens.get(index + 2))
        .and_then(|token| token_word(Some(token)))
}

fn option_value_after_sequence(
    tokens: &[Token],
    first: Keyword,
    second: Keyword,
) -> Option<String> {
    let index = tokens
        .windows(2)
        .position(|pair| is_keyword(&pair[0], first) && is_keyword(&pair[1], second))?;
    token_word(tokens.get(index + 2))
}

fn value_after(tokens: &[Token], keyword: Keyword) -> Option<String> {
    let index = find_keyword(tokens, keyword, 0)?;
    tokens.get(index + 1).map(Token::normalized)
}

fn numeric_after(tokens: &[Token], keyword: Keyword) -> Option<u64> {
    let index = find_top_level_keyword(tokens, keyword, 0)?;
    match tokens.get(index + 1) {
        Some(Token::Number(value)) => value.parse().ok(),
        _ => None,
    }
}

fn token_word(token: Option<&Token>) -> Option<String> {
    match token? {
        Token::Identifier(value) | Token::QuotedIdentifier(value) => Some(value.clone()),
        Token::Keyword(keyword) => Some(keyword.text().to_ascii_lowercase()),
        Token::String(value) | Token::Number(value) => Some(value.clone()),
        _ => None,
    }
}

fn contains_sequence(tokens: &[Token], keywords: &[Keyword]) -> bool {
    tokens.windows(keywords.len()).any(|window| {
        window
            .iter()
            .zip(keywords)
            .all(|(token, keyword)| is_keyword(token, *keyword))
    })
}
