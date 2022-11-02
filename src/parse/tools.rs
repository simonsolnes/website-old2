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
pub fn halt<'l, I, O, P>(label: &'l str, p: P) -> impl Fn(I) -> Parse<I, O> + 'l
where
    P: Fn(I) -> Parse<I, O> + 'l,
    I: Display + Copy,
{
    move |i: I| match p(i) {
        s @ Parse::Success(_, _) => s,
        Parse::Retreat(r) => Parse::Halt(format!("Halted {label}, for {i} ({r})")),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(r, s) => Parse::Limit(r, s),
    }
}
