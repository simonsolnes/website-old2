#[derive(Debug, PartialEq, Eq)]
pub enum Parse<I, O> {
    Success(O, I),
    Retreat(String),
    Halt(String),
    Limit(Option<O>, I),
}

impl<I, O> Parse<I, O> {
    pub fn is_err(&self) -> bool {
        match self {
            Self::Success(_, _) => false,
            Parse::Retreat(_) => true,
            Parse::Halt(_) => true,
            Parse::Limit(_, _) => true,
        }
    }
}

// mod generic {
//     use std::{iter::Iterator, str::CharIndices};
//     pub trait IsIterable {
//         type Iterator;
//         fn into_iterable(self) -> Self::Iterator;
//         fn
//     }
//     impl IsIterable for &str {
//         type Iterator = CharIndices;
//         fn into_iterable(self) -> CharIndices {
//             self.char_indices()
//         }
//     }
// }

//     pub fn pop<I>(input: I) -> Parse<I, char>
//     where
//         I: IsIterable + Index<I>,
//     {
//         let mut iter = input.char_indices();
//         match iter.next() {
//             Some((_, c)) => {
//                 if let Some((next, _)) = iter.next() {
//                     Parse::Success(c, &input[next..])
//                 } else {
//                     Parse::Success(c, input.get_empty())
//                 }
//             }
//             None => Parse::Deficient(None),
//         }
//     }

// pub fn satisfy<I, O, P, F>(p: P, f: F) -> impl Fn(I) -> Parse<I, O>
// where P: Fn(I) ->
// {
//     |i: I| match p(i) {}
// }

pub mod str {

    use super::comb::result;
    use super::Parse;

    const ASCII_ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    pub fn alpha_char<'a>(i: &'a str) -> impl Fn(&'a str) -> Parse<&'a str, char> {
        result(
            pop,
            |c| match ASCII_ALPHABET.contains(c) {
                true => Ok(c),
                false => Err(()),
            },
            "is not in alphabet",
        )
    }

    pub fn pop(input: &str) -> Parse<&str, char> {
        let mut iter = input.char_indices();
        match iter.next() {
            Some((_, c)) => {
                if let Some((next, _)) = iter.next() {
                    Parse::Success(c, &input[next..])
                } else {
                    Parse::Success(c, "")
                }
            }
            None => Parse::Limit(None, input),
        }
    }

    pub fn peek_char(input: &str) -> Parse<&str, char> {
        match input.chars().next() {
            Some(c) => Parse::Success(c, input),
            None => Parse::Limit(None, input),
        }
    }

    pub fn other_than(chars: &'static str) -> impl Fn(&str) -> Parse<&str, &str> {
        move |input: &str| {
            let index = {
                let mut index = 0;
                let mut iter = input.char_indices();
                loop {
                    if let Some((i, c)) = iter.next() {
                        index = i;
                        if chars.contains(c) {
                            break;
                        }
                    } else {
                        return Parse::Limit(Some(&input[0..index]), &input[index..]);
                    }
                }

                index
            };
            match index {
                0 => Parse::Retreat("hur".to_string()),
                _ => {
                    let res = &input[0..index];
                    let sur = &input[index..];
                    Parse::Success(res, sur)
                }
            }
        }
    }

    pub fn take(num: usize) -> impl Fn(&str) -> Parse<&str, &str> {
        move |input: &str| {
            let mut index = 0;
            let mut count = 0;
            let mut iter = input.char_indices();
            loop {
                if let Some((i, _)) = iter.next() {
                    index = i;
                } else {
                    return Parse::Limit(Some(&input[..index]), &input[index..]);
                }
                if count == num {
                    break;
                }

                count += 1;
            }

            let res = &input[0..index];
            let sur = &input[index..];
            println!("take: num: {num}, res: {res}, sur: {sur}");
            Parse::Success(res, sur)
        }
    }

    pub fn take_while<F>(f: F) -> impl Fn(&str) -> Parse<&str, &str>
    where
        F: Fn(char) -> bool,
    {
        move |input: &str| {
            let mut index = 0;
            let mut iter = input.char_indices();
            loop {
                if let Some((i, c)) = iter.next() {
                    index = i;
                    if !f(c) {
                        break;
                    }
                } else {
                    return Parse::Limit(Some(&input[..index]), &input[index..]);
                }
            }
            let res = &input[0..index];
            let sur = &input[index..];
            Parse::Success(res, sur)
        }
    }

    pub fn take_some_while<F>(f: F) -> impl Fn(&str) -> Parse<&str, &str>
    where
        F: Fn(char) -> bool,
        F: Copy,
    {
        move |input: &str| {
            if let Some((i, c)) = input.char_indices().next() {
                if f(c) {
                    take_while(f)(&input[i..])
                } else {
                    Parse::Retreat("Take some requires at least one match".to_string())
                }
            } else {
                Parse::Limit(None, input)
            }
        }
    }

    pub fn literal(expected: &'static str) -> impl Fn(&str) -> Parse<&str, &str> {
        move |input: &str| match input.get(0..expected.len()) {
            Some(next) if next == expected => {
                let remaining = &input[expected.len()..];
                Parse::Success(expected, remaining)
            }
            _ => Parse::Retreat(format!("Expected '{}', found '{}'", expected, input)),
        }
    }

    pub fn char<'a>(char: char) -> impl Fn(&'a str) -> Parse<&str, char> {
        move |input: &str| -> Parse<&str, char> {
            match input.char_indices().next() {
                Some((i, c)) => match c == char {
                    true => {
                        // let consumed = input.get(..i + 1).unwrap();
                        let remaining = input.get(i + 1..).unwrap();
                        Parse::Success(char, remaining)
                    }
                    false => {
                        Parse::Retreat(format!("char didnt match for: {input}, expected {char}"))
                    }
                },
                None => Parse::Limit(None, input),
            }
        }
    }

    pub fn char_of(chars: &'static str) -> impl Fn(&str) -> Parse<&str, char> {
        move |i: &str| {
            result(
                pop,
                |c| match chars.contains(c) {
                    true => Ok(c),
                    false => Err(()),
                },
                "start with non-zero",
            )(i)
        }
    }

    pub fn some_chars_of(chars: &'static str) -> impl Fn(&str) -> Parse<&str, &str> {
        take_some_while(|c| chars.contains(c))
    }

    // pub fn take(amount: usize) -> impl Fn(&str) -> Parse<&str, &str
}

pub mod sequence {
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
                Parse::Limit(_, _) => Parse::Limit(None, i),
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
        I: Copy,
    {
        map(serial(p1, p2), |(r1, _)| r1)
    }

    pub fn preceded<I, O1, O2, P1, P2>(p1: P1, p2: P2) -> impl Fn(I) -> Parse<I, O2>
    where
        P1: Fn(I) -> Parse<I, O1>,
        P2: Fn(I) -> Parse<I, O2>,
        I: Copy,
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
        I: Copy,
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
        I: Copy,
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
        I: Copy,
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
        I: Copy,
    {
        map(
            serial(serial(p1, p2), serial(p3, p4)),
            |((r1, r2), (r3, r4))| (r1, r2, r3, r4),
        )
    }
}

pub mod repeat {

    use super::Parse;

    /// Applies a parser zero or more times
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
}

pub mod tools {
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
}

pub mod comb {
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
}
