use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::exit;
//use std::str::from_utf8;
use nom::{
    bytes::streaming::tag,
    character::streaming::alpha1,
    combinator::map,
    sequence::{preceded, terminated, tuple},
    IResult,
};

static EXAMPLE_HTTP_REQUEST: &'static str = "GET /hello.htm HTTP/1.1
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)
Host: www.tutorialspoint.com
Accept-Language: en-us
Accept-Encoding: gzip, deflate
Connection: Keep-Alive
";

#[derive(Debug)]
struct Request {
    //version: String,
    method: String,
    //target: String,
    // headers: HashMap<String, String>,
    // body: Option<&'a [u8]>,
}

/*
impl Request {
    fn parse(input: &str) -> IResult<&str, Self, ()> {
        let method = map(character::complete::alpha1::<&str, ()>, |m| {
            m.to_string().to_lowercase()
        });
        let mut p = map(method, |method| Self { method });
        p(input)
    }
}
*/

mod http_parsers {
    use nom::{
        bytes::streaming::{is_not, tag},
        character::streaming::{alpha1, anychar, char},
        combinator::{map, map_res, not},
        multi::many1,
        sequence::{delimited, preceded, terminated, tuple},
        IResult,
    };

    pub fn space(i: &[u8]) -> IResult<&[u8], &[u8]> {
        tag(b" ")(i)
    }

    pub fn letters(i: &[u8]) -> IResult<&[u8], &str> {
        map_res(alpha1, |s| std::str::from_utf8(s))(i)
    }
    pub fn url(i: &[u8]) -> IResult<&[u8], &str> {
        map_res(is_not(" "), |s| std::str::from_utf8(s))(i)
    }

    pub fn parse_start_line(i: &[u8]) -> IResult<&[u8], (&str, &str, &str)> {
        tuple((
            terminated(letters, space),
            terminated(url, space),
            map_res(terminated(is_not("\r"), tag("\r\n")), |s| {
                std::str::from_utf8(s)
            }),
        ))(i)
    }
}

fn handle_client(stream: &mut TcpStream) {
    println!("Handlin connection: {:?}", stream);

    let mut acc = Vec::<u8>::new();
    const buffer_size: usize = 1;
    let mut buffer = [0u8; buffer_size];

    let parsed_start_line: Result<(&[u8], (&str, &str, &str)), nom::Err<nom::error::Error<&[u8]>>> = loop {
        match stream.read(&mut buffer) {
            Ok(read_bytes) => {
                println!("buffer: {:?}", buffer);
                println!("read_bytes: {:?}", read_bytes);
                let actual_read = &buffer[..read_bytes];
                if acc.len() == 0 {
                    match http_parsers::parse_start_line(actual_read) {
                        Err(nom::Err::Incomplete(_)) => {
                            if read_bytes < buffer_size {
                                panic!("the request needs more")
                            }
                            acc.extend_from_slice(actual_read)
                        }
                        any @ _ => break any,
                    }
                } else {
                    acc.extend_from_slice(actual_read);
                    match http_parsers::parse_start_line(&acc) {
                        Err(nom::Err::Incomplete(_)) => {
                            if read_bytes < buffer_size {
                                panic!("the request needs more")
                            }
                        }
                        any @ _ => break any,
                    }
                }
            }
            Err(e) => println!("error: {:?}", e),
        }
    };
    let mut acc = Vec::<u8>::new();
    let http_title = match parsed_start_line {
        Ok((surplus, parsed_http_title)) => {
            acc = surplus.to_vec();
            parsed_http_title
        }
        e @ _ => panic!("invalid title {:?}", e),
    };

    println!("Done parsing title {:?}", http_title);
    println!("accu {:?}", std::str::from_utf8(&acc));
}

fn serve() {
    println!("Hello, wold!");
    let listener = match TcpListener::bind("127.0.0.1:80") {
        Err(why) => {
            eprintln!("{}", why);
            exit(1);
        }
        Ok(value) => value,
    };
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => handle_client(&mut s),
            Err(e) => eprintln!("{}", e),
        }
    }
    println!("Listener {:?}", listener);
}

fn main() {
    serve();
}
