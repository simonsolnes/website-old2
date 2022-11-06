use super::comb::map;
use super::Parse;

// Applies two parsers in sequence, of which both must succeed to gain `Parse::Success`
pub fn serial<I, O1, O2, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, (O1, O2)>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    I: Copy,
{
    move |i: I| match p1(i) {
        Parse::Success(res1, sur1) => match p2(sur1) {
            Parse::Success(res2, sur2) => Parse::Success((res1, res2), sur2),
            Parse::Retreat(r) => Parse::Retreat(r),
            Parse::Halt(h) => Parse::Halt(h),
            Parse::Limit(Some(res2), sur2) => Parse::Limit(Some((res1, res2)), sur2),
            Parse::Limit(None, _) => Parse::Limit(None, i),
        },
        Parse::Retreat(r) => Parse::Retreat(r),
        Parse::Halt(h) => Parse::Halt(h),
        Parse::Limit(_, _) => Parse::Limit(None, i),
    }
}

pub fn terminated<I, O1, O2, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, O1>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(serial(p1, p2), |(r1, _)| r1)
}

pub fn preceded<I, O1, O2, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, O2>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(serial(p1, p2), |(_, r2)| r2)
}

pub fn between<I, O1, O2, O3, P1, P2, P3>(
    before: P1,
    subject: P2,
    after: P3,
) -> impl Fn(I) -> Parse<I, O2>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    P3: Fn(I) -> Parse<I, O3>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(serial3(before, subject, after), |(_, res, _)| res)
}

pub fn around<I, O1, O2, O3, P1, P2, P3>(
    before: P1,
    separator: P2,
    after: P3,
) -> impl Fn(I) -> Parse<I, (O1, O3)>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    P3: Fn(I) -> Parse<I, O3>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(serial3(before, separator, after), |(b, _, a)| (b, a))
}

pub fn serial3<I, O1, O2, O3, P1, P2, P3>(
    p1: P1,
    p2: P2,
    p3: P3,
) -> impl Fn(I) -> Parse<I, (O1, O2, O3)>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    P3: Fn(I) -> Parse<I, O3>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(serial(serial(p1, p2), p3), |((r1, r2), r3)| (r1, r2, r3))
}
pub fn serial4<I, O1, O2, O3, O4, P1, P2, P3, P4>(
    p1: P1,
    p2: P2,
    p3: P3,
    p4: P4,
) -> impl Fn(I) -> Parse<I, (O1, O2, O3, O4)>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    P3: Fn(I) -> Parse<I, O3>,
    P4: Fn(I) -> Parse<I, O4>,
    I: Copy + std::fmt::Display + std::fmt::Debug,
{
    map(
        serial(serial(p1, p2), serial(p3, p4)),
        |((r1, r2), (r3, r4))| (r1, r2, r3, r4),
    )
}
