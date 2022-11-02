use std::collections::HashMap;

use crate::parse::Parse;

use self::primitives::unreserved;
mod primitives;

/// Absolute URIs always begins with a sceme followed by a colon

/// Uri syntax components
/// https://www.rfc-editor.org/rfc/rfc3986#section-3
/// https://www.w3.org/International/wiki/IRIStatus
///
///                  host      port
///               /         \ /  \
///         foo://example.com:8042/over/there?name=ferret#nose
///         \_/   \______________/\_________/ \_________/ \__/
///          |           |            |            |        |
///       scheme     authority   abs_path        query   fragment
///          |   _____________________|__
///         / \ /                        \
///         urn:example:animal:ferret:nose
///

type Path = Vec<String>;
type Query = HashMap<String, String>;

/// The target in a HTTP request can be any of these types
#[derive(PartialEq, Debug)]
enum Target {
    Origin(Absolute<Path>, Option<Query>),
    Absolute(URI),
    //Authority(Host),
    Asterix,
}

#[derive(PartialEq, Debug)]
struct Relative<T>(T);

#[derive(PartialEq, Debug)]
struct Absolute<T>(T);

#[derive(PartialEq)]
struct URL {}

#[derive(Debug)]
struct Scheme(String);

struct Authority {
    userinfo: Option<String>,
    host: String,
    port: Option<u16>,
}

#[derive(PartialEq, Debug)]
struct HttpUrl {
    scheme: Scheme,
}

#[derive(PartialEq, Debug)]
struct URI {}

#[derive(PartialEq, Debug)]
struct Host {}

#[derive(PartialEq, Debug)]
struct IPv4Address(u8, u8, u8, u8);

enum IPLiteral {
    IPv6Address,
    IPvFuture,
}

pub trait Parser {
    fn parse(i: &str) -> Parse<&str, Self>
    where
        Self: Sized;
}
impl PartialEq for Scheme {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

mod parsers {

    use crate::parse::sequence::{serial, serial3, serial4, terminated};
    use crate::parse::str::{alpha_char, char, peek_char, take_while};
    use crate::parse::{comb::map, Parse};

    use super::primitives::{self, dec_octet, unreserved};
    use super::{Authority, IPLiteral, IPv4Address, Parser, Scheme, Target};

    impl Parser for Target {
        fn parse(i: &str) -> Parse<&str, Self> {
            terminated(map(char('*'), |_| Self::Asterix), primitives::url_end)(i)
        }
    }

    impl Parser for Scheme {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(
                serial(
                    map(alpha_char(i), |c| c.to_string()),
                    take_while(|c| {
                        c.is_ascii_alphabetic() || c.is_ascii_digit() || "+-.".contains(c)
                    }),
                ),
                |(f, r)| Scheme(format!("{f}{r}")),
            )(i)
        }
    }

    impl Parser for Authority {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(unreserved, |r| Self {
                userinfo: Some(r.to_string()),
                host: "".to_string(),
                port: None,
            })(i)
        }
    }
    impl Parser for IPv4Address {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(
                serial4(
                    terminated(dec_octet, char('.')),
                    terminated(dec_octet, char('.')),
                    terminated(dec_octet, char('.')),
                    dec_octet,
                ),
                |(a, b, c, d)| IPv4Address(a, b, c, d),
            )(i)
        }
    }
    impl Parser for IPLiteral {
        fn parse(i: &str) -> Parse<&str, Self> {
            Parse::Retreat("IPv6 parser not implemented".to_string())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_target_asterix() {
        assert_eq!(Target::parse("*"), Parse::Success(Target::Asterix, ""));
        assert_eq!(Target::parse("* "), Parse::Success(Target::Asterix, " "));
        println!("{:?}", Target::parse("*s").is_err());
        assert!(Target::parse("*s").is_err());
    }

    #[test]
    fn test_scheme() {
        assert_eq!(Scheme("N".to_string()), Scheme("n".to_string()));
        assert_eq!(
            Scheme::parse("N "),
            Parse::Success(Scheme("n".to_string()), " ")
        );
        assert_eq!(
            Scheme::parse("N+3 "),
            Parse::Success(Scheme("n+3".to_string()), " ")
        );
        assert!(Scheme::parse(" N+3 ").is_err());
    }

    #[test]
    fn test_ipv4_address() {
        assert_eq!(
            IPv4Address::parse("1.1.1.1"),
            Parse::Success(IPv4Address(1, 1, 1, 1), "")
        );
        assert_eq!(
            IPv4Address::parse("0.23.100.255"),
            Parse::Success(IPv4Address(0, 23, 100, 255), "")
        );
        assert!(IPv4Address::parse("0.23.300.29").is_err());
        assert!(IPv4Address::parse("0.23.3").is_err());
        assert!(IPv4Address::parse("340.23.3.0").is_err());
        assert!(IPv4Address::parse("14023.3.0").is_err());
    }
}
