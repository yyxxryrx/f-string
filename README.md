# f-string

> 不建议您将此库用于生产用途，建议仅个人项目使用
>
> 此库还在发展中，建议您谨慎使用

一个为 Rust 提供类似 Python f-string 字符串格式化功能的过程宏库。

## 特性

- 🎯 简洁直观的字符串插值语法
- 🔍 支持任意 Rust 表达式
- 🧩 支持嵌套 f-string
- 🛡️ 编译时类型检查
- 📦 零运行时开销

## 使用方法

### 基本用法

```rust
use f_string::f;

let name = "World";
let greeting = f!("Hello, {name}!");
// 展开为: format!("Hello, {}", name)
println!("{}", greeting); // 输出: Hello, World!
```

### 结构体字段访问

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
let s = f!("Point: x={p.x}, y={p.y}");
// 展开为: format!("Point: x={}, y={}", p.x, p.y)
println!("{}", s); // 输出: Point: x=1, y=2
```

### 表达式支持

```rust
let result = f!("2 + 2 = {2 + 2}");
// 展开为: format!("2 + 2 = {}", 2 + 2)

let upper = f!("{\"hello\".to_uppercase()}");
// 展开为: format!("{}", "hello".to_uppercase())
```

### 格式化说明符

```rust
let value = 42;
let formatted = f!("{value:#x}");  // 十六进制
// 展开为: format!("{:#x}", value)

let pi = 3.14159;
let rounded = f!("{pi:.2}");  // 保留两位小数
// 展开为: format!("{:.2}", pi)
```

### 转义花括号

```rust
let s = f!("{{}}");  // 输出: {}
// 展开为: String::from("{}")
```

### 空字符串优化

```rust
let empty = f!("");
// 展开为: String::new()
```

### 嵌套 f-string

```rust
let p = Point { x: 1, y: 2 };
let s = f!("Location: {f!("({p.x}, {p.y})")}");
// 外层展开为: format!("Location: {}", f!("({p.x}, {p.y})"))
// 内层展开为: format!("({}, {})", p.x, p.y)
```

### 复杂表达式

```rust
let s = f!("{std::time::SystemTime::now():?}");
// 展开为: format!("{:?}", std::time::SystemTime::now())

let max = f!("{std::i32::MAX}");
// 展开为: format!("{}", std::i32::MAX)
```

## 示例

查看 [examples/f_basic.rs](examples/f_basic.rs) 获取`f!`的完整的使用示例。

查看 [examples/t_basic.rs](examples/t_basic.rs) 获取`t!`的完整的使用示例。

运行示例：

```bash
cargo run --example basic_f
```

## 工作原理

`f!` 宏在编译时将 f-string 语法转换为标准的 `format!` 宏调用：

1. 解析字符串字面量
2. 提取 `{}` 中的表达式
3. 生成对应的 `format!` 调用

例如：

```rust
f!("x: {p.x}, y: {p.y}")
// 转换为
format!("x: {}, y: {}", p.x, p.y)
```

## 优势

- **类型安全**：所有表达式在编译时进行类型检查
- **性能优异**：零运行时解析开销，完全在编译时处理
- **熟悉语法**：对于 Python 开发者来说非常直观
- **功能完整**：支持所有 Rust 表达式和格式化选项

## 限制

- 仅支持字符串字面量（不支持运行时字符串）
- 需要 Rust 2024 edition 或更高版本

## 许可证

本项目采用 MIT 许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

灵感来源于 Python 的 f-string 功能。
