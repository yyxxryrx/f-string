use f_string::t;

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

struct Person<'a> {
    name: &'a str,
    age: u8,
}

impl<'a> Person<'a> {
    fn new(name: &'a str, age: u8) -> Self {
        Self { name, age }
    }
}

fn upper(s: &str) -> String {
    s.to_uppercase()
}

fn main() {
    let p = Point::new(1, 2);
    let a = t!(x: {p.x}, y: {p.y}); // Result: "x: 1, y: 2", expand: format!("x: {}, y: {}", p.x, p.y)
    let b = t!(Time is {std::time::SystemTime::now():?}); // Result: "Time is SystemTime { intervals: ... }", expand: format!("{:?}", std::time::SystemTime::now());
    let c = t!({ upper("hi") }); // Result: "HI", expand: format!("{}", upper("hi"))

    // Result: "The first line\n    The second line\n    The third line\n    The end"
    // expand: String::from("The first line\n    The second line\n    The third line\n    The end")
    let d = t!(The first line
    The second line
    {"The third line"}
    The end);

    let s = t!(This is string); // Result: "This is string", expand: String::from("This is string")
    let empty = t!(); // Result: "", expand: String::new()
    let regarding_him = t!({"Hi":-^10}); // Result: "----Hi----", expand: format!("{:-^10}", "Hi")

    let e = t!({ Point {x: 2, y: 2}:#? }); // Result: (I can't write it all down), expand: format!("{:#?}", Point {x: 2, y: 2})
    let f = t!({ i32::MAX }); // Result: "2147483647", expand: format!("{}", i32::MAX)
    let g = t!({ concat!("Hi ", 3.14, "(pi)!") }); // Result: "Hi 3.14(pi)!", expand: format!("{}", concat!("Hi ", 3.14, "(pi)!"))

    // Result: "You are at the point(x=1, y=2)"
    // expand (outer): format!("You are at the {}", t!("point(x={p.x}, y={p.y})"))
    // expand (inner): t!("...") -> format!("point(x={}, y={})", p.x, p.y)
    // final: format!("You are at the {}", format!("point(x={}, y={})", p.x, p.y))
    let h = t!(You are at the {
        t!(point(x={p.x}, y={p.y}))
    });

    // Result: "I am Alice. I am 12 years old."
    // expand: format!("I am {}. I am {} years old.", person.name, person.age)
    let person = Person::new("Alice", 12);
    let i = t!(I am { person.name }. I am { person.age } years old.);

    // Result: "0000000100"
    // expand: format!("{:0width$}", 100)
    let width = 10;
    let j = t!({100:0width$});
    let l = t!( a(a x b ) );

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
    println!("{i}");
    println!("{j}");
    println!("{l}");
}
