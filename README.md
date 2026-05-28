# f-string

[![Crates.io](https://img.shields.io/crates/v/f-string.svg)](https://crates.io/crates/f-string)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A procedural macro library that provides Python-like f-string formatting for Rust.

```toml
[dependencies]
f-string = "0.1"
```

---

## Macros

The library provides two macros:

| Macro | Syntax                | Use case                              | State    |
|-------|-----------------------|---------------------------------------|----------|
| `f!`  | `f!("string {expr}")` | Familiar Python f-string syntax       | Unstable |
| `t!`  | `t!(string {expr})`   | Native token syntax, no quotes needed | Stable   |

Both macros expand to [`format!`](https://doc.rust-lang.org/std/macro.format.html) (or `String::from`/`String::new` when
no expressions are present) at compile time, with no runtime overhead.

---

## Usage

### `f!` — quoted string literals

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

### `t!` — token stream syntax

```rust
use f_string::t;

let name = "world";
let greeting = t!(Hello, {name}!);
let pi = t!({ std::f64::consts::PI:.4 });
```

No quotes means no escaping for double quotes, and multi-line strings work naturally:

```rust
let s = t!(Line one
Line two
{"Line three"});
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

---

## Limitations

- Only works with string literals (runtime strings cannot be used).
- Requires Rust 2021 edition.
- `f!` needs `{{`/`}}` to escape braces; `t!` cannot escape braces (use `{ "{" }` as a workaround).

---

## License

MIT
