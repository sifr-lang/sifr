use super::*;
use crate::ast::{ColumnDefinition, CreateTableStatement, TableConstraint};

impl RawAdapter<'_> {
    pub(super) fn create_table(
        &self,
        body: &Map<String, Value>,
    ) -> Result<CreateTableStatement, PostgresParseError> {
        let mut columns = Vec::new();
        let mut constraints = Vec::new();
        for element in optional_array(body, "tableElts") {
            let (name, value) = tagged(object(element, "table element")?, "table element")?;
            if name == "ColumnDef" {
                columns.push(self.column_definition(value)?);
            } else if name == "Constraint" {
                constraints.push(self.table_constraint(value)?);
            } else {
                return Err(PostgresParseError::unsupported(
                    format!("unsupported CREATE TABLE element {name}"),
                    self.span(value),
                ));
            }
        }
        Ok(CreateTableStatement {
            name: relation_name(object_field(body, "relation")?),
            columns,
            constraints,
        })
    }

    pub(crate) fn column_definition(
        &self,
        body: &Map<String, Value>,
    ) -> Result<ColumnDefinition, PostgresParseError> {
        let mut nullable = true;
        let mut has_default = false;
        let mut default_sequence = None;
        let mut generated = false;
        let mut identity_generation = None;
        let mut primary_key = false;
        let mut unique = false;
        let mut references = None;
        let mut checks = Vec::new();
        for constraint in optional_array(body, "constraints") {
            let (_, constraint) = tagged(object(constraint, "column constraint")?, "constraint")?;
            match string_field(constraint, "contype").unwrap_or("") {
                "CONSTR_NOTNULL" => nullable = false,
                "CONSTR_DEFAULT" => {
                    has_default = true;
                    if let Some(raw) = optional_object_field(constraint, "raw_expr") {
                        let (tag, body) = tagged(raw, "default expression")?;
                        if tag == "FuncCall"
                            && name_list(body, "funcname")
                                .last()
                                .is_some_and(|name| name == "nextval")
                        {
                            let expression = self.expression_object(raw)?;
                            default_sequence = crate::sequence_default_reference(&expression)
                                .map_err(|message| self.invalid(message, constraint))?;
                        }
                    }
                }
                "CONSTR_GENERATED" => generated = true,
                "CONSTR_IDENTITY" => {
                    generated = true;
                    identity_generation = Some(
                        match string_field(constraint, "generated_when").unwrap_or("") {
                            "a" => "always",
                            "d" => "by-default",
                            _ => {
                                return Err(self.invalid(
                                    "identity column has an invalid generation mode",
                                    constraint,
                                ));
                            }
                        }
                        .to_string(),
                    );
                }
                "CONSTR_PRIMARY" => {
                    primary_key = true;
                    nullable = false;
                }
                "CONSTR_UNIQUE" => unique = true,
                "CONSTR_FOREIGN" => {
                    references = Some((
                        relation_name(object_field(constraint, "pktable")?),
                        name_list(constraint, "pk_attrs"),
                    ));
                }
                "CONSTR_CHECK" => {
                    checks.push(self.expression_object(object_field(constraint, "raw_expr")?)?);
                }
                _ => {}
            }
        }
        let ty = type_name(object_field(body, "typeName")?);
        if ty.path.last().is_some_and(|name| {
            matches!(
                name.as_str(),
                "serial" | "serial2" | "serial4" | "serial8" | "smallserial" | "bigserial"
            )
        }) {
            return Err(self.invalid(
                "SERIAL columns are unsupported; use an explicit sequence and nextval default or an identity column",
                body,
            ));
        }
        Ok(ColumnDefinition {
            name: string_field(body, "colname")
                .ok_or_else(|| self.invalid("column has no name", body))?
                .to_string(),
            ty,
            nullable,
            has_default,
            default_sequence,
            generated,
            identity_generation,
            primary_key,
            unique,
            references,
            checks,
            span: self.span(body),
        })
    }

    fn table_constraint(
        &self,
        body: &Map<String, Value>,
    ) -> Result<TableConstraint, PostgresParseError> {
        match string_field(body, "contype").unwrap_or("") {
            "CONSTR_PRIMARY" => Ok(TableConstraint::PrimaryKey {
                columns: name_list(body, "keys"),
            }),
            "CONSTR_UNIQUE" => Ok(TableConstraint::Unique {
                columns: name_list(body, "keys"),
            }),
            "CONSTR_FOREIGN" => Ok(TableConstraint::ForeignKey {
                columns: name_list(body, "fk_attrs"),
                relation: relation_name(object_field(body, "pktable")?),
                referenced: name_list(body, "pk_attrs"),
            }),
            "CONSTR_CHECK" => Ok(TableConstraint::Check {
                expression: self.expression_object(object_field(body, "raw_expr")?)?,
            }),
            _ => Err(self.invalid("unsupported table constraint", body)),
        }
    }
}
