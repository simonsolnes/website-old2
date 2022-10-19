# Website
The server hosting my website.

It is a work in progress, and my first project in Rust.

## HTTP-parser
The parser is an incremental parser,
meaning that the data can be acted upon during parsing and allows for absurd sizes in the HTTP header and body.

The TCP read buffer is parsed and if more bytes are needed, the read buffer is appended to a larger buffer,
which is then parsed incrementally such that the large buffer will only contain unparsed bytes, and remain as small as possible.

This is a bit more work than just reading TCP into a buffer that probably extends the header size, but having a small read buffer plus a resizeable unparsed buffer has advantages.
 1. It doesn't allocate more memory than needed. A non-incremental parser would need to allocate a large buffer per request just to handle a few large requests. The alternative can be to extend the buffer if needed, but then you have to reparse the request.
 2. It enables the unparsed buffer to grow and shrink as it is parsed, which enables the HTTP request to have a size larger than the disk size, given that some data can be dropped, or reduced, during fulfillment,

Do I utilize this mechanism? No. But I sleep well at night knowing that my parser won't choke on a request that is larger than a pre-set buffer length.
