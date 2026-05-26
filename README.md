# f-string

> ⚠️ **警告**：此库仍处于开发阶段，建议仅在个人项目中使用，生产环境请谨慎评估。

一个为 Rust 提供类似 Python f-string 字符串格式化功能的过程宏库，支持两种直观的字符串插值语法。

## 特性

- 🎯 **双宏支持**：提供 `f!`（字符串字面量）和 `t!`（原生语法）两种宏
- 🔍 **表达式支持**：支持任意 Rust 表达式、方法调用和复杂路径
- 🧩 **嵌套支持**：支持在宏内嵌套使用其他宏调用
- 🛡️ **编译时检查**：所有表达式在编译时进行类型检查
- 📦 **零运行时开销**：完全在编译时展开，无性能损失
- 🎨 **格式控制**：完整支持 Rust 格式化说明符（对齐、精度、进制等）

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
f-string = "0.1.0"
```

## 使用方法

本库提供两个宏：

- **`f!`**：接受字符串字面量，类似 Python f-string 语法
- **`t!`**：接受原生 Token 流，无需引号包裹

### f! 宏 - 字符串字面量方式

```rust
use f_string::f;

let name = "World";
let greeting = f!("Hello, {name}!");
// 展开为: format!("Hello, {}", name)
println!("{}", greeting); // 输出: Hello, World!
```

### t! 宏 - 原生语法方式

```rust
use f_string::t;

let name = "World";
let greeting = t!(Hello, {name}!);
// 展开为: format!("Hello, {}!", name)
println!("{}", greeting); // 输出: Hello, World!
```

`t!` 宏的优势：

- 无需引号包裹，更简洁
- 天然支持多行字符串
- 不需要转义引号

### 结构体字段访问

两个宏都支持访问结构体字段：

```rust
use f_string::{f, t};

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };

// 使用 f! 宏
let s1 = f!("Point: x={p.x}, y={p.y}");
// 展开为: format!("Point: x={}, y={}", p.x, p.y)

// 使用 t! 宏
let s2 = t!(Point: x={p.x}, y={p.y});
// 展开为: format!("Point: x={}, y={}", p.x, p.y)

println!("{}", s1); // 输出: Point: x=1, y=2
println!("{}", s2); // 输出: Point: x=1, y=2
```

### 表达式支持

支持任意 Rust 表达式，包括函数调用、方法链、路径等：

```rust
use f_string::{f, t};

// f! 宏
let result = f!("2 + 2 = {2 + 2}");
// 展开为: format!("2 + 2 = {}", 2 + 2)

let upper = f!("{\"hello\".to_uppercase()}");
// 展开为: format!("{}", "hello".to_uppercase())

// t! 宏（更简洁，无需转义）
let upper2 = t!({ "hello".to_uppercase() });
// 展开为: format!("{}", "hello".to_uppercase())

// 复杂表达式
let now = f!("{std::time::SystemTime::now():?}");
let max = t!({ i32::MAX });
```

### 格式化说明符

完整支持 Rust 的格式化语法：

```rust
use f_string::{f, t};

let value = 42;

// 十六进制
let hex = f!("{value:#x}");
// 展开为: format!("{:#x}", value)

// 保留两位小数
let pi = 3.14159;
let rounded = f!("{pi:.2}");
// 展开为: format!("{:.2}", pi)

// 对齐和填充
let centered = t!({"Hi":-^10});
// 展开为: format!("{:-^10}", "Hi")
// 输出: "----Hi----"

// 动态宽度
let width = 10;
let padded = f!("{100:0width$}");
// 展开为: format!("{:0width$}", 100)
// 输出: "0000000100"
```

### 转义花括号（仅 f! 宏）

在 `f!` 宏中，使用双花括号转义：

```rust
use f_string::f;

let s = f!("{{}}");  // 输出: {}
// 展开为: String::from("{}")
```

> **注意**：`t!` 宏无法转义花括号，如有需要可以考虑换用`f!`宏。

### 空字符串优化

当没有表达式时，宏会优化为 `String::from()` 或 `String::new()`：

```rust
use f_string::{f, t};

// f! 宏
let text = f!("This is string");
// 展开为: String::from("This is string")

let empty = f!("");
// 展开为: String::new()

// t! 宏
let text2 = t!(This is string);
// 展开为: String::from("This is string")

let empty2 = t!();
// 展开为: String::new()
```

### 嵌套宏调用

支持在宏内嵌套使用其他宏：

```rust
use f_string::{f, t};

let p = Point { x: 1, y: 2 };

// f! 宏嵌套
let nested_f = f!("Location: {f!(\"(x={p.x}, y={p.y})\")}");
// 外层展开为: format!("Location: {}", f!("(x={p.x}, y={p.y})"))
// 内层展开为: format!("(x={}, y={})", p.x, p.y)
// 最终输出: "Location: (x=1, y=2)"

// t! 宏嵌套（更清晰）
let nested_t = t!(Location: {
    t!(point(x={p.x}, y={p.y}))
});
// 输出: "Location: point(x=1, y=2)"
```

### 多行字符串（t! 宏优势）

`t!` 宏天然支持多行字符串，无需特殊处理：

```rust
use f_string::t;

let multiline = t!(The first line
The second line
{"The third line"}
The end);
// 展开为: format!("The first line\nThe second line\n{}The end", "The third line")
// 输出:
// The first line
// The second line
// The third line
// The end
```

### 原始字符串支持

两个宏都支持原始字符串：

```rust
use f_string::{f, t};

// f! 宏使用原始字符串
let raw_f = f!(r#"{concat!("Hi ", 3.14, "(pi)!")}"#);
// 展开为: format!("{}", concat!("Hi ", 3.14, "(pi)!"))

// t! 宏天然支持，无需 r# 前缀
let raw_t = t!({ concat!("Hi ", 3.14, "(pi)!") });
// 输出: "Hi 3.14(pi)!"
```

## 示例

查看完整的使用示例：

- [examples/f_basic.rs](examples/f_basic.rs) - `f!` 宏的完整示例
- [examples/t_basic.rs](examples/t_basic.rs) - `t!` 宏的完整示例

运行示例：

```bash
# 运行 f! 宏示例
cargo run --example f_basic

# 运行 t! 宏示例
cargo run --example t_basic
```

## 工作原理

### f! 宏

`f!` 宏在编译时解析字符串字面量：

1. 解析字符串中的 `{}` 占位符
2. 提取花括号内的 Rust 表达式
3. 将普通文本和占位符转换为 `format!` 宏的参数
4. 生成对应的 `format!` 或 `String::from()` 调用

例如：

```rust
f!("x: {p.x}, y: {p.y}")
// 转换为
format!("x: {}, y: {}", p.x, p.y)
```

### t! 宏

`t!` 宏直接解析 Token 流：

1. 接收未加引号的 Token 序列
2. 识别花括号 `{}` 包围的表达式
3. 将其余部分作为字符串字面量
4. 生成对应的 `format!` 或 `String::from()` 调用

例如：

```rust
t!(x: {p.x}, y: {p.y})
// 转换为
format!("x: {}, y: {}", p.x, p.y)
```

## f! vs t! 对比

| 特性    | f! 宏                  | t! 宏                |
|-------|-----------------------|---------------------|
| 语法    | `f!("string {expr}")` | `t!(string {expr})` |
| 需要引号  | ✅ 是                   | ❌ 否                 |
| 多行支持  | 需要 `\n`               | ✅ 天然支持              |
| 引号转义  | 需要                    | ✅ 不需要               |
| 花括号转义 | `{{}}`                | `{}`                |
| 原始字符串 | `r#"..."#`            | ✅ 天然支持              |
| 适用场景  | 简单字符串、Python 风格       | 复杂表达式、多行文本          |

**选择建议**：

- 使用 `f!`：熟悉 Python f-string、简单字符串插值
- 使用 `t!`：多行文本、包含引号的字符串、更简洁的语法

## 优势

- **类型安全**：所有表达式在编译时进行类型检查
- **性能优异**：零运行时解析开销，完全在编译时处理
- **灵活选择**：提供两种语法适应不同场景
- **功能完整**：支持所有 Rust 表达式和格式化选项
- **易于迁移**：Python 开发者可快速上手

## 限制

- 仅支持字符串字面量（不支持运行时字符串）
- 需要 Rust 2024 edition 或更高版本
- `f!` 宏中的花括号需要转义（使用 `{{}}`）
- 复杂的嵌套可能导致可读性下降

## 许可证

本项目采用 MIT 许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

灵感来源于 Python 的 f-string 功能。
