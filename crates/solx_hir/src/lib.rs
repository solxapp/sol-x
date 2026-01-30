use anyhow::Result;
use solx_ast::*;

/// High-level Intermediate Representation
/// This layer performs type checking and validation
pub struct Hir {
    pub program: Program,
}

impl Hir {
    pub fn from_ast(program: Program) -> Result<Self> {
        // Validate that all account types referenced exist
        for instruction in &program.instructions {
            for param in &instruction.params {
                if let ParamType::Account(ref name) = param.ty {
                    if !program.accounts.iter().any(|acc| acc.name == *name) {
                        anyhow::bail!("Unknown account type: {}", name);
                    }
                }
            }
        }

        Ok(Hir { program })
    }
}
