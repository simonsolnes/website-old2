use std::collections::HashMap;

use crate::parse::Parse;

use self::primitives::unreserved;
pub mod primitives;

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

#[derive(PartialEq, Debug)]
struct Authority {
    user_info: Option<UserInfo>,
    host: Host,
    port: Option<Port>,
}

#[derive(PartialEq, Debug)]
struct Port(u16);

#[derive(PartialEq, Debug)]
struct HttpUrl {
    scheme: Scheme,
}

#[derive(PartialEq, Debug)]
struct URI {}

#[derive(PartialEq, Debug)]
enum Host {
    Literal(IPLiteral),
    IPv4(IPv4Address),
    Name(RegistrationName),
}

#[derive(PartialEq, Debug)]
pub struct UserInfo(String);

#[derive(PartialEq, Debug)]
pub struct RegistrationName(String);

#[derive(PartialEq, Debug)]
struct IPv4Address(u8, u8, u8, u8);

#[derive(PartialEq, Debug)]
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

    use crate::parse::comb::{either, either3, either4, optional, ret};
    use crate::parse::repeat::repeat_some;
    use crate::parse::sequence::{preceded, serial, serial3, serial4, terminated};
    use crate::parse::str::{alpha_char, char, peek_char, take_while};
    use crate::parse::{comb::map, Parse};

    use super::primitives::{self, dec_hextet, dec_octet, percent_encoded, sub_delim, unreserved};
    use super::{
        Authority, Host, IPLiteral, IPv4Address, Parser, Port, RegistrationName, Scheme, Target,
        UserInfo,
    };

    impl Parser for Authority {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(
                serial3(
                    optional(terminated(UserInfo::parse, char('@'))),
                    Host::parse,
                    optional(preceded(char(':'), Port::parse)),
                ),
                |(user_info, host, port)| Authority {
                    user_info,
                    host,
                    port,
                },
            )(i)
        }
    }

    impl Parser for Port {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(dec_hextet, |r| Port(r))(i)
        }
    }

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
    impl Parser for UserInfo {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(
                repeat_some(either4(
                    map(unreserved, |s| s.to_string()),
                    percent_encoded,
                    map(sub_delim, |c| c.to_string()),
                    map(char(':'), |c| c.to_string()),
                )),
                |r| UserInfo(r.join("")),
            )(i)
        }
    }

    impl Parser for RegistrationName {
        fn parse(i: &str) -> Parse<&str, Self> {
            map(
                repeat_some(either3(
                    map(unreserved, |s| s.to_string()),
                    percent_encoded,
                    map(sub_delim, |c| c.to_string()),
                )),
                |r| RegistrationName(r.join("")),
            )(i)
        }
    }
    impl Parser for Host {
        fn parse(i: &str) -> Parse<&str, Self> {
            either3(
                map(IPLiteral::parse, |r| Host::Literal(r)),
                map(IPv4Address::parse, |r| Host::IPv4(r)),
                map(RegistrationName::parse, |r| Host::Name(r)),
            )(i)
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        #[test]
        fn test_target_asterix() {
            assert_eq!(Target::parse("*"), Parse::Success(Target::Asterix, ""));
            assert_eq!(Target::parse("* "), Parse::Success(Target::Asterix, " "));
            println!("{:?}", Target::parse("*s").is_retreat());
            assert!(Target::parse("*s").is_retreat());
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
            assert!(Scheme::parse(" N+3 ").is_retreat());
        }

        #[test]
        fn test_ipv4_address() {
            assert_eq!(
                IPv4Address::parse("1.1.1.1"),
                Parse::Limit(Some(IPv4Address(1, 1, 1, 1)), "")
            );
            assert_eq!(
                IPv4Address::parse("0.23.100.255"),
                Parse::Success(IPv4Address(0, 23, 100, 255), "")
            );
            assert!(IPv4Address::parse("0.23.300.29").is_retreat());
            assert!(IPv4Address::parse("0.23.3").is_limit());
            assert!(IPv4Address::parse("340.23.3.0").is_retreat());
            assert!(IPv4Address::parse("14023.3.0").is_retreat());
        }
        #[test]
        fn test_user_info() {
            assert_eq!(
                UserInfo::parse("hello "),
                Parse::Success(UserInfo("hello".to_string()), " ")
            );
            assert_eq!(
                UserInfo::parse("he!llo "),
                Parse::Success(UserInfo("he!llo".to_string()), " ")
            );
            assert_eq!(
                UserInfo::parse("he:!l%20lo "),
                Parse::Success(UserInfo("he:!l lo".to_string()), " ")
            );
        }
        #[test]
        fn test_host() {
            assert_eq!(
                Host::parse("2.34.2.5"),
                Parse::Limit(Some(Host::IPv4(IPv4Address(2, 34, 2, 5))), "")
            );
            assert_eq!(
                Host::parse("2.34.2.234"),
                Parse::Success(Host::IPv4(IPv4Address(2, 34, 2, 234)), "")
            );
            assert_eq!(
                Host::parse("youtube.com/"),
                Parse::Success(Host::Name(RegistrationName("youtube.com".to_string())), "/")
            );
        }
        #[test]
        fn test_authority() {
            assert_eq!(
                Authority::parse("youtube.com/"),
                Parse::Success(
                    Authority {
                        user_info: None,
                        host: Host::Name(RegistrationName("youtube.com".to_string())),
                        port: None,
                    },
                    "/"
                )
            );
            assert_eq!(
                Authority::parse("a@youtube.com:3"),
                Parse::Limit(
                    Some(Authority {
                        user_info: Some(UserInfo("a".to_string())),
                        host: Host::Name(RegistrationName("youtube.com".to_string())),
                        port: Some(Port(3)),
                    }),
                    ""
                )
            );
            assert_eq!(
                Authority::parse("a:b@2.3.4.255"),
                Parse::Limit(
                    Some(Authority {
                        user_info: Some(UserInfo("a:b".to_string())),
                        host: Host::IPv4(IPv4Address(2, 3, 4, 255)),
                        port: None,
                    }),
                    ""
                )
            );
            assert_eq!(
                Authority::parse("a:b@2.3.4.255:65530"),
                Parse::Success(
                    Authority {
                        user_info: Some(UserInfo("a:b".to_string())),
                        host: Host::IPv4(IPv4Address(2, 3, 4, 255)),
                        port: Some(Port(65530)),
                    },
                    ""
                )
            );
        }
    }
}
