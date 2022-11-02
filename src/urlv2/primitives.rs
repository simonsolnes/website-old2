use crate::parse::{
    str::{peek_char, take_while},
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
        Parse::Deficient(_) => Parse::Success((), i),
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unreserved() {
        assert_eq!(unreserved("hello "), Parse::Success("hello", " "))
    }
}
