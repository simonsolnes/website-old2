use parse::str::literal;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

mod http;
mod json;
mod parse;
mod url;
mod urlv2;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Request<'a> {
    method: String,
    target: String,
    version: String,
    headers: HashMap<String, String>,

    read_body: Vec<u8>,
    unread_stream: &'a mut TcpStream,
}

fn main() {
    println!("{:?}", literal("fisk")("fiske"));
    //serve();
}

fn serve() {
    let listener = match TcpListener::bind("127.0.0.1:80") {
        Err(why) => {
            eprintln!("{}", why);
            panic!("Could not bind to address");
        }
        Ok(value) => value,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(s) => handle_client(s),
            Err(e) => eprintln!("Something is wrong witht the stream: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    if let Ok(request) = http::parse_heads(&mut stream) {
        println!("Got request: {:?}", request);
    } else {
        eprintln!("There was an error parsing the request")
    }
}
