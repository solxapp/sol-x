# SOL-X Language Specification

## Overview

SOL-X is a declarative DSL for writing Solana programs. It compiles to Anchor, providing a cleaner syntax while maintaining full compatibility.

## Syntax

### Program Declaration

```solx
program ProgramName
```

The program name must be a valid Rust identifier.

### Account Definitions

```solx
account AccountName {
  field1: Type1
  field2: Type2
  ...
}
```

Accounts define on-chain data structures. Field order is significant for deterministic serialization.

**Supported Types:**

- `Pubkey` - 32-byte Solana public key
- `u8`, `u16`, `u32`, `u64` - Unsigned integers (1, 2, 4, 8 bytes)
- `i8`, `i16`, `i32`, `i64` - Signed integers (1, 2, 4, 8 bytes)
- `bool` - Boolean (1 byte)
- `String` - UTF-8 string (4-byte length prefix + data)
- `Vec<T>` - Dynamic array of type T
- `Option<T>` - Optional value of type T

### Instructions

```solx
instruction instruction_name(
  param1: Type1,
  param2: Type2,
  ...
) {
  statement1
  statement2
  ...
}
```

**Parameter Types:**

- `Signer` - Signer account (must sign the transaction)
- `AccountName` - Account type (e.g., `CounterState`)
- Primitive types: `Pubkey`, `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `bool`, `String`

### Statements

#### Initialize Account

```solx
init account var_name: AccountType payer payer_name [signer signer_name]
```

Initializes a new account. The account must be a parameter of the instruction.

- `var_name` - Variable name (must match a parameter)
- `AccountType` - Account type name
- `payer_name` - Name of the payer signer
- `signer_name` - Optional signer for the account

#### Require (Assertion)

```solx
require condition
require condition, "Error message"
```

Fails the instruction if the condition is false.

#### Assignment

```solx
target = value
target += value
target -= value
target *= value
target /= value
target %= value
```

Assigns a value to a target. Compound assignments are syntactic sugar:
- `x += y` is equivalent to `x = x + y`

#### Expression Statement

```solx
expression
```

Evaluates an expression (useful for function calls, etc.).

### Expressions

#### Identifiers

```solx
variable_name
```

References a parameter or account field.

#### Field Access

```solx
object.field
object.field.subfield
```

Accesses fields of accounts or nested structures.

#### Literals

```solx
123          // Integer
-42          // Negative integer
true         // Boolean
false        // Boolean
"hello"      // String
```

#### Binary Operations

```solx
left + right    // Addition
left - right    // Subtraction
left * right    // Multiplication
left / right    // Division
left % right    // Modulo
left == right   // Equality
left != right   // Inequality
left < right    // Less than
left <= right   // Less than or equal
left > right    // Greater than
left >= right   // Greater than or equal
left && right   // Logical AND
left || right   // Logical OR
```

#### Unary Operations

```solx
!operand    // Logical NOT
-operand    // Negation
```

### Operator Precedence

1. Unary: `!`, `-`
2. Multiplicative: `*`, `/`, `%`
3. Additive: `+`, `-`
4. Relational: `<`, `<=`, `>`, `>=`
5. Equality: `==`, `!=`
6. Logical AND: `&&`
7. Logical OR: `||`

## Examples

### Counter Program

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

### Escrow Program

```solx
program Escrow

account EscrowState {
  maker: Pubkey
  taker: Pubkey
  amount: u64
  mint: Pubkey
  initialized: bool
}

instruction initialize(
  maker: Signer,
  taker: Pubkey,
  amount: u64,
  mint: Pubkey,
  escrow: EscrowState
) {
  init account escrow: EscrowState payer maker
  escrow.maker = maker.key
  escrow.taker = taker
  escrow.amount = amount
  escrow.mint = mint
  escrow.initialized = true
}
```

## Compilation

SOL-X compiles to standard Anchor programs:

1. Account structs become `#[account]` structs
2. Instructions become `pub fn` functions in a `#[program]` module
3. Parameters become `#[derive(Accounts)]` context structs
4. Statements are translated to equivalent Rust code

The generated code is fully compatible with Anchor tooling (IDL generation, client SDKs, etc.).
