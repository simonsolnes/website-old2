use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::str::from_utf8;

mod http;

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
    serve();
}

fn serve() {
    let listener = match TcpListener::bind("127.0.0.1:80") {
        Err(why) => {
            eprintln!("{}", why);
            exit(1);
        }
        Ok(value) => value,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(s) => handle_client(s),
            Err(e) => eprintln!("{}", e),
        }
    }
    println!("Listener {:?}", listener);
}

fn handle_client(mut stream: TcpStream) {
    if let Ok(request) = http::parse_heads(&mut stream) {
        println!("Got request: {:?}", request);
        println!("Read body: {:?}", from_utf8(&request.read_body).unwrap())
    } else {
        println!("there was an error parsing the request")
    }
}
