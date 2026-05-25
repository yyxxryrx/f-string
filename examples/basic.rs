use f_string::f;

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn upper(s: &str) -> String {
    s.to_uppercase()
}

fn main() {
    let p = Point::new(1, 2);
    let a = f!("x: {p.x}, y: {p.y}"); // Result: "x: 1, y: 2", expand: format!("x: {}, y: {}", p.x, p.y)
    let b = f!("{std::time::SystemTime::now():?}"); // Result: "SystemTime { intervals: ... }", expand: format!("{:?}", std::time::SystemTime::now());
    let c = f!(r#"{upper("hi")}"#); // Result: "HI", expand: format!("{}", upper("hi"))
    let s = f!("This is string"); // Result: "This is string", expand: String::from("This is string")
    let empty = f!(""); // Result: "", expand: String::new()
    let regarding_him = f!(r#"{"Hi":-^10}"#); // Result: "----Hi----", expand: format!("{:-^10}", "Hi")

    let d = f!("{{}}"); // Result: "{}", expand: String::from("{}")
    let e = f!("{Point {x: 2, y: 2}:#?}"); // Result: (I can't write it all down), expand: format!("{:#?}", Point {x: 2, y: 2})
    let f = f!("{std::i32::MAX}"); // Result: "2147483647", expand: format!("{}", std::i32::MAX)
    let g = f!(r#"{concat!("Hi ", 3.14, "(pi)!")}"#); // Result: "Hi 3.14(pi)!", expand: format!("{}", concat!("Hi ", 3.14, "(pi)!"))

    // Result: "You are at the point(x=1, y=2)"
    // expand (outer): format!("You are at the {}", f!("point(x={p.x}, y={p.y})"))
    // expand (inner): f!("...") -> format!("point(x={}, y={})", p.x, p.y)
    // final: format!("You are at the {}", format!("point(x={}, y={})", p.x, p.y))
    let h = f!(r#"You are at the {f!("point(x={p.x}, y={p.y})")}"#);

    println!("{a}");
    println!("{b}");
    println!("{c}");
    println!("{s}");
    println!("{empty}");
    println!("{regarding_him}");
    println!("{d}");
    println!("{e}");
    println!("{f}");
    println!("{g}");
    println!("{h}");
}
