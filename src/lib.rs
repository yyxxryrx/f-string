use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::multi::many1;
use nom::{AsChar, IResult, Parser};
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

fn to_compact_string(ts: TokenStream) -> String {
    let mut result = String::new();
    format_tt(ts, &mut result);
    result
}

fn sub_span(a: proc_macro::Span, b: proc_macro::Span) -> (usize, usize) {
    (
        a.start().line().saturating_sub(b.end().line()),
        a.start().column().saturating_sub(b.end().column()),
    )
}

fn sub_span2(a: proc_macro2::Span, b: proc_macro2::Span) -> (usize, usize) {
    (
        a.start().line.saturating_sub(b.end().line),
        a.start().column.saturating_sub(b.end().column),
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

struct PartialItem<T>
where
    T: syn::parse::Parse,
{
    item: T,
    remaining: proc_macro2::TokenStream,
}

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

enum Ty {
    BracketLeft,
    BracketRight,
    Str(String),
    Expr(syn::Expr, String),
    #[allow(unused)]
    None,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BracketLeft => write!(f, "BracketLeft[{{]"),
            Self::BracketRight => write!(f, "BracketRight[}}]"),
            Self::Str(s) => write!(f, "Str[{s}]"),
            Self::Expr(e, r) => write!(f, "Expr[{}{r}]", e.to_token_stream()),
            Self::None => write!(f, ""),
        }
    }
}

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

fn parse_bracket_left(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("{{")(input)?;
    Ok((input, Ty::BracketLeft))
}

fn parse_bracket_right(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("}}")(input)?;
    Ok((input, Ty::BracketRight))
}

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

fn parse_expr(input: &str) -> IResult<&str, Ty> {
    let (input, _) = tag("{")(input)?;
    let (input, expr) = take_content(input);
    let expr = syn::parse_str::<PartialItem<syn::Expr>>(expr).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            "format strings are invalid",
            nom::error::ErrorKind::Fail,
        ))
    })?;
    Ok((
        input,
        Ty::Expr(expr.item, to_compact_string(expr.remaining.into())),
    ))
}

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

fn parse(input: &str) -> IResult<&str, Ty> {
    alt((
        parse_bracket_left,
        parse_bracket_right,
        parse_expr,
        parse_str,
    ))
    .parse(input)
}

fn parse_all(input: &str) -> IResult<&str, Vec<Ty>> {
    many1(parse).parse(input)
}

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
    fn value(&self) -> String {
        match self {
            Self::Str(str) => str.clone(),
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
}

macro_rules! update {
    ($t:ident, $s:ident, $r:ident) => {
        let (l, c) = $s.map(|s| sub_span2($t.span(), s)).unwrap_or_default();
        if l > 0 {
            $r.push_str(&"\n".repeat(l));
        } else if c > 0 {
            $r.push_str(&" ".repeat(c));
        }
    };
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

impl syn::parse::Parse for Ts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut results = vec![];
        let mut string = String::new();
        let mut prev_span: Option<proc_macro2::Span> = None;
        while !input.is_empty() {
            if input.peek(syn::token::Brace) {
                let content;
                let s = syn::braced!(content in input).span;

                update!(s, prev_span, string);
                prev_span = Some(s.span());
                results.push(Ty2::Str(string.clone()));
                string.clear();

                let expr = content.parse::<syn::Expr>()?;
                let r = content.parse::<proc_macro2::TokenStream>()?.into();
                results.push(Ty2::Expr(expr, to_compact_string(r)));
                continue;
            }
            prev_span =
                Some(input.step(|cursor| {
                    p(&cursor, &mut string, prev_span).map_err(|e| cursor.error(e))
                })?);
        }
        if !string.is_empty() {
            results.push(Ty2::Str(string.clone()));
        }
        Ok(Self { results })
    }
}

#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let ts = syn::parse_macro_input!(input as Ts);
    let results = ts.results;
    let args = results
        .iter()
        .filter(|t| t.is_expr())
        .map(|t| t.expr().unwrap())
        .collect::<Vec<_>>();
    let s = results.iter().map(|t| t.value()).collect::<String>();
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
