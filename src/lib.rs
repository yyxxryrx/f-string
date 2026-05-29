//! Python-like f-string formatting for Rust.
//!
//! This crate provides two procedural macros for string interpolation with
//! arbitrary expressions and full [`format!`] specifier support:
//!
//! | Macro | Syntax | Use case |
//! |---|---|---|
//! | `f!` | `f!("string {expr}")` | Quoted string literals (requires `f-macro` feature) |
//! | [`t!`] | `t!(string {expr})` | Native token syntax, no quotes needed |
//!
//! Both macros expand at compile time with zero runtime overhead.
//!
//! # Quick start
//!
//! ```rust
//! use f_string::t;
//!
//! let name = "world";
//! let s = t!(Hello, {name}!);
//! ```
//!
//! See the individual macro docs for details and more examples.

mod unindent;

#[cfg(feature = "f-macro")]
use nom::{
    AsChar, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    multi::many1,
};
use proc_macro::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt::Formatter;
use syn::parse::ParseStream;
use syn::spanned::Spanned;

trait GetOpenAndClose {
    fn open(&self) -> &'static str;
    fn close(&self) -> &'static str;
}

macro_rules! impl_delimiter {
    ($($t:ident)::*) => {
        impl GetOpenAndClose for $($t)::* {
            fn open(&self) -> &'static str {
                match self {
                    Self::Brace => "{",
                    Self::Bracket => "[",
                    Self::Parenthesis => "(",
                    Self::None => "∅",
                }
            }

            fn close(&self) -> &'static str {
                match self {
                    Self::Brace => "}",
                    Self::Bracket => "]",
                    Self::Parenthesis => ")",
                    Self::None => "∅",
                }
            }
        }
    };
}

impl_delimiter!(Delimiter);

impl_delimiter!(proc_macro2::Delimiter);

fn to_string(ts: TokenStream) -> String {
    let mut result = String::new();
    format_tt(ts, &mut result);
    result
}

/// 计算 `a` - `b` 的距离
fn sub_span(a: proc_macro::Span, b: proc_macro::Span) -> (usize, usize) {
    (
        a.start().line().saturating_sub(b.end().line()),
        a.start().column().saturating_sub(b.end().column()),
    )
}

/// 计算 `a` - `b` 的距离
fn sub_span2(a: proc_macro2::Span, b: proc_macro2::Span) -> (usize, usize) {
    (
        a.start().line.saturating_sub(b.end().line),
        a.start().column.saturating_sub(b.end().column),
    )
}

/// 计算 `a` - `b` 的距离（开头相减）
fn sub_start_span2(a: proc_macro2::Span, b: proc_macro2::Span) -> (usize, usize) {
    (
        a.start().line.saturating_sub(b.start().line),
        a.start().column.saturating_sub(b.start().column),
    )
}

/// 计算 `a` - `b` 的距离（结尾相减）
fn sub_end_span2(a: proc_macro2::Span, b: proc_macro2::Span) -> (usize, usize) {
    (
        a.end().line.saturating_sub(b.end().line),
        a.end().column.saturating_sub(b.end().column),
    )
}

fn format_tt(ts: TokenStream, out: &mut String) {
    let mut prev_span: Option<proc_macro::Span> = None;
    for tt in ts {
        let (l, c) = prev_span
            .map(|s| sub_span(tt.span(), s))
            .unwrap_or_default();
        if l > 0 {
            out.push_str(&"\n".repeat(l));
        }
        if c > 0 {
            out.push_str(&" ".repeat(c));
        }
        prev_span = Some(tt.span());
        match tt {
            TokenTree::Group(group) => {
                out.push_str(group.delimiter().open());
                format_tt(group.stream(), out);
                out.push_str(group.delimiter().close());
            }
            TokenTree::Ident(ident) => out.push_str(&ident.to_string()),
            TokenTree::Punct(p) => out.push_str(&p.to_string()),
            TokenTree::Literal(lit) => out.push_str(&lit.to_string()),
        }
    }
}

#[cfg(feature = "f-macro")]
struct PartialItem<T>
where
    T: syn::parse::Parse,
{
    item: T,
    remaining: proc_macro2::TokenStream,
}

#[cfg(feature = "f-macro")]
impl<T> syn::parse::Parse for PartialItem<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item = input.parse::<T>()?;
        // 将剩余 Token 收集起来（不报错）
        let remaining = input.parse()?; // 或 input.step(|c| Ok((c.remaining_token_stream(), c)))
        Ok(PartialItem { item, remaining })
    }
}

#[cfg(feature = "f-macro")]
enum Ty {
    BracketLeft,
    BracketRight,
    Str(String),
    Expr(syn::Expr, String),
    #[allow(unused)]
    None,
}

#[cfg(feature = "f-macro")]
impl Ty {
    fn value(&self, in_format: bool) -> String {
        match self {
            Self::BracketLeft => "{".repeat(if in_format { 2 } else { 1 }),
            Self::BracketRight => "}".repeat(if in_format { 2 } else { 1 }),
            Self::Expr(.., r) => format!("{{{r}}}"),
            Self::Str(s) => s.clone(),
            Self::None => String::new(),
        }
    }

    fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(..))
    }

    fn expr(&self) -> Option<syn::Expr> {
        match self {
            Self::Expr(expr, ..) => Some(expr.clone()),
            _ => None,
        }
    }
}

#[cfg(feature = "f-macro")]
fn parse_bracket_left(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("{{")(input)?;
    Ok((input, Ty::BracketLeft))
}

#[cfg(feature = "f-macro")]
fn parse_bracket_right(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("}}")(input)?;
    Ok((input, Ty::BracketRight))
}

#[cfg(feature = "f-macro")]
fn take_content(input: &str) -> (&str, &str) {
    let mut depth = 1;
    let mut content_length = 0;
    let mut ex_length = 0;
    for c in input.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        ex_length += c.len();
        if depth == 0 {
            break;
        }
        content_length += c.len();
    }
    (&input[ex_length..], &input[..content_length])
}

#[cfg(feature = "f-macro")]
fn parse_expr(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("{")(input)?;
    let (input, expr) = take_content(input);
    let expr = syn::parse_str::<PartialItem<syn::Expr>>(expr).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            "format strings are invalid",
            nom::error::ErrorKind::Fail,
        ))
    })?;
    Ok((input, Ty::Expr(expr.item, to_string(expr.remaining.into()))))
}

#[cfg(feature = "f-macro")]
fn parse_str(input: &str) -> IResult<&str, Ty> {
    if input.starts_with('{') || input.starts_with('}') {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Fail,
        )));
    }
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    }
    let (input, value) = take_while(|c| c != '{' && c != '}').parse(input)?;
    Ok((input, Ty::Str(value.to_string())))
}

#[cfg(feature = "f-macro")]
fn parse(input: &str) -> IResult<&str, Ty> {
    alt((
        parse_bracket_left,
        parse_bracket_right,
        parse_expr,
        parse_str,
    ))
    .parse(input)
}

#[cfg(feature = "f-macro")]
fn parse_all(input: &str) -> IResult<&str, Vec<Ty>> {
    many1(parse).parse(input)
}

/// Python-like f-string formatting via string literals.
///
/// This macro parses a string literal and expands it into a [`format!`] call,
/// with expressions in `{...}` interpolated directly.
///
/// ```rust
/// use f_string::f;
///
/// let name = "world";
/// let s = f!("Hello, {name}!");
/// ```
///
/// Escape literal braces with `{{` and `}}`:
///
/// ```rust
/// # use f_string::f;
/// let s = f!("{{braces}}"); // -> "{}"
/// ```
///
/// **Status: unstable.** The `f!` macro is gated behind the `f-macro` feature
/// and may have parsing issues with certain expressions. Prefer the [`t!`] macro
/// for new code — it is stable and does not require an extra feature flag or
/// the `nom` dependency.
#[cfg(feature = "f-macro")]
#[proc_macro]
pub fn f(input: TokenStream) -> TokenStream {
    let value = match syn::parse::<syn::LitStr>(input.clone()) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    }
    .value();
    if value.is_empty() {
        return quote::quote! {
            String::new()
        }
        .into();
    }
    let r = match parse_all(&value) {
        Ok((_, tys)) => tys,
        Err(e) => {
            return syn::Error::new(proc_macro2::Span::call_site(), e.to_string())
                .into_compile_error()
                .into();
        }
    };
    let args = r
        .iter()
        .filter(|s| s.is_expr())
        .map(|s| s.expr().unwrap())
        .collect::<Vec<syn::Expr>>();

    let s = r
        .iter()
        .map(|ty| ty.value(!args.is_empty()))
        .collect::<String>();
    let s = syn::LitStr::new(&s, proc_macro2::Span::call_site());
    if args.is_empty() {
        quote::quote! {
            String::from(#s)
        }
    } else {
        quote::quote! {
            format!(#s, #(#args),*)
        }
    }
    .into()
}

enum Ty2 {
    Str(String),
    Expr(syn::Expr, String),
}

impl Ty2 {
    fn value(&self, is_format: bool) -> String {
        match self {
            Self::Str(str) => {
                if is_format {
                    str.replace('{', "{{").replace('}', "}}")
                } else {
                    str.clone()
                }
            }
            Self::Expr(.., r) => format!("{{{r}}}"),
        }
    }

    fn expr(&self) -> Option<syn::Expr> {
        match self {
            Self::Str(..) => None,
            Self::Expr(expr, ..) => Some(expr.clone()),
        }
    }

    fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(..))
    }
}

impl std::fmt::Display for Ty2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(str) => write!(f, "String[{str}]"),
            Self::Expr(expr, e) => write!(f, "Expr[{}{e}]", expr.to_token_stream().to_string()),
        }
    }
}

struct Ts {
    results: Vec<Ty2>,
    first_span: Option<proc_macro2::Span>,
    last_span: Option<proc_macro2::Span>,
}

macro_rules! update {
    ($t:ident, $s:ident, $r:ident) => {
        let (l, c) = $s.map(|s| sub_span2($t.span(), s)).unwrap_or_default();
        update!(l: l, c: c, s: $t.span().start().column, $r);
    };
    (l: $l: expr, c: $c: expr, s: $s: expr, $r:ident) => {
        let l = $l;
        let c = $c;
        if l > 0 {
            $r.push_str(&"\n".repeat(l));
            $r.push_str(&" ".repeat($s));
        } else if c > 0 {
            $r.push_str(&" ".repeat(c));
        }
    }
}

fn p<'c, 'a>(
    cursor: &syn::buffer::Cursor<'c>,
    result: &mut String,
    prev_span: Option<proc_macro2::Span>,
) -> Result<(proc_macro2::Span, syn::buffer::Cursor<'c>), &'a str> {
    if let Some((ident, cursor)) = cursor.ident() {
        update!(ident, prev_span, result);
        result.push_str(&ident.to_string());
        return Ok((ident.span(), cursor));
    }
    if let Some((mut group_cursor, d, ds, cursor)) = cursor.any_group() {
        update!(ds, prev_span, result);
        result.push_str(d.open());
        let mut prev_span = group_cursor.span();
        while !group_cursor.eof() {
            (prev_span, group_cursor) = p(&group_cursor, result, Some(prev_span))?;
        }
        result.push_str(d.close());
        return Ok((ds.span(), cursor));
    }
    if let Some((p, cursor)) = cursor.punct() {
        update!(p, prev_span, result);
        result.push_str(&p.to_string());
        return Ok((p.span(), cursor));
    }
    if let Some((lit, cursor)) = cursor.literal() {
        update!(lit, prev_span, result);
        result.push_str(&lit.to_string());
        return Ok((lit.span(), cursor));
    }
    if let Some((life, cursor)) = cursor.lifetime() {
        update!(life, prev_span, result);
        result.push_str(&life.to_string());
        return Ok((life.span(), cursor));
    }
    Err("Unknown type")
}

macro_rules! repeat {
    ($mac:ident, $d:ident, $input:expr, $res:ident, $s:ident, $prev:ident, $first:ident) => {
        let content;
        let s = syn::$mac!(content in $input).span;

        if $first.is_none() {
            $first = Some(s.span());
        }

        update!(s, $prev, $s);
        $prev = Some(s.span());
        $s += Delimiter::$d.open();
        $res.push(Ty2::Str($s.clone()));
        $s.clear();

        // 计算括号内开头的空白部分
        let mut head = String::new();
        let (l, c) = sub_start_span2(content.span(), s.span());
        update!(l: l, c: c.saturating_sub(1), s: content.span().start().column, head);

        let mut ts = content.parse::<Self>()?;

        $res.push(Ty2::Str(head));

        $res.append(&mut ts.results);

        // 计算括号中末尾的空白部分
        if let Some(last_span) = ts.last_span {
            let mut tail = String::new();
            let (l, c) = sub_end_span2(s.span(), last_span);
            update!(l: l, c: c.saturating_sub(1), s: s.span().start().column, tail);
            $res.push(Ty2::Str(tail));
        }

        // let ts_span = ts.last_span;
        // update!(s, ts_span, $s);
        $s += Delimiter::$d.close();
    };
}

impl syn::parse::Parse for Ts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut results = vec![];
        let mut string = String::new();
        let mut first_span: Option<proc_macro2::Span> = None;
        let mut prev_span: Option<proc_macro2::Span> = None;
        while !input.is_empty() {
            if input.peek(syn::token::Paren) {
                repeat!(
                    parenthesized,
                    Parenthesis,
                    input,
                    results,
                    string,
                    prev_span,
                    first_span
                );
                continue;
            }
            if input.peek(syn::token::Bracket) {
                repeat!(
                    bracketed, Bracket, input, results, string, prev_span, first_span
                );
                continue;
            }
            if input.peek(syn::token::Brace) {
                let content;
                let s = syn::braced!(content in input).span;

                if first_span.is_none() {
                    first_span = Some(s.span());
                }

                update!(s, prev_span, string);
                prev_span = Some(s.span());
                results.push(Ty2::Str(string.clone()));
                string.clear();

                let expr = content.parse::<syn::Expr>()?;
                let r = if !content.is_empty() {
                    content.parse::<syn::Token![:]>()?;
                    let r = content.parse::<proc_macro2::TokenStream>()?.into();
                    String::from(":") + &to_string(r)
                } else {
                    String::new()
                };
                match &expr {
                    syn::Expr::Lit(lit) => match lit.lit {
                        syn::Lit::Str(ref str) if r.is_empty() => {
                            results.push(Ty2::Str(str.value()))
                        }
                        _ => results.push(Ty2::Expr(expr, r)),
                    },
                    _ => results.push(Ty2::Expr(expr, r)),
                };
                continue;
            }
            prev_span =
                Some(input.step(|cursor| {
                    p(&cursor, &mut string, prev_span).map_err(|e| cursor.error(e))
                })?);
            if first_span.is_none() {
                first_span = prev_span;
            }
        }
        if !string.is_empty() {
            results.push(Ty2::Str(string.clone()));
        }
        Ok(Self {
            results,
            first_span,
            last_span: prev_span,
        })
    }
}

fn parse_t(input: TokenStream) -> syn::Result<(String, Vec<syn::Expr>)> {
    let ts = syn::parse::<Ts>(input)?;
    let results = ts.results;
    let args = results
        .iter()
        .filter(|t| t.is_expr())
        .map(|t| t.expr().unwrap())
        .collect::<Vec<_>>();
    let mut s = results
        .iter()
        .map(|t| t.value(!args.is_empty()))
        .collect::<String>();

    let span = proc_macro2::Span::call_site();

    if let Some(first_span) = ts.first_span
        && span.start().line < first_span.start().line
    {
        s.insert_str(0, &" ".repeat(first_span.start().column))
    }

    Ok((unindent::dedent(&s), args))
}

/// Python-like f-string formatting via token stream syntax.
///
/// This macro parses a raw token stream directly — no quotes needed. Text
/// outside `{...}` is treated as string content, and expressions inside `{...}`
/// are interpolated. The macro expands to a [`format!`] call at compile time.
///
/// ```rust
/// use f_string::t;
///
/// let name = "world";
/// let s = t!(Hello, {name}!);
/// ```
///
/// Multi-line strings and double quotes work naturally.
/// Common leading indentation is automatically stripped (like Python's
/// `textwrap.dedent`), so you can align continuation lines without
/// affecting the output:
///
/// ```rust
/// # use f_string::t;
/// let s = t!(
///     Hello
///     World
/// );
/// assert_eq!(s, "Hello\nWorld");
/// ```
///
/// Relative indentation is preserved:
///
/// ```rust
/// # use f_string::t;
/// let s = t!(
///     Hello
///       indented
///     World
/// );
/// assert_eq!(s, "Hello\n  indented\nWorld");
/// ```
///
/// Format specifiers are fully supported:
///
/// ```rust
/// # use f_string::t;
/// let s = t!({ std::f64::consts::PI:.4 });
/// ```
///
/// Unlike `f!`, `t!` does not require a feature flag and is considered stable.
/// Braces cannot be escaped — use `{ "{" }` instead of `{{`.
///
/// ### Comments
///
/// Rust comments (`//`, `/* */`) are stripped by the lexer before the macro
/// sees the token stream. This means a `//` in the middle of your text will
/// consume the rest of the line:
///
/// ```ignore
/// // The URL part after "https:" is lost to the comment:
/// let url = t!(https://example.com); // expands to "https:"
/// ```
///
/// Workaround: wrap the problematic part in `{ "" }`. Thanks to constant
/// folding, this has no runtime cost:
///
/// ```rust
/// # use f_string::t;
/// let url = t!(https:{"//example.com"});
/// assert_eq!(url, "https://example.com");
/// ```
///
/// ### Constant folding for string literals
///
/// If the expression inside `{...}` is a bare string literal with no format
/// specifier, it is folded directly into the surrounding string at compile
/// time rather than being passed as a runtime argument:
///
/// ```rust
/// # use f_string::t;
/// let name = "ferris";
/// // Folded at compile time — no format! call:
/// let a = t!(Hello, {"world"}!);
/// // Runtime interpolation via format!:
/// let b = t!(Hello, {name}!);
/// ```
#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let (s, args) = match parse_t(input) {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };
    let lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
    match (args.is_empty(), s.is_empty()) {
        (true, false) => quote::quote! {
            String::from(#lit)
        },
        (true, true) => quote::quote! {
            String::new()
        },
        (false, false) => quote::quote! {
            format!(#lit, #(#args), *)
        },
        _ => unreachable!(),
    }
    .into()
}

/// Like [`t!`], but expands to [`println!`] instead of returning a `String`.
///
/// ```rust
/// use f_string::t_println;
///
/// let name = "world";
/// t_println!(Hello, {name}!);  // prints: Hello, world!
/// ```
#[proc_macro]
pub fn t_println(input: TokenStream) -> TokenStream {
    let (s, args) = match parse_t(input) {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };
    let lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
    match (args.is_empty(), s.is_empty()) {
        (true, false) => quote::quote! {
            println!(#lit)
        },
        (true, true) => quote::quote! {
            println!()
        },
        (false, false) => quote::quote! {
            println!(#lit, #(#args), *)
        },
        _ => unreachable!(),
    }
    .into()
}
