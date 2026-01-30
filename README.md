# SOL-X

![SOL-X](https://i.imgur.com/ECONnCj.png)

> The contract DSL that compiles to Anchor

**SOL-X v0.1.0 — Anchor, Less Boilerplate**

SOL-X is a declarative domain-specific language for writing Solana programs. It compiles to Anchor, providing a cleaner syntax while maintaining full compatibility with the Anchor ecosystem.

*"It's not just less code—it's declarative clarity. Anchor stays. The boilerplate goes."* — SOL-X

- **Minimal boilerplate** — Write contracts with far less code than raw Anchor
- **Deterministic layouts** — Explicit account field ordering, predictable serialization
- **Type safety** — Strong typing with compile-time checks
- **Anchor compatible** — Generates standard Anchor programs, works with existing tooling
- **Less macro soup** — No `#[derive(Accounts)]` per instruction
- **One-line init** — `init account state: CounterState payer authority`
- **Readable instructions** — Declarative `instruction` blocks, not scattered handlers
- **Same toolchain** — `anchor build`, `anchor deploy`, IDL, clients unchanged

SOL-X v0.1.0 delivers MVP: parser, codegen, CLI, and example programs. Same chain, same Anchor—just less code.

**SOL-X vs raw Anchor — Line counts (single program file)**

| Program   | SOL-X (.solx) | Raw Anchor (lib.rs) | Verdict        |
|----------|----------------|----------------------|----------------|
| Counter  | 24             | ~95                  | SOL-X ~4× less |
| Escrow   | 37             | ~160                 | SOL-X ~4× less |

*Equivalent Anchor programs: declare_id, account structs, `#[derive(Accounts)]` per instruction, handler functions, error types. SOL-X emits the same structure from a single declarative file.*

---

## Prerequisites

To build and run SOL-X, and to build/deploy generated programs:

- **Rust** — Stable toolchain, 2021 edition (see [Cargo.toml](Cargo.toml))
- **Anchor CLI** — For `anchor build` / `anchor deploy` on generated projects ([Anchor](https://www.anchor-lang.com/) / [install](https://www.anchor-lang.com/docs/installation))

**Verify:**

```bash
rustc --version   # stable, 2021
anchor --version  # e.g. 0.29.x or later
```

---

## Features

- Declarative `program` / `account` / `instruction` syntax
- Deterministic account layout (field order preserved)
- `init account`, `require`, assignments, expressions
- No macro soup, no per-instruction `#[derive(Accounts)]` by hand
- Error propagation with `require`; full Anchor compatibility
- Single source file → standard Anchor Rust → same IDL and clients

---

## Quick Start

```bash
# Install CLI
cargo install --path crates/solx_cli

# New project
solx new my_program
cd my_program
solx build

# Build and deploy (Anchor as usual)
anchor build
anchor deploy
```

---

## Language Overview

### Program structure

```solx
program Counter

account CounterState {
  authority: Pubkey
  count: u64
}

instruction initialize(authority: Signer, state: CounterState) {
  init account state: CounterState payer authority
  state.authority = authority.key
  state.count = 0
}

instruction increment(authority: Signer, state: CounterState) {
  require state.authority == authority.key
  state.count += 1
}
```

### Account definitions

Accounts define the data structures stored on-chain:

```solx
account MyAccount {
  owner: Pubkey
  balance: u64
  active: bool
}
```

Supported types: `Pubkey`, `u8`–`u64`, `i8`–`i64`, `bool`, `String`, `Vec<T>`, `Option<T>`.

### Instructions

Instructions define the program's entry points. Parameter types: `Signer`, account types (e.g. `CounterState`), primitives.

### Statements

- **Init:** `init account state: CounterState payer authority`
- **Require:** `require condition` or `require condition, "Error message"`
- **Assignment:** `state.count = 0`, `state.count += 1`, etc.
- **Expressions:** field access, binary/unary ops (`+`, `-`, `==`, `&&`, `!`, …).

---

## Examples

- `examples/counter/` — Counter with initialize, increment, decrement (24 lines)
- `examples/escrow/` — Escrow contract (37 lines)

---

## CLI

| Command | Description |
|---------|-------------|
| `solx new <name>` | Create a new SOL-X project |
| `solx build [--path <dir>]` | Compile SOL-X to Anchor Rust |
| `solx fmt [--path <dir>]` | Format SOL-X (coming soon) |
| `solx test [--path <dir>]` | Run Anchor tests |

---

## Architecture

1. **Parser** (`solx_parser`) — `.solx` → AST  
2. **HIR** (`solx_hir`) — High-level IR, type checking  
3. **Codegen** (`solx_codegen`) — Anchor Rust  
4. **CLI** (`solx_cli`) — User-facing commands  

---

## Status

- **v0.1.0** — MVP: parser, codegen, CLI, counter & escrow examples.
- Next: formatter, better errors, more expression types, IDL, test scaffolding.

---

## License

MIT OR Apache-2.0 © 2026 SOL-X Contributors

*Anchor stays. The syntax doesn’t have to.*
