# Website
The server hosting my website.

It is a work in progress, and my first project in Rust.

## HTTP-parser
The TCP read buffer is parsed and if more bytes are needed, the read buffer is appended to a larger buffer,
which is then parsed incrementally such that the large buffer will only contained unparsed bytes, and remain as small as possible.

This enables the TCP read buffer to be of any length
which in turn enables the HTTP request to have absurd sizes in the headers and the body.

This is a bit more work that just reading TCP into a buffer that will probably extend the header,
but it enables partially executing upon the request,
and subsequenty freeing data from the request before it's even finished parsing.
