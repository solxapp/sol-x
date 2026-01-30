use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub accounts: Vec<AccountDef>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Pubkey,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Bool,
    String,
    Vec(Box<Type>),
    Option(Box<Type>),
}

impl Type {
    pub fn to_rust_type(&self) -> String {
        match self {
            Type::Pubkey => "Pubkey".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Vec(inner) => format!("Vec<{}>", inner.to_rust_type()),
            Type::Option(inner) => format!("Option<{}>", inner.to_rust_type()),
        }
    }

    pub fn to_anchor_type(&self) -> String {
        match self {
            Type::Pubkey => "pubkey".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Vec(inner) => format!("vec<{}>", inner.to_anchor_type()),
            Type::Option(inner) => format!("option<{}>", inner.to_anchor_type()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: ParamType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamType {
    Signer,
    Account(String), // Account type name
    Pubkey,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Bool,
    String,
}

impl ParamType {
    pub fn to_rust_type(&self) -> String {
        match self {
            ParamType::Signer => "Signer<'_>".to_string(),
            ParamType::Account(name) => format!("Account<'_, {}>", name),
            ParamType::Pubkey => "Pubkey".to_string(),
            ParamType::U8 => "u8".to_string(),
            ParamType::U16 => "u16".to_string(),
            ParamType::U32 => "u32".to_string(),
            ParamType::U64 => "u64".to_string(),
            ParamType::I8 => "i8".to_string(),
            ParamType::I16 => "i16".to_string(),
            ParamType::I32 => "i32".to_string(),
            ParamType::I64 => "i64".to_string(),
            ParamType::Bool => "bool".to_string(),
            ParamType::String => "String".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    InitAccount {
        var_name: String,      // Variable name (e.g., "state")
        account_name: String,   // Account type name (e.g., "CounterState")
        payer: String,
        signer: Option<String>,
    },
    Require {
        condition: Expr,
        message: Option<String>,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Ident(String),
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Literal(Literal),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    UInt(u64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnOp {
    Not,
    Neg,
}
