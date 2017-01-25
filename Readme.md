# uhttp_uri -- HTTP URI parser

[Documentation](https://docs.rs/uhttp_uri)

This crate provides a barebone, zero-allocation parser for [HTTP
URIs](https://tools.ietf.org/html/rfc7230#section-2.7) as they appear in a request
header.

In general, components are extracted along defined delimiters, but further validation
and processing (such as percent decoding, query decoding, and punycode decoding) is
left to higher layers. In the pursuit of simplicity, this crate also contains no
support for generic and non-http URIs such as `file:` and `ftp://` – only the reduced
syntax for [`http://`](https://tools.ietf.org/html/rfc7230#section-2.7.1) and
[`https://`](https://tools.ietf.org/html/rfc7230#section-2.7.2) schemes is
implemented.

## Example

```rust
use uhttp_uri::{HttpUri, HttpResource, HttpScheme};

let uri = HttpUri::new("https://example.com:443/r/rarepuppers?k=v&v=k#top").unwrap();
assert_eq!(uri.scheme, HttpScheme::Https);
assert_eq!(uri.authority, "example.com:443");
assert_eq!(uri.resource.path, "/r/rarepuppers");
assert_eq!(uri.resource.query, Some("k=v&v=k"));
assert_eq!(uri.resource.fragment, Some("top"));

let res = HttpResource::new("/shittydogs?lang=en");
assert_eq!(res.path, "/shittydogs");
assert_eq!(res.query, Some("lang=en"));
assert_eq!(res.fragment, None);
```

## Usage

This [crate](https://crates.io/crates/uhttp_uri) can be used through cargo by adding
it as a dependency in `Cargo.toml`:

```toml
[dependencies]
uhttp_uri = "0.5.1"
```
and importing it in the crate root:

```rust
extern crate uhttp_uri;
```
