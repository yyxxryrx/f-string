use f_string::t;

#[test]
fn pure_text() {
    assert_eq!(t!(), String::new());
    assert_eq!(t!(Hello), String::from("Hello"));
    assert_eq!(t!(Hello, World!), String::from("Hello, World!"));
    assert_eq!(t!(A( B) C[D ] E), String::from("A( B) C[D ] E"));
    assert_eq!(
        t!(
            First line.
            Second Line.
        ),
        String::from("First line.\nSecond Line.")
    )
}

#[test]
fn ident_format() {
    let a = 1;
    let b = 2f32;
    let c = String::from("C");
    let d = "d";
    assert_eq!(t!({ a }), format!("{a}"));
    assert_eq!(t!({ b }), format!("{b}"));
    assert_eq!(t!({ c }), format!("{c}"));
    assert_eq!(t!({ d }), format!("{d}"));
    assert_eq!(t!({ a } { b } { c } { d }), format!("{a} {b} {c} {d}"));
    assert_eq!(t!({ a }{ b } { c }{ d }), format!("{a}{b} {c}{d}"));
}

#[test]
fn expr_format() {
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Point {
        pub fn new(x: i32, y: i32) -> Self {
            Self { x, y }
        }
    }

    assert_eq!(t!(1 + 1 = { 1 + 1 }), format!("1 + 1 = {}", 1 + 1));
    assert_eq!(t!({ 100:#x }), format!("{:#x}", 100));

    let p = Point::new(-1, 2);
    assert_eq!(
        t!(p = Point(x = { p.x }, y = { p.y })),
        format!("p = Point(x = {}, y = {})", p.x, p.y)
    );
    assert_eq!(
        t!({Point::new(-3, 2).y:010}),
        format!("{:010}", Point::new(-3, 2).y)
    );
    assert_eq!(
        t!({Point::new(2, -3):#?}),
        format!("{:#?}", Point::new(2, -3))
    );
    assert_eq!(t! {
        <div>
        <p class="title">Point</p>
        <span>x={p.x}</span>
        <span>y={p.y}</span>
        </div>
    }, format!("<div>\n<p class=\"title\">Point</p>\n<span>x={}</span>\n<span>y={}</span>\n</div>", p.x, p.y))
}
