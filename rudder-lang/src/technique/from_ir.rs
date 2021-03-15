use super::*;
use crate::ir::{
    enums::EnumExpressionPart,
    ir2::IR2,
    resource::{ResourceDef, StateDef, Statement},
    variable::VariableDef,
};
use crate::parser::Token;
use std::collections::HashMap;
use std::convert::From;
use std::str;
use toml::{map::Map, value::Value as TOMLValue};

impl<'src> From<&IR2<'src>> for Technique {
    fn from(ir: &IR2<'src>) -> Self {
        let resources = ir
            .resources
            .iter()
            .filter_map(|(tk, res)| {
                if res.metadata.get("library") == Some(&TOMLValue::String("std".to_owned())) {
                    return None;
                } else {
                    return Some(res);
                }
            })
            .collect::<Vec<&ResourceDef>>();
        let resource = resources
            .get(0)
            .expect("There should only be one resource definition matching the technique");

        let meta = &resource.metadata;

        let interpolated_parameters: Vec<InterpolatedParameter> = meta
            .get("parameters")
            .expect(&format!(
                "Expected a 'parameter' field in '{}' metadatas",
                resource.name
            ))
            .as_array()
            .expect(&format!(
                "Metadata field 'parameters' of '{}' should be an array",
                resource.name
            ))
            .iter()
            .map(|p| {
                let param = p.as_table().expect(&format!(
                    "Metadata field 'parameter' of '{}' parameters metadata should be a table",
                    resource.name
                ));
                InterpolatedParameter {
                    id: extract_meta_string(param, "id"),
                    name: extract_meta_string(param, "name"),
                    description: extract_meta_string(param, "description"),
                }
            })
            .collect::<Vec<InterpolatedParameter>>();

        let method_calls: Vec<MethodCall> = resource
            .states
            .iter()
            .flat_map(|(_, state)| {
                state
                    .statements
                    .iter()
                    .flat_map(|stmt| {
                        statement_to_method_call(ir, resource, state, stmt, "any".to_owned())
                    })
                    .collect::<Vec<MethodCall>>()
            })
            .collect::<Vec<MethodCall>>();

        // TODO unit tests
        Technique {
            // can't determine a particular technique type yet
            r#type: "ncf_technique".to_owned(),
            version: 2,
            data: TechniqueData {
                bundle_name: resource.name.to_owned(),
                description: extract_meta_string(meta, "description"),
                name: extract_meta_string(meta, "name"),
                version: Version::from_str(&extract_meta_string(meta, "version")).unwrap(),
                category: extract_meta_string(meta, "category"),
                interpolated_parameters,
                method_calls,
                resources: Vec::new(),
            },
        }
    }
}

fn extract_meta_string(map: &Map<String, TOMLValue>, field: &str) -> String {
    map.get(field)
        .expect(&format!("Missing '{}' metadata", field))
        .as_str()
        .expect(&format!("Expected type string for '{}' metadata", field))
        .to_owned()
}

fn format_expr(ir: &IR2, expr: &EnumExpressionPart) -> String {
    match expr {
        EnumExpressionPart::And(e1, e2) => {
            let mut lexpr = format_expr(ir, &*e1);
            let mut rexpr = format_expr(ir, &*e2);
            if lexpr.contains('|') {
                lexpr = format!("({})", lexpr);
            }
            if rexpr.contains('|') {
                rexpr = format!("({})", rexpr);
            }
            format!("{}.{}", lexpr, rexpr)
        }
        EnumExpressionPart::Or(e1, e2) => {
            format!("{}|{}", format_expr(ir, &*e1), format_expr(ir, &*e2))
        }
        EnumExpressionPart::Not(e) => {
            let mut expr = format_expr(ir, &*e);
            if expr.contains('|') || expr.contains('.') {
                expr = format!("({})", expr);
            }
            format!("!{}", expr)
        }
        EnumExpressionPart::Compare(var, tree, item) => {
            if let Some(true) = ir.enum_list.enum_is_global(*var) {
                ir.enum_list.get_cfengine_item_name(*var, *item)
            } else {
                // if var is a foreign variable, output it as it is
                if tree.fragment() == "boolean" && item.fragment() == "true" {
                    var.fragment().to_owned()
                } else {
                    // concat var name + item
                    // TODO there may still be some conflicts with var or enum containing '_'
                    // format!("{}_{}", var.fragment(), item.fragment())
                    format!(
                        "{}_${{report_data.canonified_directive_id}}_{}",
                        var.fragment(),
                        item.fragment()
                    )
                }
            }
        }
        EnumExpressionPart::RangeCompare(var, tree, left, right) => unimplemented!(), // TODO
        EnumExpressionPart::Default(_) => "any".to_owned(),
        EnumExpressionPart::NoDefault(_) => "".to_owned(),
    }
}

fn value_to_string(
    value: &Value,
    variables: &HashMap<&Token, &VariableDef>,
    string_delim: bool,
) -> Result<String> {
    let delim = if string_delim { "\"" } else { "" };
    Ok(match value {
        Value::String(s) => format!("{}{}{}", delim, String::try_from(s)?, delim),
        Value::Float(_, n) => format!("{}", n),
        Value::Integer(_, n) => format!("{}", n),
        Value::Boolean(_, b) => format!("{}", b),
        Value::List(l) => format!(
            "[ {} ]",
            map_strings_results(l.iter(), |x| value_to_string(value, variables, true), ",")?
        ),
        Value::Struct(s) => unimplemented!(),
        Value::EnumExpression(_e) => unimplemented!(),
        Value::Variable(v) => {
            if let Some(var) = variables.get(v).and_then(|var_def| {
                var_def
                    .value
                    .first_value()
                    .and_then(|v| value_to_string(v, variables, string_delim))
                    .ok()
            }) {
                return Ok(var);
            }
            warn!(
                "The variable {} isn't recognized by rudderc, so we can't guarantee it will be defined when evaluated",
                v.fragment()
            );
            format!("{}${{{}}}{}", delim, v.fragment(), delim)
        } // resolve with context
    })
}

fn statement_to_method_call(
    ir: &IR2,
    res_def: &ResourceDef,
    state_def: &StateDef,
    stmt: &Statement,
    condition: String,
) -> Vec<MethodCall> {
    // get variables to try to get the proper parameter value
    let mut variables: HashMap<&Token, &VariableDef> = HashMap::new();
    for st_from_list in &state_def.statements {
        // variables declared after the current statemnt are not defined at this point
        if st_from_list == stmt {
            break;
        } else if let Statement::VariableDefinition(v) = st_from_list {
            variables.insert(&v.name, v);
        }
    }
    variables.extend(res_def.variable_definitions.get());
    variables.extend(&ir.variable_definitions);

    match stmt {
        Statement::ConditionVariableDefinition(s) => {
            let method_name = format!("{}_{}", *s.resource, *s.state);
            let parameters =
                fetch_method_parameters(ir, &s.to_method(), |name, value, _metadatas| {
                    Parameter::new(
                        name,
                        &value_to_string(value, &variables, false)
                            .expect("Value is not formatted correctly"),
                    )
                });
            vec![MethodCall {
                parameters,
                condition,
                method_name,
                component: s.metadata.get("component").and_then(|c| {
                    Some(
                        c.as_str()
                            .expect("Expected type string for 'component' metadata")
                            .to_owned(),
                    )
                }),
            }]
        }
        Statement::StateDeclaration(s) => {
            let method_alias = s
                .metadata
                .get("method_alias")
                .and_then(|v| v.as_str())
                .map(String::from);
            let method_name = if let Some(method_alias_content) = method_alias {
                method_alias_content
            } else {
                format!("{}_{}", *s.resource, *s.state)
            };
            let mut parameters = fetch_method_parameters(ir, s, |name, value, _| {
                Parameter::new(
                    name,
                    &value_to_string(value, &variables, false)
                        .expect("Value is not formatted correctly"),
                )
            });

            // EXCEPTION: reunite variable_string_escaped resource parameters that appear to be joined from cfengine side
            if method_name == "variable_string_escaped" {
                let merged_values = parameters
                    .iter()
                    .map(|p| p.value.clone())
                    .collect::<Vec<String>>()
                    .join(".");
                parameters = vec![Parameter::new("variable_name", &merged_values)];
            };

            vec![MethodCall {
                parameters,
                condition,
                method_name,
                component: s.metadata.get("component").and_then(|c| {
                    Some(
                        c.as_str()
                            .expect("Expected type string for 'component' metadata")
                            .to_owned(),
                    )
                }),
            }]
        }
        Statement::Case(_, enum_expressions) => enum_expressions
            .iter()
            .flat_map(|(enum_expr, stmts)| {
                stmts
                    .iter()
                    .flat_map(|stmt| {
                        statement_to_method_call(
                            ir,
                            res_def,
                            state_def,
                            stmt,
                            format_expr(ir, &enum_expr.expression),
                        )
                    })
                    .collect::<Vec<MethodCall>>()
            })
            .collect::<Vec<MethodCall>>(),
        _ => Vec::new(),
    }
}
