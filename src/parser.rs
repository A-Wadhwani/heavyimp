use pest::{pratt_parser::PrattParser, Parser};
use pest_derive::Parser;

use lazy_static::lazy_static;

use crate::{
    error::ImpParseError,
    syntax::{Constant, Expr, Statement},
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct ImpParser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};

        PrattParser::new()
            .op(Op::infix(Rule::less, Left))
            .op(Op::infix(Rule::add, Left))
    };
}

pub fn parse(source: &str) -> Result<Statement, ImpParseError> {
    let pairs = ImpParser::parse(Rule::program, source).map_err(|_| ImpParseError::Other)?;
    let statements = pairs.filter_map(|pair| match pair.as_rule() {
        Rule::EOI => None,
        _ => Some(build_stmnt(pair)),
    });

    Ok(statements.fold(Statement::Skip, |acc, next| {
        Statement::Sequence(Box::new(acc), Box::new(next))
    }))
}

pub fn build_stmnt(pair: pest::iterators::Pair<Rule>) -> Statement {
    match pair.as_rule() {
        Rule::store_assign => {
            let mut pairs = pair.into_inner();
            let ident = pairs.next().unwrap().as_str().to_owned();
            let rhs_pair = pairs.next().unwrap();
            let rhs = build_expr(rhs_pair);
            Statement::StoreAssign(ident, rhs)
        }
        Rule::heap_new => {
            let mut pairs = pair.into_inner();
            let ident = pairs.next().unwrap().as_str().to_owned();
            let rhs_pair = pairs.next().unwrap();
            let rhs = build_expr(rhs_pair);
            Statement::HeapNew(ident, rhs)
        }
        Rule::heap_update => {
            let mut pairs = pair.into_inner();
            let ident = pairs.next().unwrap().as_str().to_owned();
            let rhs_pair = pairs.next().unwrap();
            let rhs = build_expr(rhs_pair);
            Statement::HeapUpdate(ident, rhs)
        }
        Rule::heap_alias => {
            let mut pairs = pair.into_inner();
            let ident = pairs.next().unwrap().as_str().to_owned();
            let rhs_ident = pairs.next().unwrap().as_str().to_owned();
            Statement::HeapAlias(ident, rhs_ident)
        }
        Rule::conditional => {
            let mut pairs = pair.into_inner();
            let cond_pair = pairs.next().unwrap();
            let cond_expr = build_expr(cond_pair);
            let then_stmnt_pair = pairs.next().unwrap();
            let then_stmnt = build_stmnt(then_stmnt_pair);
            let else_stmnt_pair = pairs.next().unwrap();
            let else_stmnt = build_stmnt(else_stmnt_pair);
            Statement::Conditional(cond_expr, Box::new(then_stmnt), Box::new(else_stmnt))
        }
        Rule::while_loop => {
            let mut pairs = pair.into_inner();
            let cond_pair = pairs.next().unwrap();
            let cond_expr = build_expr(cond_pair);
            let body_stmnt_pair = pairs.next().unwrap();
            let body_stmnt = build_stmnt(body_stmnt_pair);
            Statement::While(cond_expr, Box::new(body_stmnt))
        }
        Rule::skip => Statement::Skip,
        _ => panic!("{:?}", pair.as_rule()),
    }
}

pub fn build_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => build_expr(pair.into_inner().next().unwrap()),
        Rule::boolean => match pair.as_str() {
            "true" => Expr::Constant(Constant::Bool(true)),
            "false" => Expr::Constant(Constant::Bool(false)),
            _ => unreachable!(),
        },
        Rule::number => {
            let n = pair.as_str().parse::<i64>().unwrap();
            Expr::Constant(Constant::Nat(n))
        }
        Rule::ident => Expr::StoreRead(pair.as_str().to_string()),
        Rule::deref_ident => Expr::HeapRead(pair.as_str().strip_prefix('*').unwrap().to_string()),
        Rule::binary_expr | Rule::unary_expr => PRATT_PARSER
            .map_primary(|primary| build_expr(primary))
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::not => Expr::BoolNot(Box::new(rhs)),
                _ => unreachable!(),
            })
            .map_infix(|lhs, op, rhs| match op.as_rule() {
                Rule::add => Expr::NatAdd(Box::new(lhs), Box::new(rhs)),
                Rule::less => Expr::NatLeq(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            })
            .parse(pair.into_inner()),
        _ => panic!("{:?}", pair.as_rule()),
    }
}
