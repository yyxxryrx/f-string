# f-string

[![Crates.io](https://img.shields.io/crates/v/f-string.svg)](https://crates.io/crates/f-string)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A procedural macro library that provides arbitrary expression interpolation and Python-like f-string formatting for Rust.

```toml
[dependencies]
f-string = "0.1"
```

---

## Macros

| Macro | Syntax                | Use case                              | State    | Feature flag |
|-------|-----------------------|---------------------------------------|----------|--------------|
| `f!`  | `f!("string {expr}")` | Familiar Python f-string syntax       | Unstable | `f-macro`    |
| `t!`  | `t!(string {expr})`   | Native token syntax, no quotes needed | Stable   | (default)    |

Both macros expand to [`format!`](https://doc.rust-lang.org/std/macro.format.html) (or `String::from`/`String::new` when
no expressions are present) at compile time, with no runtime overhead.

> **`f-macro` feature**: `f!` is gated behind the `f-macro` feature and may have issues. Prefer `t!` when possible.

---

## Usage

### `t!` (recommended) — token stream syntax

```rust
use f_string::t;

let name = "world";
let greeting = t!(Hello, {name}!);
let pi = t!({ std::f64::consts::PI:.4 });
let hex = t!({255:#x});
```

No quotes means no escaping for double quotes, and multi-line strings work naturally.
Common leading indentation is automatically stripped — you can align continuation
lines without affecting the output:

```rust
let s = t!(
    Hello
    World
);
assert_eq!(s, "Hello\nWorld");

// Relative indentation is preserved:
let s = t!(
    Hello
      indented
    World
);
assert_eq!(s, "Hello\n  indented\nWorld");
```

### `f!` — quoted string literals

Requires the `f-macro` feature in `Cargo.toml`:

```toml
[dependencies]
f-string = { version = "0.1", features = ["f-macro"] }
```

```rust
use f_string::f;

let name = "world";
let greeting = f!("Hello, {name}!");
let pi = f!("{std::f64::consts::PI:.4}");
let hex = f!("{255:#x}");
```

Escape braces with `{{` and `}}`:

```rust
let s = f!("{{braces}}"); // -> "{}"
```

### Expressions and format specifiers

Any Rust expression can be used inside `{}`, including method chains, function calls, and paths. Standard
Rust [format specifiers](https://doc.rust-lang.org/std/fmt/) are fully supported.

```rust
let upper = t!({ "hello".to_uppercase() });
let padded = f!("{42:0width$}");
```

### Nesting

Macros can be nested:

```rust
let s = t!(Point: {
    t!(x={p.x}, y={p.y})
});
```

---

## How it works

- **`f!`** parses the string literal, extracts expressions from `{...}` placeholders, and generates a `format!` call.
- **`t!`** parses the token stream directly, treats text outside `{...}` as string content, and generates a `format!`
  call.

Both produce no runtime overhead — the expansion happens entirely at compile time.

Additionally, when `t!` encounters a bare string literal with no format specifier
inside `{...}`, it folds the string content directly into the surrounding string
at compile time, eliminating even the `format!` call overhead:

```rust
// This expands to String::from("Hello, world!") — no format! call at all.
let s = t!(Hello, {"world"}!);
```

### Design Philosophy: `f!` vs `t!`

Two macros exist because they represent different trade-offs in Rust's macro system:

- **`t!` (token-based):** Started as a test experiment (`t` for test), but turned out to be the more robust approach. It parses the native token stream, benefiting from full IDE support and the compiler's own error reporting. The trade-off: it operates within the Rust lexer's constraints — for example, `}}` is rejected before the macro ever sees it, so brace escaping is impossible.
- **`f!` (string-based):** By taking a string literal, it bypasses the lexer's brace-matching rules, enabling true Python f-string syntax including `{{`/`}}` escaping. The trade-off: the hand-written string parser is less reliable, which is why it remains behind the `f-macro` feature flag.

---

## Limitations

- Only works with string literals (runtime strings cannot be used).
- Requires Rust edition 2024 or later. This is strictly necessary because the macros rely on new `Span` features in the 2024 edition to fully preserve formatting and provide accurate error reporting.
- `f!` needs `{{`/`}}` to escape braces.
- `t!` cannot escape braces with `{{` / `}}`. Because Rust's lexer validates brace matching *before* procedural macros are expanded, `}}` will cause a compile error (unmatched `}`) before `t!` ever sees it. As a workaround, use `{ "{" }` or `{ "}" }`.

---

## License

MIT

The `unindent` module is adapted from the [`textwrap`](https://crates.io/crates/textwrap) crate, MIT-licensed.
