# Website
The server hosting my website.

It is a work in progress, and my first project in Rust.

## HTTP-parser
The parser is an incremental parser,
meaning that the data can be acted upon during parsing and allows for absurd sizes in the HTTP header and body.

The TCP read buffer is parsed and if more bytes are needed, the read buffer is appended to a larger buffer,
which is then parsed incrementally such that the large buffer will only contain unparsed bytes, and remain as small as possible.

This is a bit more work that just reading TCP into a buffer that will probably extend the header,
but it enables the TCP read buffer to be of any length,
which in turn enables the HTTP request to have a size larger than the disk size,
given that some data can be dropped, or reduced, during fulfillment.
