pub mod comb;
pub mod repeat;
pub mod sequence;
pub mod str;
pub mod tools;

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
