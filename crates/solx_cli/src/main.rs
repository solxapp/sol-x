use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use solx_codegen::generate_anchor_code;
use solx_hir::Hir;
use solx_parser::parse;

#[derive(Parser)]
#[command(name = "solx")]
#[command(about = "SOL-X: A contract DSL that compiles to Anchor")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new SOL-X project
    New {
        /// Project name
        name: String,
    },
    /// Build the SOL-X project (generates Anchor code)
    Build {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Format SOL-X source files
    Fmt {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Run tests
    Test {
        /// Project directory (default: current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => cmd_new(&name)?,
        Commands::Build { path } => cmd_build(path.as_deref().unwrap_or(PathBuf::from(".").as_path()))?,
        Commands::Fmt { path } => cmd_fmt(path.as_deref().unwrap_or(PathBuf::from(".").as_path()))?,
        Commands::Test { path } => cmd_test(path.as_deref().unwrap_or(PathBuf::from(".").as_path()))?,
    }

    Ok(())
}

fn cmd_new(name: &str) -> Result<()> {
    let dir = PathBuf::from(name);
    if dir.exists() {
        anyhow::bail!("Directory {} already exists", name);
    }

    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("src"))?;

    // Create Anchor.toml (program ID must be valid Base58; same as declare_id! in generated lib.rs)
    const DEFAULT_PROGRAM_ID: &str = "11111111111111111111111111111111";
    let anchor_toml = format!(
        r#"[features]
resolution = true
skip-lint = false

[programs.localnet]
{} = "{}"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
"#,
        name, DEFAULT_PROGRAM_ID
    );
    fs::write(dir.join("Anchor.toml"), anchor_toml)?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
description = "Generated Anchor program from SOL-X"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "{}"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []

[dependencies]
anchor-lang = "0.30.0"

[profile.release]
overflow-checks = true
"#,
        name, name
    );
    fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    // Create default src/lib.rs (will be overwritten by build)
    let lib_rs = r#"use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");
"#;
    fs::write(dir.join("src").join("lib.rs"), lib_rs)?;

    // Create default program.solx
    let program_solx = format!(
        r#"program {}

account CounterState {{
  authority: Pubkey
  count: u64
}}

instruction initialize(authority: Signer, state: CounterState) {{
  init account state: CounterState payer authority
  state.authority = authority.key
  state.count = 0
}}

instruction increment(authority: Signer, state: CounterState) {{
  require state.authority == authority.key
  state.count += 1
}}
"#,
        name
    );
    fs::write(dir.join("src").join("program.solx"), program_solx)?;

    println!("Created new SOL-X project: {}", name);
    println!("  cd {}", name);
    println!("  solx build");

    Ok(())
}

fn cmd_build(path: &std::path::Path) -> Result<()> {
    let src_dir = path.join("src");
    let solx_in_src = src_dir.join("program.solx");
    let solx_in_root = path.join("program.solx");

    let solx_file = if solx_in_src.exists() {
        solx_in_src
    } else if solx_in_root.exists() {
        solx_in_root
    } else {
        anyhow::bail!(
            "No program.solx found in {} or {}",
            src_dir.display(),
            path.display()
        );
    };

    println!("Parsing SOL-X source...");
    let source = fs::read_to_string(&solx_file)
        .with_context(|| format!("Failed to read {}", solx_file.display()))?;

    let ast = parse(&source)?;
    println!("Type checking...");
    let hir = Hir::from_ast(ast)?;
    println!("Generating Anchor code...");
    let anchor_code = generate_anchor_code(&hir)?;

    // Write generated code (ensure src/ exists for examples with program.solx in root)
    fs::create_dir_all(&src_dir)?;
    let lib_rs_path = src_dir.join("lib.rs");
    let full_code = format!(
        "use anchor_lang::prelude::*;\n\ndeclare_id!(\"11111111111111111111111111111111\");\n\n{}",
        anchor_code
    );
    fs::write(&lib_rs_path, full_code)
        .with_context(|| format!("Failed to write {}", lib_rs_path.display()))?;

    println!("Generated Anchor code: {}", lib_rs_path.display());

    // Run anchor build only when this is an Anchor workspace (has Anchor.toml)
    let anchor_toml = path.join("Anchor.toml");
    if anchor_toml.exists() {
        println!("Running anchor build...");
        let status = Command::new("anchor")
            .arg("build")
            .current_dir(path)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Build successful!");
            }
            Ok(s) => {
                anyhow::bail!("Anchor build failed with exit code: {:?}", s.code());
            }
            Err(e) => {
                anyhow::bail!("Failed to run anchor build: {}. Make sure Anchor is installed.", e);
            }
        }
    } else {
        println!("Skipping anchor build (no Anchor.toml in {}).", path.display());
    }

    Ok(())
}

fn cmd_fmt(_path: &std::path::Path) -> Result<()> {
    // TODO: Implement formatter
    println!("Formatting not yet implemented. Coming soon!");
    Ok(())
}

fn cmd_test(path: &std::path::Path) -> Result<()> {
    println!("Running tests...");
    let status = Command::new("anchor")
        .arg("test")
        .current_dir(path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Tests passed!");
            Ok(())
        }
        Ok(s) => {
            anyhow::bail!("Tests failed with exit code: {:?}", s.code());
        }
        Err(e) => {
            anyhow::bail!("Failed to run anchor test: {}. Make sure Anchor is installed.", e);
        }
    }
}
