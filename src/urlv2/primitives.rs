use std::ops::Range;

use crate::parse::{
    comb::{either, either3, either5, either_of, map, map_bool, map_option, result, ret},
    sequence::{serial, serial3},
    str::{char, digit, peek_char, pop, take, take_while},
    tools::accept_limit,
    Parse,
};

#[allow(dead_code)]
pub mod ascii_charsets {
    pub const NUMERIC: &'static str = "0123456789";
    pub const ALPHA_SMALL: &'static str = "abcdefghijklmnopqrstuvwxyz";
    pub const ALPHA_CAPITAL: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const CONTROL: &'static str = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\
                                      \x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12\x13\
                                      \x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\
                                      \x1d\x1E\x1F";

    pub const URL_UNRESERVED: &'static str = "-._~";
    pub const GEN_DELIMS: &'static str = ":/?#[]@";
    pub const SUB_DELIMS: &'static str = "!$&'()*+,;=";
    pub const PERCENT: char = '%';
    pub const URL_ILLEGAL: &'static str = " \"<>\\^`}{|";
}

pub fn is_ucschar(c: char) -> bool {
    for g in [
        0xA0..0xD7FF,
        0xF900..0xFDCF,
        0xFDF0..0xFFEF,
        0x10000..0x1FFFD,
        0x20000..0x2FFFD,
        0x30000..0x3FFFD,
        0x40000..0x4FFFD,
        0x50000..0x5FFFD,
        0x60000..0x6FFFD,
        0x70000..0x7FFFD,
        0x80000..0x8FFFD,
        0x90000..0x9FFFD,
        0xA0000..0xAFFFD,
        0xB0000..0xBFFFD,
        0xC0000..0xCFFFD,
        0xD0000..0xDFFFD,
        0xE1000..0xEFFFD,
    ] {
        if g.contains(&(c as u32)) {
            return true;
        }
    }
    false
}

pub fn unreserved(i: &str) -> Parse<&str, &str> {
    take_while(|c| c.is_ascii_alphanumeric() || is_ucschar(c) || "-._~".contains(c))(i)
}

fn digit_within(range: Range<u8>) -> impl Fn(&str) -> Parse<&str, u8> {
    move |i: &str| map_bool(digit, |&d| d >= range.start && d <= range.end)(i)
}

/// Parse number between 0 and 255, with no leading 0 allowed
pub fn dec_octet(input: &str) -> Parse<&str, u8> {
    either5(
        map(
            serial3(ret(char('2'), 200), ret(char('5'), 50), digit_within(0..5)),
            |(h, t, b)| h + t + b,
        ),
        map(
            serial3(ret(char('2'), 200), digit_within(0..4), digit_within(0..9)),
            |(h, a, b)| h + (a * 10) + b,
        ),
        map(
            serial3(ret(char('1'), 100), digit_within(0..9), digit_within(0..9)),
            |(h, a, b)| h + (a * 10) + b,
        ),
        map(serial(digit_within(1..9), digit_within(0..9)), |(a, b)| {
            (a * 10) + b
        }),
        map(digit_within(0..9), |a| a),
    )(input)
}

/// Returns true if c is contained in ascii and in either of these:
/// `CONTROL`: Invisible characters from 0 to 1F
/// `GEN_DELIMS`: :/?#[]@
/// `SUB_DELIMS`: !$&'()*+,;=
/// `URL_ILLEGAL`: "< >\\^`}{|
pub fn is_url_terminative(c: char) -> bool {
    let c = c as u32;
    c < 128
        && c > 47
        && ((c > 57 && c < 65) || c == 96 || (c > 90 && c < 95) || c > 122 && c != 126)
        || c < 45
        || c == 47
}

pub fn url_end(i: &str) -> Parse<&str, ()> {
    match peek_char(i) {
        Parse::Success(c, _) => match is_url_terminative(c) {
            true => Parse::Success((), i),
            false => Parse::Retreat("is not url terminative".to_string()),
        },
        Parse::Limit(_, _) => Parse::Success((), i),
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
    }
}

// pub fn hex_digit()

// pub fn percent_encoded(i: &str) -> Parse<&str, char> {
// preceded(char('%'),
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unreserved() {
        assert_eq!(unreserved("hello"), Parse::Limit(Some("hello"), ""));
        assert_eq!(unreserved("helloå˚˚~"), Parse::Limit(Some("helloå˚˚~"), ""));
        assert_eq!(unreserved("hello/s"), Parse::Success("hello", "/s"));
    }
    #[test]
    fn test_num_within() {
        assert_eq!(digit_within(2..5)("45"), Parse::Success(4, "5"));
        assert_eq!(digit_within(2..5)("4"), Parse::Success(4, ""));
        assert_eq!(digit_within(0..1)("0"), Parse::Success(0, ""));
        assert!(digit_within(2..5)("13").is_err());
        assert!(digit_within(0..5)("6").is_err());
    }
    #[test]
    fn test_dec_octet() {
        assert_eq!(dec_octet("0"), Parse::Success(0, ""));
        assert_eq!(dec_octet("10"), Parse::Success(10, ""));
        assert_eq!(dec_octet("00"), Parse::Success(0, "0"));
        assert_eq!(dec_octet("245"), Parse::Success(245, ""));
        assert_eq!(dec_octet("24"), Parse::Success(24, ""));
        assert_eq!(dec_octet("260"), Parse::Success(26, "0"));
        assert_eq!(dec_octet("250"), Parse::Success(250, ""));
        assert_eq!(dec_octet("249"), Parse::Success(249, ""));
        assert_eq!(dec_octet("350"), Parse::Success(35, "0"));
        assert_eq!(dec_octet("351"), Parse::Success(35, "1"));
        assert_eq!(dec_octet("255"), Parse::Success(255, ""));
        assert_eq!(dec_octet("256"), Parse::Success(25, "6"));
        assert_eq!(dec_octet("2:90"), Parse::Success(2, ":90"));
    }
}
