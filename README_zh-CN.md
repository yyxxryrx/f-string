# f-string

为 Rust 提供类似 Python f-string 字符串插值功能的过程宏库。

```toml
[dependencies]
f-string = "0.1"
```

`unindent`模块改编自MIT授权的[`textwrap`](https://crates.io/crates/textwrap) crate。


---

## 宏

| 宏    | 语法                | 说明              | 状态  | 特性开关      |
|------|-------------------|-----------------|-----|-----------|
| `f!` | `f!("字符串 {表达式}")` | 带引号的字符串字面量语法    | 不稳定 | `f-macro` |
| `t!` | `t!(字符串 {表达式})`   | 原生 Token 流，无需引号 | 稳定  | (默认)      |

两种宏均在编译时展开为 [`format!`](https://doc.rust-lang.org/std/macro.format.html)（无表达式时展开为 `String::from` /
`String::new`），无运行时开销。

> **注意**：`f!` 需要启用 `f-macro` 特性，目前仍不稳定，建议优先使用 `t!`。

---

## 使用

### `t!`（推荐）

```rust
use f_string::t;

let name = "world";
let greeting = t!(Hello, {name}!);
let pi = t!({ std::f64::consts::PI:.4 });
let hex = t!({255:#x});
```

无需引号，天然支持多行和双引号。公共前导缩进会被自动去除，排版不影响输出：

```rust
let s = t!(
    第一行
    第二行
);
assert_eq!(s, "第一行\n第二行");

// 相对缩进被保留：
let s = t!(
    第一行
      缩进行
    第三行
);
assert_eq!(s, "第一行\n  缩进行\n第三行");
```

### `f!`

需在 `Cargo.toml` 中启用 `f-macro` 特性：

```toml
[dependencies]
f-string = { version = "0.1", features = ["f-macro"] }
```

```rust
use f_string::f;

let name = "world";
let greeting = f!("Hello, {name}!");
```

使用 `{{` 和 `}}` 转义花括号：

```rust
let s = f!("{{花括号}}"); // -> "{}"
```

### 表达式与格式化说明符

`{}` 内支持任意 Rust 表达式（方法链、函数调用、路径等），并完整支持标准 [format 说明符](https://doc.rust-lang.org/std/fmt/)：

```rust
let upper = t!({ "hello".to_uppercase() });
let padded = f!("{42:0width$}");
```

### 嵌套

宏支持嵌套使用：

```rust
let s = t!(坐标: {
    t!(x={p.x}, y={p.y})
});
```

---

## 工作原理

- **`f!`** 解析字符串字面量，提取 `{...}` 占位符中的表达式，生成 `format!` 调用。
- **`t!`** 直接解析 Token 流，将 `{...}` 之外的文本作为字符串内容，生成 `format!` 调用。

两者均在编译时完成展开，无运行时开销。

此外，当 `t!` 中 `{...}` 内是一个不带格式化参数的字符串字面量时，该字符串会在编译时直接拼入周围的字符串中，连 `format!`
调用都省去：

```rust
// 展开为 String::from("Hello, world!")，完全没有 format! 调用。
let s = t!(Hello, {"world"}!);
```

---

## 限制

- 仅支持字符串字面量（无法用于运行时字符串）。
- 需要 Rust edition 2024 或更高版本。
- `f!` 使用 `{{`/`}}` 转义花括号；`t!` 无法转义（可用 `{ "{" }` 替代）。

---

## 许可证

MIT
