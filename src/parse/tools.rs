use super::Parse;
use std::fmt::Display;
pub fn label<I, O, P>(p: P, label: &'static str) -> impl Fn(I) -> Parse<I, O>
where
    P: Fn(I) -> Parse<I, O>,
{
    move |i: I| match p(i) {
        s @ Parse::Success(_, _) => s,
        Parse::Retreat(_) => Parse::Retreat(format!("Error parsing {}", label)),
        Parse::Halt(_) => Parse::Halt(format!("Halted at parsing {}", label)),
        Parse::Limit(res, sur) => Parse::Limit(res, sur),
    }
}

/// Halts the parsing if the parser retreats
pub fn halt<I, O, P>(p: P) -> impl Fn(I) -> Parse<I, O>
where
    P: Fn(I) -> Parse<I, O>,
    I: Display + Copy,
{
    move |i: I| match p(i) {
        s @ Parse::Success(_, _) => s,
        Parse::Retreat(r) => Parse::Halt(format!("Halted {r}, for {i}")),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(r, s) => Parse::Limit(r, s),
    }
}

/// Turns a limit into success
pub fn accept_limit<I, O, P>(p: P) -> impl Fn(I) -> Parse<I, O>
where
    P: Fn(I) -> Parse<I, O>,
{
    move |i: I| match p(i) {
        s @ Parse::Success(_, _) => s,
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(r, s) => match r {
            Some(r) => Parse::Success(r, s),
            None => Parse::Retreat(
                "not to use accept limit on a parser that isnt able to Some on a limit".to_string(),
            ),
        },
    }
}
