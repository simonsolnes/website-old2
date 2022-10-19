use nom::error::Error;
use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::str::from_utf8;

#[derive(Debug)]
#[allow(dead_code)]
struct Request<'a> {
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
    if let Ok(request) = parse_request_header(&mut stream) {
        println!("Got request: {:?}", request);
        println!("Read body: {:?}", from_utf8(&request.read_body).unwrap())
    } else {
        println!("there was an error parsing the request")
    }
}

fn parse_request_header(stream: &mut TcpStream) -> Result<Request, &str> {
    let mut acc = Vec::<u8>::new();
    const BUFFER_SIZE: usize = 4;
    let mut buffer = [0u8; BUFFER_SIZE];

    let parsed_start_line: Result<(&[u8], (String, String, String)), nom::Err<Error<&[u8]>>> = loop {
        match stream.read(&mut buffer) {
            Ok(read_bytes) => {
                let actual_read = &buffer[..read_bytes];
                if acc.len() == 0 {
                    match http_parsers::parse_start_line(actual_read) {
                        Err(nom::Err::Incomplete(_)) => {
                            if read_bytes < BUFFER_SIZE {
                                return Err("The request is incomplete");
                            }
                            acc.extend_from_slice(actual_read)
                        }
                        any @ _ => break any,
                    }
                } else {
                    acc.extend_from_slice(actual_read);
                    match http_parsers::parse_start_line(&acc) {
                        Err(nom::Err::Incomplete(_)) => {
                            if read_bytes < BUFFER_SIZE {
                                return Err("The request is incomplete");
                            }
                        }
                        any @ _ => break any,
                    }
                }
            }
            e @ Err(_) => {
                println!("Socket read error {:?}", e);
                return Err("Socket read error");
            }
        }
    };
    let mut acc: Vec<u8>;
    let start_line = match parsed_start_line {
        Ok((surplus, parsed_http_title)) => {
            acc = surplus.to_vec();
            parsed_http_title
        }
        _ => return Err("The HTTP title could not be parsed"),
    };

    let mut headers = HashMap::<String, String>::new();

    let mut buffer = [0u8; BUFFER_SIZE];

    loop {
        let mut done_with_headers = false;
        loop {
            match http_parsers::parse_header(&acc) {
                Ok((surplus, (key, value))) => {
                    if let Some(_) = headers.insert(key.to_string(), value.to_string()) {
                        return Err("Identical keys in header");
                    }
                    acc = surplus.to_vec();
                }
                Err(nom::Err::Incomplete(_)) => break,
                Err(nom::Err::Error(Error { input, code }))
                    if code == nom::error::ErrorKind::IsNot && input[0] == 13 =>
                {
                    done_with_headers = true;
                    break;
                }
                _ => return Err("Could not parse a spesific header"),
            }
        }
        if done_with_headers {
            break;
        }
        match stream.read(&mut buffer) {
            Ok(read_bytes) => {
                let actual_read = &buffer[..read_bytes];
                acc.extend_from_slice(actual_read);
            }
            e @ Err(_) => {
                println!("Socket read error {:?}", e);
                return Err("Socket read error");
            }
        }
    }

    Ok({
        let (method, target, version) = start_line;
        Request {
            method,
            target,
            version,
            headers,
            read_body: acc,
            unread_stream: stream,
        }
    })
}

mod http_parsers {
    use nom::{
        bytes::streaming::{is_not, tag},
        character::streaming::alpha1,
        combinator::{map, map_res},
        sequence::{preceded, separated_pair, terminated, tuple},
        IResult,
    };
    use std::str::from_utf8;

    pub fn space(i: &[u8]) -> IResult<&[u8], &[u8]> {
        tag(b" ")(i)
    }

    pub fn letters(i: &[u8]) -> IResult<&[u8], &str> {
        map_res(alpha1, |s| std::str::from_utf8(s))(i)
    }
    pub fn url(i: &[u8]) -> IResult<&[u8], &str> {
        map_res(is_not(" "), |s| std::str::from_utf8(s))(i)
    }

    pub fn parse_start_line(i: &[u8]) -> IResult<&[u8], (String, String, String)> {
        tuple((
            map(terminated(letters, space), |s| s.to_lowercase()),
            map(terminated(url, space), |s| s.to_owned()),
            map(
                map_res(
                    terminated(preceded(tag("HTTP/"), is_not("\r")), tag("\r\n")),
                    |s| std::str::from_utf8(s),
                ),
                |s| s.to_owned(),
            ),
        ))(i)
    }
    pub fn parse_header(i: &[u8]) -> IResult<&[u8], (String, &str)> {
        terminated(
            separated_pair(
                map(map_res(is_not(":\r"), |s| from_utf8(s)), |s| {
                    s.to_lowercase()
                }),
                tag(": "),
                map_res(is_not("\r"), |s| from_utf8(s)),
            ),
            tag("\r\n"),
        )(i)
    }
}
