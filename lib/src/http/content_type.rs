use std::borrow::{Borrow, Cow};
use std::ops::Deref;
use std::str::FromStr;
use std::fmt;

use ext::IntoCollection;
use http::{Header, MediaType};
use http::hyper::mime::Mime;

/// Representation of HTTP Content-Types.
///
/// # Usage
///
/// `ContentTypes` should rarely be created directly. Instead, an associated
/// constant should be used; one is declared for most commonly used content
/// types.
///
/// ## Example
///
/// A Content-Type of `text/html; charset=utf-8` can be insantiated via the
/// `HTML` constant:
///
/// ```rust
/// use rocket::http::ContentType;
///
/// # #[allow(unused_variables)]
/// let html = ContentType::HTML;
/// ```
///
/// # Header
///
/// `ContentType` implements `Into<Header>`. As such, it can be used in any
/// context where an `Into<Header>` is expected:
///
/// ```rust
/// use rocket::http::ContentType;
/// use rocket::response::Response;
///
/// # #[allow(unused_variables)]
/// let response = Response::build().header(ContentType::HTML).finalize();
/// ```
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ContentType(pub MediaType);

macro_rules! content_types {
    ($($name:ident ($check:ident): $str:expr, $t:expr,
        $s:expr $(; $k:expr => $v:expr)*),+) => {
        $(
            #[doc="Media type for <b>"] #[doc=$str] #[doc="</b>: <i>"]
            #[doc=$t] #[doc="/"] #[doc=$s]
            $(#[doc="; "] #[doc=$k] #[doc=" = "] #[doc=$v])*
            #[doc="</i>"]
            #[allow(non_upper_case_globals)]
            pub const $name: ContentType = ContentType(MediaType::$name);

            #[doc="Returns `true` if `self` is the media type for <b>"]
            #[doc=$str]
            #[doc="</b>, "]
            /// without considering parameters.
            #[inline(always)]
            pub fn $check(&self) -> bool {
                *self == ContentType::$name
            }
         )+

        /// Returns `true` if this `ContentType` is known to Rocket, that is,
        /// there is an associated constant for `self`.
        pub fn is_known(&self) -> bool {
            $(if self.$check() { return true })+
            false
        }
    };
}

impl ContentType {
    /// Creates a new `ContentType` with top-level type `top` and subtype `sub`.
    /// This should _only_ be used to construct uncommon or custom content
    /// types. Use an associated constant for everything else.
    ///
    /// # Example
    ///
    /// Create a custom `application/x-person` content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let custom = ContentType::new("application", "x-person");
    /// assert_eq!(custom.top(), "application");
    /// assert_eq!(custom.sub(), "x-person");
    /// ```
    #[inline(always)]
    pub fn new<T, S>(top: T, sub: S) -> ContentType
        where T: Into<Cow<'static, str>>, S: Into<Cow<'static, str>>
    {
        ContentType(MediaType::new(top, sub))
    }

    /// Returns the Content-Type associated with the extension `ext`. Not all
    /// extensions are recognized. If an extensions is not recognized, then this
    /// method returns a ContentType of `Any`. The currently recognized
    /// extensions are: txt, html, htm, xml, js, css, json, png, gif, bmp, jpeg,
    /// jpg, and pdf.
    ///
    /// # Example
    ///
    /// A recognized content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let xml = ContentType::from_extension("xml");
    /// assert!(xml.is_xml());
    /// ```
    ///
    /// An unrecognized content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let foo = ContentType::from_extension("foo");
    /// assert!(foo.is_any());
    /// ```
    pub fn from_extension(ext: &str) -> ContentType {
        MediaType::from_extension(ext)
            .map(|mt| ContentType(mt))
            .unwrap_or(ContentType::Any)
    }

    /// Creates a new `ContentType` with top-level type `top`, subtype `sub`,
    /// and parameters `ps`. This should _only_ be used to construct uncommon or
    /// custom content types. Use an associated constant for everything else.
    ///
    /// # Example
    ///
    /// Create a custom `application/x-id; id=1` content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let id = ContentType::with_params("application", "x-id", ("id", "1"));
    /// assert_eq!(id.to_string(), "application/x-id; id=1".to_string());
    /// ```
    ///
    /// Create a custom `text/person; name=bob; weight=175` content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let params = vec![("name", "bob"), ("ref", "2382")];
    /// let mt = ContentType::with_params("text", "person", params);
    /// assert_eq!(mt.to_string(), "text/person; name=bob; ref=2382".to_string());
    /// ```
    #[inline]
    pub fn with_params<T, S, K, V, P>(top: T, sub: S, ps: P) -> ContentType
        where T: Into<Cow<'static, str>>, S: Into<Cow<'static, str>>,
              K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>,
              P: IntoCollection<(K, V)>
    {
        ContentType(MediaType::with_params(top, sub, ps))
    }

    #[inline(always)]
    pub fn media_type(&self) -> &MediaType {
        &self.0
    }

    #[inline(always)]
    pub fn into_media_type(self) -> MediaType {
        self.0
    }

    known_media_types!(content_types);
}

impl Default for ContentType {
    /// Returns a ContentType of `Any`, or `*/*`.
    #[inline(always)]
    fn default() -> ContentType {
        ContentType::Any
    }
}

impl Deref for ContentType {
    type Target = MediaType;

    #[inline(always)]
    fn deref(&self) -> &MediaType {
        &self.0
    }
}

#[doc(hidden)]
impl<T: Borrow<Mime>> From<T> for ContentType {
    #[inline(always)]
    default fn from(mime: T) -> ContentType {
        let mime: Mime = mime.borrow().clone();
        ContentType::from(mime)
    }
}

#[doc(hidden)]
impl From<Mime> for ContentType {
    #[inline]
    fn from(mime: Mime) -> ContentType {
        // soooo inneficient.
        let params = mime.2.into_iter()
            .map(|(attr, value)| (attr.to_string(), value.to_string()))
            .collect::<Vec<_>>();

        ContentType::with_params(mime.0.to_string(), mime.1.to_string(), params)
    }
}

impl FromStr for ContentType {
    type Err = String;

    /// Parses a `ContentType` from a given Content-Type header value.
    ///
    /// # Examples
    ///
    /// Parsing an `application/json`:
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
    ///
    /// let json = ContentType::from_str("application/json").unwrap();
    /// assert!(json.is_known());
    /// assert_eq!(json, ContentType::JSON);
    /// ```
    ///
    /// Parsing a content type extension:
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
    ///
    /// let custom = ContentType::from_str("application/x-custom").unwrap();
    /// assert!(!custom.is_known());
    /// assert_eq!(custom.top(), "application");
    /// assert_eq!(custom.sub(), "x-custom");
    /// ```
    ///
    /// Parsing an invalid Content-Type value:
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
    ///
    /// let custom = ContentType::from_str("application//x-custom");
    /// assert!(custom.is_err());
    /// ```
    #[inline(always)]
    fn from_str(raw: &str) -> Result<ContentType, String> {
        MediaType::from_str(raw).map(|mt| ContentType(mt))
    }
}

impl fmt::Display for ContentType {
    /// Formats the ContentType as an HTTP Content-Type value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let ct = format!("{}", ContentType::JSON);
    /// assert_eq!(ct, "application/json");
    /// ```
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Creates a new `Header` with name `Content-Type` and the value set to the
/// HTTP rendering of this Content-Type.
impl Into<Header<'static>> for ContentType {
    #[inline(always)]
    fn into(self) -> Header<'static> {
        Header::new("Content-Type", self.to_string())
    }
}
