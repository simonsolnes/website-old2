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
                return if num == count {
                    Parse::Success(input, "")
                } else {
                    Parse::Limit(Some(input), "")
                };
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
                return Parse::Limit(Some(input), "");
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
                false => Parse::Retreat(format!("char didnt match for: {input}, expected {char}")),
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
