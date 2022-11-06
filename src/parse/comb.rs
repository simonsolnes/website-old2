use std::fmt::Debug;
use std::fmt::Display;

use super::Parse;

// pub fn not<P, I, O>(p: P) -> impl Fn(I) -> Parse<I, ()>
// where
//     P: Fn(I) -> Parse<I, O>,
//     I: Copy,
// {
//     move |i: I| match p(i) {
//         Parse::Success(_, _) => Parse::Retreat("Negative was found".to_string()),
//         Parse::Retreat(_) => Parse::Success((), i),
//         Parse::Halt(_) => Parse::Success((), i),
//         Parse::Deficient(d) => Parse::Deficient(d),
//     }
// }

pub fn optional<I, O, P>(p: P) -> impl Fn(I) -> Parse<I, Option<O>>
where
    P: Fn(I) -> Parse<I, O>,
    I: Copy,
{
    move |i: I| match p(i) {
        Parse::Success(res, sur) => Parse::Success(Some(res), sur),
        Parse::Retreat(_) => Parse::Success(None, i),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(r, s) => Parse::Limit(Some(r), s),
    }
}

/// Returns the parse of the first parser that succeeds
/// If a parser limits out, it will try others as well
pub fn either_of<I, O, P>(parsers: &[P]) -> impl Fn(I) -> Parse<I, O> + '_
where
    P: Fn(I) -> Parse<I, O>,
    I: Copy,
{
    move |input: I| {
        let mut limit: Option<(Option<O>, I)> = None;
        for parser in parsers.iter() {
            match parser(input) {
                s @ Parse::Success(_, _) => return s,
                Parse::Retreat(_) => continue,
                h @ Parse::Halt(_) => return h,
                Parse::Limit(r, s) => {
                    limit = Some((r, s));
                    continue;
                }
            }
        }
        if let Some((r, s)) = limit {
            Parse::Limit(r, s)
        } else {
            Parse::Retreat("Not any of the anys".to_string())
        }
    }
}
pub fn either<I, O, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, O>
where
    P1: Fn(I) -> Parse<I, O>,
    P2: Fn(I) -> Parse<I, O>,
    I: Copy + PartialOrd + Display,
    O: Debug,
{
    move |i: I| match p1(i) {
        Parse::Retreat(_) => p2(i),
        Parse::Limit(None, _) => match p2(i) {
            Parse::Success(r, s) => Parse::Limit(Some(r), s),
            a @ _ => a,
        },
        a @ _ => a,
    }
}
pub fn either3<I, O, P1, P2, P3>(p1: P1, p2: P2, p3: P3) -> impl Fn(I) -> Parse<I, O>
where
    P1: Fn(I) -> Parse<I, O>,
    P2: Fn(I) -> Parse<I, O>,
    P3: Fn(I) -> Parse<I, O>,
    I: Copy + PartialOrd + Display,
    O: Debug,
{
    either(either(p1, p2), p3)
}
pub fn either4<I, O, P1, P2, P3, P4>(p1: P1, p2: P2, p3: P3, p4: P4) -> impl Fn(I) -> Parse<I, O>
where
    P1: Fn(I) -> Parse<I, O>,
    P2: Fn(I) -> Parse<I, O>,
    P3: Fn(I) -> Parse<I, O>,
    P4: Fn(I) -> Parse<I, O>,
    I: Copy + PartialOrd + Display,
    O: Debug,
{
    either(either(p1, p2), either(p3, p4))
}
pub fn either5<I, O, P1, P2, P3, P4, P5>(
    p1: P1,
    p2: P2,
    p3: P3,
    p4: P4,
    p5: P5,
) -> impl Fn(I) -> Parse<I, O>
where
    P1: Fn(I) -> Parse<I, O>,
    P2: Fn(I) -> Parse<I, O>,
    P3: Fn(I) -> Parse<I, O>,
    P4: Fn(I) -> Parse<I, O>,
    P5: Fn(I) -> Parse<I, O>,
    I: Copy + PartialOrd + Display,
    O: Debug,
{
    either(either(p1, p2), either3(p3, p4, p5))
}

// Returns a parser which pipes the output from the first parser to the second parser
pub fn pipe<I, O1, O2, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, O2>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(O1) -> Parse<I, O2>,
    I: Copy,
{
    //TODO what should the surplus be
    move |i: I| match p1(i) {
        Parse::Success(res1, _) => match p2(res1) {
            Parse::Success(res2, sur2) => Parse::Success(res2, sur2),
            Parse::Retreat(r) => Parse::Retreat(r),
            Parse::Halt(h) => Parse::Halt(h),
            Parse::Limit(r, s) => Parse::Limit(r, s),
        },
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(_, _) => Parse::Limit(None, i),
    }
}

/// Returns a parser, which applies `func` to the `parser` result
pub fn map<I, O, M, P, F>(parser: P, func: F) -> impl Fn(I) -> Parse<I, M>
where
    P: Fn(I) -> Parse<I, O>,
    F: Fn(O) -> M,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => Parse::Success(func(res), sur),
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(Some(r), s) => Parse::Limit(Some(func(r)), s),
        Parse::Limit(None, _) => Parse::Limit(None, i),
    }
}

pub fn map_result<'l, I, O, P, F, M, E>(
    parser: P,
    func: F,
    label: &'l str,
) -> impl Fn(I) -> Parse<I, M> + '_
where
    P: Fn(I) -> Parse<I, O> + 'l,
    F: Fn(O) -> Result<M, E> + 'l,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => match func(res) {
            Ok(m) => Parse::Success(m, sur),
            Err(_) => Parse::Retreat(format!("Result error {}", label)),
        },
        Parse::Limit(Some(res), sur) => match func(res) {
            Ok(m) => Parse::Limit(Some(m), sur),
            Err(_) => Parse::Limit(None, sur),
        },
        Parse::Limit(None, _) => Parse::Limit(None, i),
        Parse::Retreat(_) => Parse::Retreat("Result error".to_string()),
        Parse::Halt(h) => Parse::Halt(h),
    }
}

pub fn map_result_halts<'l, I, O, P, F, M, E>(
    parser: P,
    func: F,
    label: &'l str,
) -> impl Fn(I) -> Parse<I, M> + '_
where
    P: Fn(I) -> Parse<I, O> + 'l,
    F: Fn(O) -> Result<M, E> + 'l,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => match func(res) {
            Ok(m) => Parse::Success(m, sur),
            Err(_) => Parse::Halt(format!("Result error {}", label)),
        },
        Parse::Limit(Some(res), sur) => match func(res) {
            Ok(m) => Parse::Limit(Some(m), sur),
            Err(_) => Parse::Limit(None, i),
        },
        Parse::Retreat(_) => Parse::Retreat("Result error".to_string()),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(None, _) => Parse::Limit(None, i),
    }
}

pub fn map_option<I, O, P, F, M>(parser: P, func: F) -> impl Fn(I) -> Parse<I, M>
where
    P: Fn(I) -> Parse<I, O>,
    F: Fn(O) -> Option<M>,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => match func(res) {
            Some(m) => Parse::Success(m, sur),
            None => Parse::Retreat(format!("Map option error")),
        },
        Parse::Retreat(_) => Parse::Retreat("Result error".to_string()),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(Some(res), sur) => match func(res) {
            Some(m) => Parse::Limit(Some(m), sur),
            None => Parse::Limit(None, i),
        },
        Parse::Limit(None, _) => Parse::Limit(None, i),
    }
}

pub fn map_bool<I, O, P, F>(parser: P, func: F) -> impl Fn(I) -> Parse<I, O>
where
    P: Fn(I) -> Parse<I, O>,
    F: Fn(&O) -> bool,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => match func(&res) {
            true => Parse::Success(res, sur),
            false => Parse::Retreat(format!("Map bool error")),
        },
        Parse::Retreat(_) => Parse::Retreat("Result error".to_string()),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(Some(r), s) => match func(&r) {
            true => Parse::Limit(Some(r), s),
            false => Parse::Limit(None, i),
        },
        Parse::Limit(None, _) => Parse::Limit(None, i),
    }
}

pub fn ret<I, O, P, M>(p: P, x: M) -> impl Fn(I) -> Parse<I, M>
where
    P: Fn(I) -> Parse<I, O>,
    M: Copy,
    I: Copy,
{
    move |i: I| match p(i) {
        Parse::Success(_, sur) => Parse::Success(x, sur),
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(Some(_), s) => Parse::Limit(Some(x), s),
        Parse::Limit(None, _) => Parse::Limit(None, i),
    }
}

// If parser succeeds,
// pub fn resolve<P>(parser: P, value: V) {
// }

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test_either() {
        use self::str::{literal, some_chars_of};
        assert_eq!(
            either(literal("hei"), literal("hallo"))("hei"),
            Parse::Success("hei", "")
        );
        assert_eq!(
            either(literal("hei"), literal("hallo"))("hallo"),
            Parse::Success("hallo", "")
        );
        assert_eq!(
            either(some_chars_of("ab"), literal("hallo"))("hallo"),
            Parse::Success("hallo", "")
        );
        assert_eq!(
            either(some_chars_of("ab"), literal("hallo"))("ababs"),
            Parse::Success("abab", "s")
        );
        assert_eq!(
            either(literal("hallo"), some_chars_of("ab"))("ababs"),
            Parse::Success("abab", "s")
        );
        assert_eq!(
            either(some_chars_of("ab"), literal("hallo"))("abab"),
            Parse::Limit(Some("abab"), "")
        );
        assert_eq!(
            either(literal("hallo"), some_chars_of("ab"))("abab"),
            Parse::Limit(Some("abab"), "")
        );
    }
}
