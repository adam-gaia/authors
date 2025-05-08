use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use winnow::Result;
use winnow::ascii::space0;
use winnow::combinator::opt;
use winnow::combinator::separated;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Accumulate;
use winnow::token::take_till;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Author {
    name: String,
    email: Option<String>,
}

impl Author {
    pub fn new(name: String, email: Option<String>) -> Self {
        Self { name, email }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> Option<&String> {
        self.email.as_ref()
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(email) = &self.email {
            write!(f, " <{email}>")?;
        }
        Ok(())
    }
}

fn separator(s: &mut &str) -> Result<()> {
    let _ = space0.parse_next(s)?;
    let _ = ",".parse_next(s)?;
    let _ = space0.parse_next(s)?;
    Ok(())
}

fn name(s: &mut &str) -> Result<String> {
    let name = take_till(1.., |c| matches!(c, ']' | '<' | ',' | '"')).parse_next(s)?;
    let name = name.trim().to_string();
    Ok(name)
}

fn email(s: &mut &str) -> Result<String> {
    let _ = "<".parse_next(s)?;
    let email = take_till(1.., |c| c == '>')
        .map(|x: &str| x.to_string())
        .parse_next(s)?;
    let _ = ">".parse_next(s)?;
    Ok(email)
}

fn author(s: &mut &str) -> Result<Author> {
    let _ = opt("\"").parse_next(s)?;
    let name = name.parse_next(s)?;
    let email = opt(email).parse_next(s)?;
    let _ = opt("\"").parse_next(s)?;
    Ok(Author { name, email })
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    message: String,
    span: std::ops::Range<usize>,
    input: String,
}

impl ParseError {
    fn from_parse(error: winnow::error::ParseError<&str, ContextError>) -> Self {
        let message = error.inner().to_string();
        let input = (*error.input()).to_owned();
        let span = error.char_span();
        Self {
            message,
            span,
            input,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = annotate_snippets::Level::Error
            .title(&self.message)
            .snippet(
                annotate_snippets::Snippet::source(&self.input)
                    .fold(true)
                    .annotation(annotate_snippets::Level::Error.span(self.span.clone())),
            );
        let renderer = annotate_snippets::Renderer::plain();
        let rendered = renderer.render(message);
        rendered.fmt(f)
    }
}

impl std::error::Error for ParseError {}

impl FromStr for Authors {
    type Err = ParseError;
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        authors.parse(input).map_err(|e| ParseError::from_parse(e))
    }
}

impl FromStr for Author {
    type Err = ParseError;
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        author.parse(input).map_err(|e| ParseError::from_parse(e))
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Authors {
    authors: Vec<Author>,
}

impl Authors {
    pub fn len(&self) -> usize {
        self.authors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.authors.is_empty()
    }
}

impl<'a> IntoIterator for &'a Authors {
    type Item = &'a Author;
    type IntoIter = std::slice::Iter<'a, Author>;
    fn into_iter(self) -> Self::IntoIter {
        self.authors.iter()
    }
}

impl<'a> IntoIterator for &'a mut Authors {
    type Item = &'a mut Author;
    type IntoIter = std::slice::IterMut<'a, Author>;
    fn into_iter(self) -> Self::IntoIter {
        self.authors.iter_mut()
    }
}

impl Accumulate<Author> for Authors {
    fn initial(capacity: Option<usize>) -> Self {
        let authors = match capacity {
            Some(capacity) => Vec::with_capacity(capacity),
            None => Vec::new(),
        };
        Authors { authors }
    }

    fn accumulate(&mut self, acc: Author) {
        self.authors.push(acc);
    }
}

fn authors(s: &mut &str) -> winnow::Result<Authors> {
    let _ = opt("[").parse_next(s)?;
    let authors = separated(1.., author, separator).parse_next(s)?;
    let _ = opt("]").parse_next(s)?;
    Ok(authors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use s_string::s;
    use std::cell::LazyCell;

    const FOOBAR: LazyCell<Authors> = LazyCell::new(|| Authors {
        authors: vec![Author {
            name: s!("Foo Bar"),
            email: Some(s!("foo@bar.com")),
        }],
    });

    const FOOBAR_NO_EMAIL: LazyCell<Authors> = LazyCell::new(|| Authors {
        authors: vec![Author {
            name: s!("Foo Bar"),
            email: None,
        }],
    });

    const FOOBAR_AUTHOR: LazyCell<Author> = LazyCell::new(|| Author {
        name: s!("Foo Bar"),
        email: Some(s!("foo@bar.com")),
    });

    const FOOBAR_NO_EMAIL_AUTHOR: LazyCell<Author> = LazyCell::new(|| Author {
        name: s!("Foo Bar"),
        email: None,
    });

    const MULTIPLE: LazyCell<Authors> = LazyCell::new(|| Authors {
        authors: vec![
            Author {
                name: s!("Foo Bar"),
                email: Some(s!("foo@bar.com")),
            },
            Author {
                name: s!("Foo2 Bar"),
                email: Some(s!("foo2@bar.com")),
            },
            Author {
                name: s!("Foo3 Bar"),
                email: Some(s!("foo3@bar.com")),
            },
        ],
    });

    #[test]
    fn test_parse_email() {
        let mut input = "<firstlast@foo.com>";
        let expected = s!("firstlast@foo.com");
        let actual = email.parse_next(&mut input);
        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn test_parse_author() {
        let mut input = "First Last <firstlast@foo.com>";
        let expected = Author {
            name: s!("First Last"),
            email: Some(s!("firstlast@foo.com")),
        };
        let actual = author.parse_next(&mut input);
        assert_eq!(Ok(expected), actual);
    }

    #[rstest]
    #[case("Foo Bar <foo@bar.com>", FOOBAR)]
    #[case("[Foo Bar <foo@bar.com>]", FOOBAR)]
    #[case("\"Foo Bar <foo@bar.com>\"", FOOBAR)]
    #[case("[\"Foo Bar <foo@bar.com>\"]", FOOBAR)]
    #[case("Foo Bar", FOOBAR_NO_EMAIL)]
    #[case("[Foo Bar]", FOOBAR_NO_EMAIL)]
    #[case("\"Foo Bar\"", FOOBAR_NO_EMAIL)]
    #[case("[\"Foo Bar\"]", FOOBAR_NO_EMAIL)]
    fn test_single_authors(#[case] input: &str, #[case] expected: LazyCell<Authors>) {
        let actual = Authors::from_str(input);
        assert_eq!(Ok((*expected).clone()), actual);
    }

    #[rstest]
    #[case(
        "[\"Foo Bar <foo@bar.com>\", \"Foo2 Bar <foo2@bar.com>\", \"Foo3 Bar <foo3@bar.com>\"]",
        MULTIPLE
    )]
    #[case(
        "[Foo Bar <foo@bar.com>, Foo2 Bar <foo2@bar.com>, Foo3 Bar <foo3@bar.com>]",
        MULTIPLE
    )]
    fn test_multiple_authors(#[case] input: &str, #[case] expected: LazyCell<Authors>) {
        let actual = Authors::from_str(input);
        assert_eq!(Ok((*expected).clone()), actual);
    }

    #[rstest]
    #[case("Foo Bar <foo@bar.com>", FOOBAR_AUTHOR)]
    #[case("\"Foo Bar <foo@bar.com>\"", FOOBAR_AUTHOR)]
    #[case("Foo Bar", FOOBAR_NO_EMAIL_AUTHOR)]
    #[case("\"Foo Bar\"", FOOBAR_NO_EMAIL_AUTHOR)]
    fn test_author(#[case] input: &str, #[case] expected: LazyCell<Author>) {
        let actual = Author::from_str(input);
        assert_eq!(Ok((*expected).clone()), actual);
    }
}
