
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
    I: Copy,
{
    move |i: I| match p1(i) {
        s @ Parse::Success(_, _) => s,
        Parse::Retreat(_) => p2(i),
        h @ Parse::Halt(_) => h,
        Parse::Limit(r, s) => Parse::Limit(r, s),
    }
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
        Parse::Limit(r, s) => Parse::Limit(None, i),
    }
}

pub fn result<'l, I, O, P, F, M, E>(
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
        Parse::Retreat(_) => Parse::Retreat("Result error".to_string()),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(_, _) => Parse::Limit(None, i),
    }
}

pub fn result_halt<'a, I, O, P, F, M>(
    parser: P,
    func: F,
    label: &'a str,
) -> impl Fn(I) -> Parse<I, M> + 'a
where
    P: Fn(I) -> Parse<I, O> + 'a,
    F: Fn(O) -> Result<M, ()> + 'a,
    O: std::fmt::Display + Copy,
    I: Copy,
{
    move |i: I| match parser(i) {
        Parse::Success(res, sur) => match func(res) {
            Ok(m) => Parse::Success(m, sur),
            Err(_) => Parse::Halt(format!("Illegal {}: {}", label, res)),
        },
        Parse::Retreat(r) => Parse::Retreat(format!("retreat from {r}")),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(_, _) => Parse::Limit(None, i),
    }
}

pub fn ret<I, O: Copy>(x: O) -> impl Fn(I) -> Parse<I, O> {
    move |i: I| Parse::Success(x, i)
}

// If parser succeeds,
// pub fn resolve<P>(parser: P, value: V) {
// }
