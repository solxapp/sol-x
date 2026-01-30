# SOL-X Architecture

## Overview

SOL-X uses a multi-stage compilation pipeline to transform SOL-X source code into Anchor Rust programs.

## Compilation Pipeline

```
SOL-X Source (.solx)
    ↓
[Parser] → AST
    ↓
[HIR] → Typed IR
    ↓
[Codegen] → Anchor Rust
    ↓
Anchor Build → Program
```

## Components

### 1. Parser (`solx_parser`)

**Purpose:** Parse SOL-X source code into an Abstract Syntax Tree (AST).

**Technology:** Chumsky parser combinator library

**Input:** SOL-X source code (`.solx` files)

**Output:** `solx_ast::Program` AST

**Key Features:**
- Recursive descent parsing
- Error recovery and reporting
- Operator precedence handling

### 2. AST (`solx_ast`)

**Purpose:** Define the structure of parsed SOL-X programs.

**Key Types:**
- `Program` - Root node containing accounts and instructions
- `AccountDef` - Account structure definition
- `Instruction` - Instruction handler definition
- `Statement` - Statement types (init, require, assign, expr)
- `Expr` - Expression tree
- `Type` - Type system

**Design Decisions:**
- AST is serializable (for debugging/analysis)
- Types are explicit and checked
- Field order is preserved (important for deterministic layouts)

### 3. HIR (`solx_hir`)

**Purpose:** High-level Intermediate Representation with type checking and validation.

**Responsibilities:**
- Validate account type references
- Check parameter types
- Perform semantic analysis
- Prepare for code generation

**Future Enhancements:**
- Type inference
- More sophisticated validation
- Optimization passes

### 4. Codegen (`solx_codegen`)

**Purpose:** Generate Anchor Rust code from the HIR.

**Output Structure:**
1. Account structs with `#[account]` attributes
2. Program module with `#[program]` attribute
3. Instruction handler functions
4. Context structs with `#[derive(Accounts)]`

**Key Features:**
- Deterministic account layout calculation
- Proper Anchor attribute generation
- Expression translation
- Statement translation

**Account Size Calculation:**
- 8 bytes for Anchor discriminator
- Sum of field sizes based on type
- Handles nested types (Vec, Option)

### 5. CLI (`solx_cli`)

**Purpose:** User-facing command-line interface.

**Commands:**
- `new` - Create new project
- `build` - Compile SOL-X to Anchor
- `fmt` - Format source (planned)
- `test` - Run tests (planned)

**Integration:**
- Calls Anchor build/test commands
- Manages project structure
- Handles file I/O

## Project Structure

```
solx/
├── crates/
│   ├── solx_ast/       # AST definitions
│   ├── solx_parser/    # Parser implementation
│   ├── solx_hir/       # HIR and validation
│   ├── solx_codegen/   # Code generation
│   └── solx_cli/       # CLI tool
├── examples/            # Example programs
├── docs/               # Documentation
└── Cargo.toml          # Workspace configuration
```

## Design Principles

### 1. Deterministic Layouts

Account field order is preserved exactly as written, ensuring predictable serialization. This is a key differentiator from raw Rust where field order can be compiler-dependent.

### 2. Anchor Compatibility

Generated code is standard Anchor, ensuring:
- IDL generation works
- Client SDKs work
- Existing tooling works
- No lock-in

### 3. Minimal Surface Area

The language is intentionally limited to what's needed for Solana programs. This reduces complexity and makes the compiler easier to maintain.

### 4. Explicit Over Implicit

- Explicit types everywhere
- Explicit account initialization
- Explicit field ordering
- No magic or hidden behavior

## Future Enhancements

### Short Term
- Better error messages with source spans
- Formatter implementation
- More expression types
- Test scaffolding

### Medium Term
- Direct IDL generation (without Anchor build)
- More sophisticated type system
- Optimization passes
- Language server protocol (LSP)

### Long Term
- Advanced features (loops, conditionals)
- Macro system
- Package manager
- Standard library

## Testing Strategy

- Unit tests for parser
- Integration tests for full compilation
- Example programs as test cases
- Property-based testing for codegen

## Error Handling

Errors are propagated using `anyhow::Result`:
- Parse errors include source location
- Type errors include context
- Codegen errors include helpful messages

Future: Better error reporting with spans and suggestions.
