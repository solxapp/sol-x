use anyhow::Result;
use solx_ast::*;
use solx_hir::Hir;

pub fn generate_anchor_code(hir: &Hir) -> Result<String> {
    let program = &hir.program;
    let mut output = String::new();

    // Generate use statements
    output.push_str("use anchor_lang::prelude::*;\n\n");

    // Generate account structs
    for account in &program.accounts {
        output.push_str(&format!("#[account]\n"));
        output.push_str(&format!("pub struct {} {{\n", account.name));
        for field in &account.fields {
            output.push_str(&format!(
                "    pub {}: {},\n",
                field.name,
                field.ty.to_rust_type()
            ));
        }
        output.push_str("}\n\n");
    }

    // Generate program module
    output.push_str(&format!("#[program]\n"));
    output.push_str(&format!("pub mod {} {{\n", program.name.to_lowercase()));
    output.push_str("    use super::*;\n\n");

    // Generate instruction handlers
    for instruction in &program.instructions {
        output.push_str(&format!("    pub fn {}(\n", instruction.name));
        output.push_str("        ctx: Context<");
        output.push_str(&format!("{}", instruction.name));
        output.push_str("Context>,\n");

        // Generate parameters
        for param in &instruction.params {
            if matches!(param.ty, ParamType::Signer | ParamType::Account(_)) {
                continue; // These are in the context
            }
            output.push_str(&format!("        {}: {},\n", param.name, param.ty.to_rust_type()));
        }
        output.push_str("    ) -> Result<()> {\n");

        // Generate context struct name
        let context_name = format!("{}Context", instruction.name);

        // Generate body
        for stmt in &instruction.body {
            output.push_str(&generate_statement(stmt, &context_name));
        }

        output.push_str("        Ok(())\n");
        output.push_str("    }\n\n");
    }

    output.push_str("}\n\n");

    // Generate context structs
    for instruction in &program.instructions {
        output.push_str(&format!("#[derive(Accounts)]\n"));
        output.push_str(&format!("pub struct {}Context<'info> {{\n", instruction.name));

        // Find init account statements to determine which accounts need init
        let init_accounts: Vec<(&str, &str, Option<&str>)> = instruction
            .body
            .iter()
            .filter_map(|s| {
                if let Statement::InitAccount {
                    var_name,
                    account_name: _,
                    payer,
                    signer,
                } = s
                {
                    // Find the parameter that matches the variable name
                    instruction
                        .params
                        .iter()
                        .find(|p| p.name == *var_name)
                        .map(|p| (p.name.as_str(), payer.as_str(), signer.as_deref()))
                } else {
                    None
                }
            })
            .collect();

        // Generate accounts from parameters
        for param in &instruction.params {
            match &param.ty {
                ParamType::Signer => {
                    output.push_str(&format!(
                        "    #[account(mut)]\n    pub {}: Signer<'info>,\n",
                        param.name
                    ));
                }
                ParamType::Account(acc_name) => {
                    // Check if this account needs to be initialized
                    let init_info = init_accounts
                        .iter()
                        .find(|(param_name, _, _)| param_name == &param.name);

                    if let Some((_, payer, _signer)) = init_info {
                        // Calculate account size: 8 (discriminator) + sum of field sizes
                        let mut size = 8u64;
                        if let Some(account_def) = program.accounts.iter().find(|a| a.name == *acc_name) {
                            for field in &account_def.fields {
                                size += calculate_type_size(&field.ty);
                            }
                        }
                        output.push_str(&format!(
                            "    #[account(\n        init,\n        payer = {},\n        space = {}\n    )]\n",
                            payer, size
                        ));
                    } else {
                        output.push_str(&format!("    #[account(mut)]\n"));
                    }
                    output.push_str(&format!(
                        "    pub {}: Account<'info, {}>,\n",
                        param.name, acc_name
                    ));
                }
                _ => {}
            }
        }

        output.push_str("}\n\n");
    }

    Ok(output)
}

fn calculate_type_size(ty: &Type) -> u64 {
    match ty {
        Type::Pubkey => 32,
        Type::U8 | Type::I8 => 1,
        Type::U16 | Type::I16 => 2,
        Type::U32 | Type::I32 => 4,
        Type::U64 | Type::I64 => 8,
        Type::Bool => 1,
        Type::String => 4 + 4, // length prefix + data (variable, but we'll use a default)
        Type::Vec(inner) => 4 + 4 + calculate_type_size(inner), // length + capacity + element size
        Type::Option(inner) => 1 + calculate_type_size(inner), // discriminant + inner
    }
}

fn generate_statement(stmt: &Statement, context_name: &str) -> String {
    match stmt {
        Statement::InitAccount { .. } => {
            // Init is handled in the context struct via #[account(init)]
            // No code needed here
            String::new()
        }
        Statement::Require { condition, message } => {
            let cond_str = generate_expr(condition, context_name);
            if let Some(msg) = message {
                format!("        require!({}, {});\n", cond_str, msg)
            } else {
                format!("        require!({});\n", cond_str)
            }
        }
        Statement::Assign { target, value } => {
            let target_str = generate_expr(target, context_name);
            let value_str = generate_expr(value, context_name);
            format!("        {} = {};\n", target_str, value_str)
        }
        Statement::Expr(expr) => {
            format!("        {};\n", generate_expr(expr, context_name))
        }
    }
}

fn generate_expr(expr: &Expr, context_name: &str) -> String {
    match expr {
        Expr::Ident(name) => {
            // Check if it's a context field
            format!("ctx.accounts.{}", name)
        }
        Expr::FieldAccess { object, field } => {
            let obj_str = generate_expr(object, context_name);
            format!("{}.{}", obj_str, field)
        }
        Expr::Literal(lit) => match lit {
            Literal::Int(i) => i.to_string(),
            Literal::UInt(u) => u.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::String(s) => format!("\"{}\"", s),
        },
        Expr::BinaryOp { op, left, right } => {
            let left_str = generate_expr(left, context_name);
            let right_str = generate_expr(right, context_name);
            let op_str = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
                BinOp::And => "&&",
                BinOp::Or => "||",
            };
            format!("({} {} {})", left_str, op_str, right_str)
        }
        Expr::UnaryOp { op, operand } => {
            let op_str = match op {
                UnOp::Not => "!",
                UnOp::Neg => "-",
            };
            format!("{}{}", op_str, generate_expr(operand, context_name))
        }
    }
}
