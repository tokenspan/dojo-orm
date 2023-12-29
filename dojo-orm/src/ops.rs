use std::borrow::Cow;

use crate::order_by::Order;
use crate::types::ToSql;

#[derive(Debug)]
pub struct OpValue<'a> {
    pub column: Cow<'a, str>,
    pub op: &'a str,
    pub value: &'a (dyn ToSql + Sync),
}

#[derive(Debug)]
pub enum Op<'a> {
    Value(OpValue<'a>),
    And(Vec<Op<'a>>),
    Or(Vec<Op<'a>>),
}

impl<'a> Op<'a> {
    pub fn sql(&self, params_index: &mut usize) -> (String, Vec<&'a (dyn ToSql + Sync)>) {
        match self {
            Op::Value(op_value) => {
                let query = format!("{} {} ${}", op_value.column, op_value.op, params_index);
                let params = vec![op_value.value];
                *params_index += 1;
                (query, params)
            }
            Op::And(ops) => {
                let mut ands = vec![];
                let mut params = vec![];
                for op in ops {
                    let (q, p) = op.sql(params_index);
                    ands.push(q);
                    params.extend_from_slice(&p);
                }

                let query = format!("({})", ands.join(" AND "));
                (query, params)
            }
            Op::Or(ops) => {
                let mut ors = vec![];
                let mut params = vec![];
                for op in ops {
                    let (q, p) = op.sql(params_index);
                    ors.push(q);
                    params.extend_from_slice(&p);
                }

                let query = format!("({})", ors.join(" OR "));
                (query, params)
            }
        }
    }
}

pub fn and(ops: Vec<Op>) -> Op {
    Op::And(ops)
}

pub fn or(ops: Vec<Op>) -> Op {
    Op::Or(ops)
}

pub fn eq<'a, T: ToSql + Sync>(column: &'a str, value: &'a T) -> Op<'a> {
    Op::Value(OpValue {
        column: column.into(),
        op: "=",
        value,
    })
}

pub fn in_list<'a, T: ToSql + Sync>(column: &'a str, values: &'a Vec<T>) -> Op<'a> {
    Op::Value(OpValue {
        column: column.into(),
        op: "IN",
        value: values,
    })
}

pub fn asc(column: &str) -> (&str, Order) {
    (column, Order::Asc)
}

pub fn desc(column: &str) -> (&str, Order) {
    (column, Order::Desc)
}
