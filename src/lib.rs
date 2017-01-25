//! This crate provides a barebone, zero-allocation parser for [HTTP
//! URIs](https://tools.ietf.org/html/rfc7230#section-2.7) as they appear in a request
//! header.
//!
//! In general, components are extracted along defined delimiters, but further validation
//! and processing (such as percent decoding, query decoding, and punycode decoding) is
//! left to higher layers. In the pursuit of simplicity, this crate also contains no
//! support for generic and non-http URIs such as `file:` and `ftp://` – only the reduced
//! syntax for [`http://`](https://tools.ietf.org/html/rfc7230#section-2.7.1) and
//! [`https://`](https://tools.ietf.org/html/rfc7230#section-2.7.2) schemes is
//! implemented.
//!
//! ## Example
//!
//! ```rust
//! use uhttp_uri::{HttpUri, HttpResource, HttpScheme};
//!
//! let uri = HttpUri::new("https://example.com:443/r/rarepuppers?k=v&v=k#top").unwrap();
//! assert_eq!(uri.scheme, HttpScheme::Https);
//! assert_eq!(uri.authority, "example.com:443");
//! assert_eq!(uri.resource.path, "/r/rarepuppers");
//! assert_eq!(uri.resource.query, Some("k=v&v=k"));
//! assert_eq!(uri.resource.fragment, Some("top"));
//!
//! let res = HttpResource::new("/shittydogs?lang=en");
//! assert_eq!(res.path, "/shittydogs");
//! assert_eq!(res.query, Some("lang=en"));
//! assert_eq!(res.fragment, None);
//! ```

/// Components in an HTTP Request Line URI.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct HttpUri<'a> {
    /// HTTP scheme of request.
    ///
    /// This is automatically parsed to an `HttpScheme` since RFC7230 only gives syntax for
    /// the two http schemes.
    pub scheme: HttpScheme,

    /// Authority for the URI's target resource.
    ///
    /// This should typically be a domain name or IP address and may also contain a
    /// username and port.
    pub authority: &'a str,

    /// Path and parameters for requested target resource.
    pub resource: HttpResource<'a>,
}

impl<'a> HttpUri<'a> {
    /// Try to parse the given string into `HttpUri` components.
    ///
    /// The string must contain no whitespace, as required by
    /// [RFC7230§3.1.1](https://tools.ietf.org/html/rfc7230#section-3.1.1).
    pub fn new(s: &'a str) -> Result<Self, ()> {
        let (scheme, rest) = match s.find("://") {
            Some(idx) => s.split_at(idx),
            None => return Err(()),
        };

        let scheme = scheme.parse()?;
        let rest = &rest[3..];

        let (authority, rest) = match rest.find('/') {
            Some(idx) => rest.split_at(idx),
            None => (rest, ""),
        };

        if authority.is_empty() {
            return Err(());
        }

        Ok(HttpUri {
            scheme: scheme,
            authority: authority,
            resource: HttpResource::new(rest),
        })
    }

}

/// Writes the URI in the format required by [RFC7230§2.7.1]/[RFC7230§2.7.2].
impl<'a> std::fmt::Display for HttpUri<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}://{}{}", self.scheme, self.authority, self.resource)
    }
}

/// Components in an HTTP URI resource.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct HttpResource<'a> {
    /// Path to the resource.
    ///
    /// This is guaranteed to be nonempty (it will contain at least `"/"`.)
    pub path: &'a str,

    /// Parameters used to further identify the resource.
    pub query: Option<&'a str>,

    /// Identifier and parameters for a subresource.
    pub fragment: Option<&'a str>,
}

impl<'a> HttpResource<'a> {
    /// Parse the given string into a new `HttpResource`.
    pub fn new(s: &'a str) -> Self {
        let (path, query, fragment) = parts(s, s.find('?'), s.find('#'));

        HttpResource {
            path: if path.is_empty() {
                "/"
            } else {
                path
            },
            query: if query.is_empty() {
                None
            } else {
                Some(query)
            },
            fragment: if fragment.is_empty() {
                None
            } else {
                Some(fragment)
            }
        }
    }
}

impl<'a> std::fmt::Display for HttpResource<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        try!(fmt.write_str(self.path));

        if let Some(q) = self.query {
            try!(write!(fmt, "?{}", q));
        }

        if let Some(f) = self.fragment {
            try!(write!(fmt, "#{}", f));
        }

        Ok(())
    }
}

/// Split URI into path, query, and fragment [RFC3986§3].
fn parts<'a>(s: &'a str, qidx: Option<usize>, fidx: Option<usize>)
    -> (&'a str, &'a str, &'a str)
{
    match (qidx, fidx) {
        (Some(q), Some(f)) => if q < f {
            let (path, query) = (&s[..f]).split_at(q);
            let (_, frag) = s.split_at(f);

            (path, &query[1..], &frag[1..])
        } else {
            parts(s, None, Some(f))
        },
        (Some(q), None) => {
            let (path, query) = s.split_at(q);
            (path, &query[1..], "")
        },
        (None, Some(f)) => {
            let (path, frag) = s.split_at(f);
            (path, "", &frag[1..])
        },
        (None, None) => {
            (s, "", "")
        },
    }
}

/// Schemes for HTTP requests.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HttpScheme {
    /// Plaintext http.
    Http,
    /// Secure http.
    Https,
}

impl std::str::FromStr for HttpScheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(HttpScheme::Http),
            "https" => Ok(HttpScheme::Https),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for HttpScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match *self {
            HttpScheme::Http => "http",
            HttpScheme::Https => "https",
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_http_resource() {
        assert_eq!(HttpResource::new("/a/b/c"),
            HttpResource {
                path: "/a/b/c",
                query: None,
                fragment: None,
            });

        assert_eq!(HttpResource::new("/a/b/c?key=val"),
            HttpResource {
                path: "/a/b/c",
                query: Some("key=val"),
                fragment: None,
            });

        assert_eq!(HttpResource::new("/a/b/c#frag"),
            HttpResource {
                path: "/a/b/c",
                query: None,
                fragment: Some("frag"),
            });

        assert_eq!(HttpResource::new("/a/b/c#frag?frag-param"),
            HttpResource {
                path: "/a/b/c",
                query: None,
                fragment: Some("frag?frag-param"),
            });

        assert_eq!(HttpResource::new("/a/b/c?key=val&param#frag"),
            HttpResource {
                path: "/a/b/c",
                query: Some("key=val&param"),
                fragment: Some("frag"),
            });

        assert_eq!(HttpResource::new("/a/b/c/?key=val&param#frag"),
            HttpResource {
                path: "/a/b/c/",
                query: Some("key=val&param"),
                fragment: Some("frag"),
            });

        assert_eq!(HttpResource::new("/a/b/c?key=d/e#frag/ment?param"),
            HttpResource {
                path: "/a/b/c",
                query: Some("key=d/e"),
                fragment: Some("frag/ment?param"),
            });

        assert_eq!(HttpResource::new("/a/b/c#frag?param&key=val"),
            HttpResource {
                path: "/a/b/c",
                query: None,
                fragment: Some("frag?param&key=val"),
            });

        assert_eq!(HttpResource::new("/%02/%03/%04#frag"),
            HttpResource {
                path: "/%02/%03/%04",
                query: None,
                fragment: Some("frag"),
            });

        assert_eq!(HttpResource::new("/"),
            HttpResource {
                path: "/",
                query: None,
                fragment: None,
            });

        assert_eq!(HttpResource::new(""),
            HttpResource {
                path: "/",
                query: None,
                fragment: None,
            });

        assert_eq!(HttpResource::new("?#"),
            HttpResource {
                path: "/",
                query: None,
                fragment: None,
            });

        assert_eq!(HttpResource::new("?key=val#"),
            HttpResource {
                path: "/",
                query: Some("key=val"),
                fragment: None,
            });

        assert_eq!(HttpResource::new("?#frag"),
            HttpResource {
                path: "/",
                query: None,
                fragment: Some("frag"),
            });
    }

    #[test]
    fn test_http_uri() {
        assert_eq!(HttpUri::new("http://example.com").unwrap(),
            HttpUri {
                scheme: HttpScheme::Http,
                authority: "example.com",
                resource: HttpResource {
                    path: "/",
                    query: None,
                    fragment: None,
                }
            });

        assert_eq!(HttpUri::new("http://127.0.0.1:61761/chunks").unwrap(),
            HttpUri {
                scheme: HttpScheme::Http,
                authority: "127.0.0.1:61761",
                resource: HttpResource {
                    path: "/chunks",
                    query: None,
                    fragment: None,
                }
            });

        assert_eq!(HttpUri::new("https://127.0.0.1:61761").unwrap(),
            HttpUri {
                scheme: HttpScheme::Https,
                authority:  "127.0.0.1:61761",
                resource: HttpResource {
                    path: "/",
                    query: None,
                    fragment: None,
                }
            });

        assert!(HttpUri::new("http://").is_err());
        assert!(HttpUri::new("http:///").is_err());
        assert!(HttpUri::new("://example.com").is_err());
        assert!(HttpUri::new("ftp://example.com").is_err());
        assert!(HttpUri::new("file:example").is_err());
        assert!(HttpUri::new("htt:p//host").is_err());
        assert!(HttpUri::new("hyper.rs/").is_err());
        assert!(HttpUri::new("hyper.rs?key=val").is_err());

        assert_eq!(HttpUri::new("http://test.com/nazghul?test=3").unwrap(),
            HttpUri {
                scheme: HttpScheme::Http,
                authority: "test.com",
                resource: HttpResource {
                    path: "/nazghul",
                    query: Some("test=3"),
                    fragment: None,
                }
            });
    }
}
