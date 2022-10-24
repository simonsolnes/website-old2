# Website
The server hosting my website.

It is a work in progress, and my first project in Rust.

## HTTP-parser
The parser is an incremental parser,
meaning that the data can be acted upon during parsing and allows for absurd sizes in the HTTP header and body.

The TCP read buffer is parsed and if more bytes are needed, the read buffer is appended to a larger buffer,
which is then parsed incrementally such that the large buffer will only contain unparsed bytes, and remain as small as possible.

This is a bit more work than just reading TCP into a buffer that probably extends the header size, but having a small read buffer plus a resizeable unparsed buffer has advantages.
 1. It doesn't allocate more memory than needed. A non-incremental parser would need to allocate a large buffer per request just to handle a few large requests. The alternative can be to extend the buffer if needed, but then the request has to be reparsed. If the next buffer is not large enough, the server can end up reparsing a large request several times until it finds the size.
 2. It enables the unparsed buffer to grow and shrink as it is parsed, which enables the HTTP request to have a size larger than the disk size, given that parts of the request can be dropped, or reduced, during fulfillment.

Is this mechanism need? Maybe in production, but not in this project. However, I sleep well at night knowing that my parser won't choke on a request that is larger than a pre-set buffer length.

## URL/IRI-parser
*Note:* The term URL and URI is being used interchangably here.
If you want to learn the difference between URI, URL, URN and IRI, I **strongly** advice you to **not** web seach it.
The web is littered with inaccurate, false and confusing explanations. Just go straight to the source; they explain it accuately and pretty well.
- [RFC 3986 URI](https://www.rfc-editor.org/rfc/rfc3986)
- [RFC 1738 URL](https://www.rfc-editor.org/rfc/rfc1738)
- [RFC 2141 URN](https://www.rfc-editor.org/rfc/rfc2141)

IRI is a replacement of URI, with support for unicode. The spesification is pretty simple; any non-ASCII unicode characters are valid.
This is because only ASCII characters can innterrupt the flow of URL parsing, such as `:`, `?`, `space` or `\\n`.

The URI spesification with its percent encoding, optional stuff and several edge-cases, was a lot more complicated to implement than the HTTP spec.
Using parser combinators made the process structured, than just trying to parse by hand.
IRIs are very easy to support. First decode as UTF-8 and then parse URL as a string rather than `[u8]`.

Since this requires seeking over the text twice, my first implementation skips the first step
and parses the URL as unicode codepoints and then do the UTF-8 decoding per sub-component. It's less comparisons, and it's cheaper to compare `u8`s than `u32`.
This works because UTF-8 is a superset of the ASCII character set.
ASCII is a 7 bit encoding, so if the first bit is `0` it's an ASCII character, and If the first bit is `1`, the URL parser can ignore it, and apply a UTF-8 decoder on the result.

It was unnecessarily cumbersome, and would result in not being able to parse URLs that are alerady encoded as unicode string.
Therefore the current parser does the UTF-8 decoding first, and then the URL parse with percent encoding.
