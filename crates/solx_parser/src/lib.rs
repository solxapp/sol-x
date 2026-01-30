use anyhow::Result;
use chumsky::prelude::*;
use solx_ast::*;

pub fn parse(source: &str) -> Result<Program> {
    program_parser()
        .parse(source)
        .map_err(|errs| {
            anyhow::anyhow!(
                "Parse errors:\n{}",
                errs.into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    let ident = text::ident().padded();

    let keyword = |s: &'static str| {
        just(s).padded()
    };

    let type_parser = recursive(|ty| {
        choice((
            just("Pubkey").to(Type::Pubkey),
            just("u8").to(Type::U8),
            just("u16").to(Type::U16),
            just("u32").to(Type::U32),
            just("u64").to(Type::U64),
            just("i8").to(Type::I8),
            just("i16").to(Type::I16),
            just("i32").to(Type::I32),
            just("i64").to(Type::I64),
            just("bool").to(Type::Bool),
            just("String").to(Type::String),
            ty.clone()
                .delimited_by(just("Vec<"), just(">"))
                .map(|t| Type::Vec(Box::new(t))),
            ty.clone()
                .delimited_by(just("Option<"), just(">"))
                .map(|t| Type::Option(Box::new(t))),
        ))
        .padded()
    });

    let field = ident
        .then_ignore(just(":").padded())
        .then(type_parser)
        .map(|(name, ty)| Field { name, ty })
        .padded();

    let account_def = keyword("account")
        .ignore_then(ident)
        .then(
            field.repeated()
                .delimited_by(just("{").padded(), just("}").padded()),
        )
        .map(|(name, fields)| AccountDef { name, fields })
        .padded();

    let param_type_parser = choice((
        just("Signer").to(ParamType::Signer),
        just("Pubkey").to(ParamType::Pubkey),
        just("u8").to(ParamType::U8),
        just("u16").to(ParamType::U16),
        just("u32").to(ParamType::U32),
        just("u64").to(ParamType::U64),
        just("i8").to(ParamType::I8),
        just("i16").to(ParamType::I16),
        just("i32").to(ParamType::I32),
        just("i64").to(ParamType::I64),
        just("bool").to(ParamType::Bool),
        just("String").to(ParamType::String),
        ident.map(|name| ParamType::Account(name)),
    ))
    .padded();

    let param = ident
        .then_ignore(just(":").padded())
        .then(param_type_parser)
        .map(|(name, ty)| Param { name, ty })
        .padded();

    let expr_parser = recursive(|_expr| {
        let literal = choice((
            text::int(10)
                .map(|s: String| {
                    if s.starts_with('-') {
                        Literal::Int(s.parse().unwrap_or(0))
                    } else {
                        Literal::UInt(s.parse().unwrap_or(0))
                    }
                }),
            just("true").to(Literal::Bool(true)),
            just("false").to(Literal::Bool(false)),
            just('"')
                .ignore_then(none_of('"').repeated().collect::<String>())
                .then_ignore(just('"'))
                .map(Literal::String),
        ))
        .map(Expr::Literal)
        .padded();

        let atom = choice((
            literal,
            ident.map(Expr::Ident),
        ));

        let field_access = atom
            .then(
                just(".")
                    .ignore_then(ident)
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .foldl(|obj, field| {
                Expr::FieldAccess {
                    object: Box::new(obj),
                    field,
                }
            });

        let unary = choice((
            just("!").to(UnOp::Not),
            just("-").to(UnOp::Neg),
        ))
        .then(field_access.clone())
        .map(|(op, expr)| Expr::UnaryOp {
            op,
            operand: Box::new(expr),
        })
        .or(field_access);

        let product = unary
            .clone()
            .then(
                choice((
                    just("*").to(BinOp::Mul),
                    just("/").to(BinOp::Div),
                    just("%").to(BinOp::Mod),
                ))
                .then(unary.clone())
                .repeated()
                .collect::<Vec<_>>(),
            )
            .foldl(|lhs, (op, rhs)| Expr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let sum = product
            .clone()
            .then(
                choice((
                    just("+").to(BinOp::Add),
                    just("-").to(BinOp::Sub),
                ))
                .then(product.clone())
                .repeated()
                .collect::<Vec<_>>(),
            )
            .foldl(|lhs, (op, rhs)| Expr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let comparison = sum
            .clone()
            .then(
                choice((
                    just("==").to(BinOp::Eq),
                    just("!=").to(BinOp::Ne),
                    just("<").to(BinOp::Lt),
                    just("<=").to(BinOp::Le),
                    just(">").to(BinOp::Gt),
                    just(">=").to(BinOp::Ge),
                ))
                .then(sum.clone())
                .repeated()
                .collect::<Vec<_>>(),
            )
            .foldl(|lhs, (op, rhs)| Expr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let logical_and = comparison
            .clone()
            .then(
                just("&&")
                    .ignore_then(comparison.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .foldl(|lhs, rhs| Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        let logical_or = logical_and
            .clone()
            .then(
                just("||")
                    .ignore_then(logical_and.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .foldl(|lhs, rhs| Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });

        logical_or
    });

    let statement_parser = recursive(|_stmt| {
        let init_account = keyword("init")
            .ignore_then(keyword("account"))
            .ignore_then(ident) // account variable name
            .then_ignore(just(":").padded())
            .then(ident) // account type name
            .then_ignore(keyword("payer").padded())
            .then(ident) // payer name
            .then(
                keyword("signer")
                    .ignore_then(ident)
                    .or_not(),
            )
            .map(|(((var_name, acc_type), payer), signer)| Statement::InitAccount {
                var_name,
                account_name: acc_type,
                payer,
                signer,
            })
            .padded();

        let require = keyword("require")
            .ignore_then(expr_parser.clone())
            .then(
                just(",")
                    .ignore_then(
                        just('"')
                            .ignore_then(none_of('"').repeated().collect::<String>())
                            .then_ignore(just('"')),
                    )
                    .or_not(),
            )
            .map(|(condition, message)| Statement::Require { condition, message })
            .padded();

        let assign_op = choice((
            just("+=").to(Some(BinOp::Add)),
            just("-=").to(Some(BinOp::Sub)),
            just("*=").to(Some(BinOp::Mul)),
            just("/=").to(Some(BinOp::Div)),
            just("%=").to(Some(BinOp::Mod)),
            just("=").to(None::<BinOp>),
        ))
        .padded();

        let assign = expr_parser
            .clone()
            .then(assign_op)
            .then(expr_parser.clone())
            .map(|((target, op_opt), value)| {
                if let Some(op) = op_opt {
                    // Compound assignment: x += y -> x = x + y
                    let bin_expr = Expr::BinaryOp {
                        op,
                        left: Box::new(target.clone()),
                        right: Box::new(value),
                    };
                    Statement::Assign {
                        target,
                        value: bin_expr,
                    }
                } else {
                    // Regular assignment
                    Statement::Assign { target, value }
                }
            })
            .padded();

        choice((
            init_account,
            require,
            assign,
            expr_parser.clone().map(Statement::Expr),
        ))
    });

    let instruction = keyword("instruction")
        .ignore_then(ident)
        .then(
            param.separated_by(just(",").padded())
                .delimited_by(just("(").padded(), just(")").padded()),
        )
        .then(
            statement_parser
                .repeated()
                .delimited_by(just("{").padded(), just("}").padded()),
        )
        .map(|((name, params), body)| Instruction { name, params, body })
        .padded();

    keyword("program")
        .ignore_then(ident)
        .then(account_def.repeated())
        .then(instruction.repeated())
        .map(|((name, accounts), instructions)| Program {
            name,
            accounts,
            instructions,
        })
        .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_counter() {
        let source = r#"
program Counter

account CounterState {
  authority: Pubkey
  count: u64
}

instruction initialize(authority: Signer) {
  init account state: CounterState payer authority
  state.authority = authority.key
  state.count = 0
}

instruction increment(authority: Signer) {
  require state.authority == authority.key
  state.count += 1
}
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }
}
