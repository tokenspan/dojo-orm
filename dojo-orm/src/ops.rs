use std::borrow::Cow;

use crate::order_by::Order;
use crate::types::ToSql;

#[derive(Debug, Copy, Clone)]
pub enum OpValueType {
    Value,
    Array,
}

#[derive(Debug, Clone)]
pub struct OpValue<'a> {
    pub ty: OpValueType,
    pub column: Cow<'a, str>,
    pub op: &'a str,
    pub value: &'a (dyn ToSql + Sync),
}

#[derive(Debug, Clone)]
pub enum Op<'a> {
    Value(OpValue<'a>),
    And(&'a [Op<'a>]),
    Or(&'a [Op<'a>]),
}

impl<'a> Op<'a> {
    pub fn sql(&self, params_index: &mut usize) -> (Option<String>, Vec<&'a (dyn ToSql + Sync)>) {
        match self {
            Op::Value(op_value) => {
                let query = match op_value.ty {
                    OpValueType::Value => {
                        format!("{} {} ${}", op_value.column, op_value.op, params_index)
                    }
                    OpValueType::Array => {
                        format!("{} {} ANY(${})", op_value.column, op_value.op, params_index)
                    }
                };
                let params = vec![op_value.value];
                *params_index += 1;
                (Some(query), params)
            }
            Op::And(ops) => {
                if ops.is_empty() {
                    return (None, vec![]);
                }

                let mut ands = vec![];
                let mut params = vec![];
                for op in *ops {
                    let (q, p) = op.sql(params_index);
                    if let Some(q) = q {
                        ands.push(q);
                        params.extend_from_slice(&p);
                    }
                }

                let query = format!("({})", ands.join(" AND "));
                (Some(query), params)
            }
            Op::Or(ops) => {
                if ops.is_empty() {
                    return (None, vec![]);
                }

                let mut ors = vec![];
                let mut params = vec![];
                for op in *ops {
                    let (q, p) = op.sql(params_index);
                    if let Some(q) = q {
                        ors.push(q);
                        params.extend_from_slice(&p);
                    }
                }

                let query = format!("({})", ors.join(" OR "));
                (Some(query), params)
            }
        }
    }
}

pub fn and<'a>(ops: &'a [Op<'a>]) -> Op<'a> {
    Op::And(ops)
}

pub fn or<'a>(ops: &'a [Op<'a>]) -> Op<'a> {
    Op::Or(ops)
}

pub fn eq<'a, T: ToSql + Sync>(column: &'a str, value: &'a T) -> Op<'a> {
    Op::Value(OpValue {
        ty: OpValueType::Value,
        column: column.into(),
        op: "=",
        value,
    })
}

pub fn in_list<'a, T: ToSql + Sync>(column: &'a str, values: &'a T) -> Op<'a> {
    Op::Value(OpValue {
        ty: OpValueType::Array,
        column: column.into(),
        op: "=",
        value: values,
    })
}

pub fn asc(column: &str) -> (&str, Order) {
    (column, Order::Asc)
}

pub fn desc(column: &str) -> (&str, Order) {
    (column, Order::Desc)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_and() {
        use super::{and, eq, in_list, Op};
        use crate::types::ToSql;

        let op = and(&[eq("foo", &1), eq("bar", &2), in_list("baz", &vec![3, 4, 5])]);

        match op {
            Op::And(ops) => {
                assert_eq!(ops.len(), 3);
                match &ops[0] {
                    Op::Value(op_value) => {
                        assert_eq!(op_value.column, "foo");
                        assert_eq!(op_value.op, "=");
                        assert_eq!(
                            op_value
                                .value
                                .to_sql(&crate::types::Type::INT4, &mut vec![])
                                .unwrap(),
                            1
                        );
                    }
                    _ => panic!("Expected Op::Value"),
                }
                match &ops[1] {
                    Op::Value(op_value) => {
                        assert_eq!(op_value.column, "bar");
                        assert_eq!(op_value.op, "=");
                        assert_eq!(
                            op_value
                                .value
                                .to_sql(&crate::types::Type::INT4, &mut vec![])
                                .unwrap(),
                            2
                        );
                    }
                    _ => panic!("Expected Op::Value"),
                }
                match &ops[2] {
                    Op::Value(op_value) => {
                        assert_eq!(op_value.column, "baz");
                        assert_eq!(op_value.op, "IN");
                        assert_eq!(
                            op_value
                                .value
                                .to_sql(&crate::types::Type::INT4, &mut vec![])
                                .unwrap(),
                            vec![3, 4, 5]
                        );
                    }
                    _ => panic!("Expected Op::Value"),
                }
            }
            _ => panic!("Expected Op::And"),
        }
    }
}
