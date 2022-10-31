mod json;

fn main() {
    println!("Hello, world! {}", 3_2);

    println!(
        "{:?}",
        json::object("{\"something\": false, \"something else\": [true, false, \"hello\", 3, {\"fisk\": 3}]}"));
}
