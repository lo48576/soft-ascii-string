use std::borrow::{Cow, Borrow};
use std::ops::{Deref, DerefMut, AddAssign, Add};
use std::cmp::PartialEq;
use std::iter::{IntoIterator, FromIterator, Extend};
use std::ops::{
    Index, IndexMut,
    Range, RangeFrom,
    RangeTo, RangeFull,
};
use std::path::Path;
use std::ffi::OsStr;
use std::net::{ToSocketAddrs, SocketAddr};
use std::fmt::{self, Display};
use std::{io, vec};
use std::str::FromStr;

// this import will become unused in future rust versions
// but won't be removed for now for supporting current
// rust versions
#[allow(warnings)]
use std::ascii::AsciiExt;

use error::{StringFromStrError, FromSourceError};
use soft_char::SoftAsciiChar;
use soft_str::SoftAsciiStr;

/// a `String` wrapper with an additional "is us-ascii" soft constraint
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SoftAsciiString(String);

impl SoftAsciiString {

    #[inline(always)]
    pub fn from_unchecked<S: Into<String>>(s: S) -> Self {
        SoftAsciiString(s.into())
    }

    #[inline(always)]
    #[deprecated(since="1.0.0", note="use from_unchecked")]
    pub fn from_string_unchecked<S: Into<String>>(s: S) -> Self {
        SoftAsciiString::from_unchecked(s)
    }

    pub fn from_string<S>(source: S) -> Result<Self, FromSourceError<S>>
        where S: fmt::Debug + AsRef<str> + Into<String>
    {
        if source.as_ref().is_ascii() {
            Ok(Self::from_unchecked(source))
        } else {
            Err(FromSourceError::new(source))
        }
    }

    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        SoftAsciiString(String::with_capacity(cap))
    }


    pub fn revalidate_soft_constraint(self) -> Result<SoftAsciiString, String> {
        if self.is_ascii() {
            Ok(self)
        } else {
            Err(self.0)
        }
    }

    //TODO warn in doc
    #[inline]
    pub fn inner_string_mut(&mut self) -> &mut String {
        &mut self.0
    }

    #[inline]
    pub fn inner_string(&self) -> &String {
        &self.0
    }

    #[inline]
    pub fn push_str(&mut self, other: &SoftAsciiStr) {
        self.0.push_str(other.as_str())
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    #[inline]
    pub fn push(&mut self, ch: SoftAsciiChar) {
        self.0.push(ch.into())
    }

    pub fn pop(&mut self) -> Option<SoftAsciiChar> {
        self.0.pop()
            .map(SoftAsciiChar::from_unchecked)
    }

    pub fn remove(&mut self, idx: usize) -> SoftAsciiChar {
        SoftAsciiChar::from_unchecked(self.0.remove(idx))
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, ch: SoftAsciiChar) {
        self.0.insert(idx, ch.into())
    }

    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &SoftAsciiStr) {
        self.0.insert_str(idx, string.as_str())
    }

    #[inline]
    pub fn as_soft_ascii_str(&self) -> &SoftAsciiStr {
        SoftAsciiStr::from_unchecked(self.0.as_str())
    }

    #[inline]
    pub fn as_soft_ascii_str_mut(&mut self) -> &mut SoftAsciiStr {
        SoftAsciiStr::from_unchecked_mut(self.0.as_mut_str())
    }

    #[inline]
    pub fn split_off(&mut self, at: usize) -> SoftAsciiString {
        SoftAsciiString::from_unchecked(self.0.split_off(at))
    }

    #[inline]
    pub fn into_boxed_soft_ascii_str(self) -> Box<SoftAsciiStr> {
        SoftAsciiStr::from_boxed_str(self.into_boxed_str())
    }

    #[inline]
    pub fn into_boxed_str(self) -> Box<str> {
        self.0.into_boxed_str()
    }

    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.0.is_ascii()
    }

}

macro_rules! impl_wrapping {
    (pub > $(fn $name:ident(&self$(, $param:ident: $tp:ty)*) -> $ret:ty),*) => (
        impl SoftAsciiString {$(
            #[inline]
            pub fn $name(&self $(, $param: $tp)*) -> $ret {
                String::$name(&self.0 $(, $param)*)
            }
        )*}
    )
}

impl_wrapping! {
    pub >
    fn as_bytes(&self) -> &[u8],
    fn capacity(&self) -> usize,
    fn len(&self) -> usize,
    fn as_str(&self) -> &str,
    fn is_empty(&self) -> bool
}

macro_rules! impl_wrapping_mut {
    (pub > $(fn $name:ident(&mut self$(, $param:ident: $tp:ty)*) -> $ret:ty),*) => (
        impl SoftAsciiString {$(
            #[inline]
            pub fn $name(&mut self $(, $param: $tp)*) -> $ret {
                String::$name(&mut self.0 $(, $param)*)
            }
        )*}
    )
}

impl_wrapping_mut! {
    pub >
    fn reserve(&mut self, additional: usize) -> (),
    fn reserve_exact(&mut self, additional: usize) -> (),
    fn shrink_to_fit(&mut self) -> (),
    fn truncate(&mut self, new_len: usize) -> (),
    fn clear(&mut self) -> ()
}

impl Borrow<str> for SoftAsciiString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<SoftAsciiStr> for SoftAsciiString {
    fn borrow(&self) -> &SoftAsciiStr {
        &*self
    }
}

impl Deref for SoftAsciiString {
    type Target = SoftAsciiStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_soft_ascii_str()
    }
}

impl DerefMut for SoftAsciiString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_soft_ascii_str_mut()
    }
}


impl<'a> AddAssign<&'a SoftAsciiStr> for SoftAsciiString {
    fn add_assign(&mut self,  other: &'a SoftAsciiStr) {
        self.push_str(other)
    }
}
impl<'a> Add<&'a SoftAsciiStr> for SoftAsciiString {
    type Output = Self;

    fn add(mut self, other: &'a SoftAsciiStr) -> Self {
        self.push_str(other);
        self
    }
}

impl PartialEq<SoftAsciiString> for str {
    fn eq(&self, other: &SoftAsciiString) -> bool {
        self == other.as_str()
    }
}

impl<'a> PartialEq<&'a str> for SoftAsciiString {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> PartialEq<SoftAsciiString> for &'a str {
    fn eq(&self, other: &SoftAsciiString) -> bool {
        other.as_str() == *self
    }
}

impl PartialEq<str> for SoftAsciiString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}


impl PartialEq<String> for SoftAsciiString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<SoftAsciiString> for String {
    fn eq(&self, other: &SoftAsciiString) -> bool {
        self == other.as_str()
    }
}

impl<'a> PartialEq<&'a SoftAsciiStr> for SoftAsciiString {
    #[inline]
    fn eq(&self, other: &&'a SoftAsciiStr) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> PartialEq<SoftAsciiString> for Cow<'a, str> {
    #[inline]
    fn eq(&self, other: &SoftAsciiString) -> bool {
        other.as_str() == &*self
    }
}

impl<'a> PartialEq<Cow<'a, str>> for SoftAsciiString {
    #[inline]
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.as_str() == &*other
    }
}

impl<'a> PartialEq<SoftAsciiString> for Cow<'a, SoftAsciiStr> {
    #[inline]
    fn eq(&self, other: &SoftAsciiString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> PartialEq<Cow<'a, SoftAsciiStr>> for SoftAsciiString {
    #[inline]
    fn eq(&self, other: &Cow<'a, SoftAsciiStr>) -> bool {
        self.as_str() == other.as_str()
    }
}


impl FromIterator<SoftAsciiChar> for SoftAsciiString {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=SoftAsciiChar>
    {
        let mut buf = SoftAsciiString::new();
        buf.extend(iter);
        buf
    }
}

impl<'a> FromIterator<&'a SoftAsciiChar> for SoftAsciiString {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=&'a SoftAsciiChar>
    {
        let mut buf = SoftAsciiString::new();
        buf.extend(iter);
        buf
    }
}

impl FromIterator<SoftAsciiString> for SoftAsciiString {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=SoftAsciiString>
    {
        let mut buf = SoftAsciiString::new();
        buf.extend(iter);
        buf
    }
}

impl<'a> FromIterator<Cow<'a, SoftAsciiStr>> for SoftAsciiString {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=Cow<'a, SoftAsciiStr>>
    {
        let mut buf = SoftAsciiString::new();
        buf.extend(iter);
        buf
    }
}

impl<'a> FromIterator<&'a SoftAsciiStr> for SoftAsciiString {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=&'a SoftAsciiStr>
    {
        let mut buf = SoftAsciiString::new();
        buf.extend(iter);
        buf
    }
}

impl AsRef<SoftAsciiStr> for SoftAsciiString {
    #[inline]
    fn as_ref(&self) -> &SoftAsciiStr {
        &*self
    }
}

impl AsRef<str> for SoftAsciiString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for SoftAsciiString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<OsStr> for SoftAsciiString {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for SoftAsciiString {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl ToSocketAddrs for SoftAsciiString {
    type Iter = vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        self.as_str().to_socket_addrs()
    }
}

macro_rules! impl_index {
    ($($idx:ty),*) => ($(
        impl Index<$idx> for SoftAsciiString {
            type Output = SoftAsciiStr;
            fn index(&self, index: $idx) -> &Self::Output {
                SoftAsciiStr::from_unchecked(self.0.index(index))
            }
        }
    )*);
}

impl_index! {
    Range<usize>,
    RangeFrom<usize>,
    RangeTo<usize>,
    RangeFull
}

macro_rules! impl_index_mut {
    ($($idx:ty),*) => ($(
        impl IndexMut<$idx> for SoftAsciiString {
            fn index_mut(&mut self, index: $idx) -> &mut Self::Output {
                SoftAsciiStr::from_unchecked_mut(self.0.index_mut(index))
            }
        }
    )*);
}

impl_index_mut! {
    Range<usize>,
    RangeFrom<usize>,
    RangeTo<usize>,
    RangeFull
}

impl Extend<SoftAsciiChar> for SoftAsciiString {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=SoftAsciiChar>
    {
        let iterator = iter.into_iter();
        let (min, _max) = iterator.size_hint();
        self.reserve(min);
        for ch in iterator {
            self.push(ch)
        }
    }
}

impl<'a> Extend<&'a SoftAsciiChar> for SoftAsciiString {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=&'a SoftAsciiChar>
    {
        self.extend(iter.into_iter().cloned())
    }
}

impl Extend<SoftAsciiString> for SoftAsciiString {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=SoftAsciiString>
    {
        for string in iter {
            self.push_str(&*string);
        }
    }
}

impl<'a> Extend<&'a SoftAsciiStr> for SoftAsciiString {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=&'a SoftAsciiStr>
    {
        for str in iter {
            self.push_str(str);
        }
    }
}

impl<'a> Extend<Cow<'a, SoftAsciiStr>> for SoftAsciiString {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=Cow<'a, SoftAsciiStr>>
    {
        for cow in iter {
            self.push_str(&cow)
        }
    }
}




impl<'a> From<Cow<'a, SoftAsciiStr>> for SoftAsciiString {
    fn from(cow: Cow<'a, SoftAsciiStr>) -> Self {
        match cow {
            Cow::Owned(s) => s,
            Cow::Borrowed(b) => b.to_owned()
        }
    }
}

impl<'a> From<&'a SoftAsciiStr> for SoftAsciiString {
    #[inline]
    fn from(s: &'a SoftAsciiStr) -> Self {
        s.to_owned()
    }
}

impl From<Box<SoftAsciiStr>> for SoftAsciiString {
    #[inline]
    fn from(b: Box<SoftAsciiStr>) -> SoftAsciiString {
        SoftAsciiStr::into_soft_ascii_string(b)
    }
}

impl Display for SoftAsciiString {
    #[inline]
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fter)
    }
}

impl Into<Vec<u8>> for SoftAsciiString {

    #[inline]
    fn into(self) -> Vec<u8> {
        self.0.into()
    }
}

impl Into<String> for SoftAsciiString {
    #[inline]
    fn into(self) -> String {
        self.0
    }
}

impl FromStr for SoftAsciiString {
    type Err = StringFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_ascii() {
            Ok(SoftAsciiString(s.to_owned()))
        } else {
            Err(StringFromStrError)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::borrow::Borrow;
    use ::SoftAsciiStr;

    const SOME_NOT_ASCII: &str = "malformed←";
    const SOME_ASCII: &str = "hy there";

    //FIXME use impl Trait instead
    fn borrow_untyped<T: ?Sized, B: Borrow<T>>(b: &B) -> &B { b }
    fn as_ref_untype<T: ?Sized, B: AsRef<T>>(b: &B) -> &B { b }

    mod SoftAsciiString {
        #![allow(non_snake_case)]
        use super::*;
        use super::super::SoftAsciiString;

        #[test]
        fn from_unchecked() {
            let sas = SoftAsciiString::from_unchecked(SOME_ASCII);
            assert_eq!(&*sas, SOME_ASCII);
            let sas = SoftAsciiString::from_unchecked(SOME_NOT_ASCII);
            assert_eq!(&*sas, SOME_NOT_ASCII);
        }

        #[test]
        fn from_string() {
            let sas: SoftAsciiString = assert_ok!(SoftAsciiString::from_string(SOME_ASCII));
            assert_eq!(&*sas, SOME_ASCII);
                        let sas: SoftAsciiString = assert_ok!(SoftAsciiString::from_string(SOME_ASCII.to_owned()));
            assert_eq!(&*sas, SOME_ASCII);
            let failed: String =
                assert_err!(SoftAsciiString::from_string(SOME_NOT_ASCII.to_owned())).into_source();
            assert_eq!(&*failed, SOME_NOT_ASCII);
        }

        #[test]
        fn borrow_str() {
            let sas = SoftAsciiString::from_string(SOME_ASCII);
            let sas = assert_ok!(sas);

            assert_eq!(
                borrow_untyped::<str, _>(&sas),
                SOME_ASCII
            );
        }

        #[test]
        fn as_ref_str() {
            let sas = SoftAsciiString::from_string(SOME_ASCII);
            let sas = assert_ok!(sas);

            assert_eq!(
                as_ref_untype::<str, _>(&sas),
                SOME_ASCII
            );
        }

        #[test]
        fn buffer() {
            let mut sas = assert_ok!(SoftAsciiString::from_string(SOME_ASCII));
            {
                let b: &String = sas.inner_string();
                assert_eq!(b, &String::from(SOME_ASCII));
            }
            {
                let b: &mut String = sas.inner_string_mut();
                assert_eq!(b, &mut String::from(SOME_ASCII));
            }
        }

        #[test]
        fn revalidate_soft_constraint() {
            let sas: SoftAsciiString =
                SoftAsciiString::from_unchecked(SOME_ASCII);
            assert_ok!(sas.revalidate_soft_constraint());

            let bad: SoftAsciiString =
                SoftAsciiString::from_unchecked(SOME_NOT_ASCII);
            assert_err!(bad.revalidate_soft_constraint());
        }

        #[test]
        fn has_into_vec_u8() {
            let sas = SoftAsciiString::from_unchecked("test");
            let v: Vec<u8> = sas.into();
            assert_eq!(v.as_slice(), b"test" as &[u8]);
        }

        #[test]
        fn has_into_string() {
            let sas = SoftAsciiString::from_unchecked("test");
            let v: String = sas.into();
            assert_eq!(v, "test");
        }

        #[test]
        fn str_eq_string() {
            let str = SoftAsciiStr::from_str("hy").unwrap();
            let string = SoftAsciiString::from_string("hy").unwrap();

            assert_eq!(str, str);
            assert_eq!(str, string);
            assert_eq!(string, str);
            assert_eq!(string, string);
        }

        #[test]
        fn from_str() {
            use std::str::FromStr;
            let s: SoftAsciiString = assert_ok!(FromStr::from_str("hy ho"));
            assert_eq!(s, "hy ho");
            assert_err!("↓".parse::<SoftAsciiString>());
        }
    }
}
