use super::{comb::map_option, Parse};

/// Applies a parser zero or more times and returns a vector with the results
pub fn repeat_any<I, O, P>(p: P) -> impl Fn(I) -> Parse<I, Vec<O>>
where
    P: Fn(I) -> Parse<I, O>,
    I: Copy,
{
    move |i: I| {
        let mut acc: Vec<O> = Vec::new();
        let mut remainder = i;
        loop {
            match p(remainder) {
                Parse::Success(res, sur) => {
                    remainder = sur;
                    acc.push(res);
                }
                Parse::Retreat(_) => break,
                Parse::Halt(h) => return Parse::Halt(h),
                Parse::Limit(res, sur) => {
                    if let Some(lr) = res {
                        acc.push(lr);
                    }
                    return Parse::Limit(Some(acc), sur);
                }
            }
        }
        Parse::Success(acc, remainder)
    }
}
/// Applies a parser one or more times and returns a vector with the results
/// Retreats if zero repetitions was found
pub fn repeat_some<I, O, P>(p: P) -> impl Fn(I) -> Parse<I, Vec<O>>
where
    P: Fn(I) -> Parse<I, O>,
    O: std::fmt::Debug,
    I: Copy,
{
    map_option(repeat_any(p), |res| {
        println!("res: {:?}", res);
        if res.len() > 0 {
            Some(res)
        } else {
            None
        }
    })
}

pub fn separated_items<I, O1, O2, P1, P2>(
    separator: P1,
    item: P2,
) -> impl Fn(I) -> Parse<I, Vec<O2>>
where
    P1: Fn(I) -> Parse<I, O1>,
    P2: Fn(I) -> Parse<I, O2>,
    I: Copy,
{
    move |i: I| {
        let mut list = Vec::new();
        let mut rest = i;
        let mut rest_unassumed = i;
        loop {
            match item(rest) {
                Parse::Success(item_res, sur) => {
                    list.push(item_res);
                    rest = sur;
                    rest_unassumed = sur;
                }
                Parse::Retreat(_) => return Parse::Success(list, rest_unassumed),
                Parse::Halt(_) => return Parse::Halt(format!("Halted at parsing item")),
                Parse::Limit(res, sur) => {
                    if let Some(r) = res {
                        list.push(r);
                    }
                    return Parse::Limit(Some(list), sur);
                }
            }
            match separator(rest) {
                Parse::Success(_, sur) => {
                    rest = sur;
                }
                Parse::Retreat(_) => return Parse::Success(list, rest),
                Parse::Halt(_) => return Parse::Halt(format!("Halted at parsing item")),
                Parse::Limit(_, _) => return Parse::Limit(Some(list), rest_unassumed),
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::super::*;
    #[test]
    fn test_repeat_any() {
        assert_eq!(
            repeat::repeat_any(str::char('a'))("aa"),
            Parse::Limit(Some(vec!['a', 'a']), "")
        );
        assert_eq!(
            repeat::repeat_any(comb::map_bool(str::take(2), |r| r.is_ascii()))("aabbåå"),
            Parse::Success(vec!["aa", "bb"], "åå")
        );
        assert_eq!(
            repeat::repeat_any(str::take(2))("aabbcc"),
            Parse::Limit(Some(vec!["aa", "bb", "cc"]), "")
        );
    }
    #[test]
    fn test_repeat_some() {
        assert_eq!(
            repeat::repeat_some(str::char('a'))("aa"),
            Parse::Limit(Some(vec!['a', 'a']), "")
        );
        assert_eq!(
            repeat::repeat_some(str::char('a'))("aab"),
            Parse::Success(vec!['a', 'a'], "b")
        );
        assert!(repeat::repeat_some(str::char('a'))("").is_err(),);
    }
}
