//! string handles parsing of GPX-spec strings.

use std::io::Read;

use xml::reader::XmlEvent;

use crate::errors::{GpxError, GpxResult};
use crate::parser::{verify_starting_tag, Context};

/// consume consumes a single string as tag content.
pub fn consume<R: Read>(
    context: &mut Context<R>,
    tagname: &'static str,
    allow_empty: bool,
) -> GpxResult<String> {
    verify_starting_tag(context, tagname)?;
    let mut string = String::new();

    for event in context.reader() {
        match event? {
            XmlEvent::StartElement { ref name, .. } => {
                return Err(GpxError::InvalidChildElement(
                    name.local_name.clone(),
                    tagname,
                ));
            }
            XmlEvent::Characters(content) => string = content,
            XmlEvent::EndElement { ref name } => {
                if name.local_name != tagname {
                    return Err(GpxError::InvalidClosingTag(
                        name.local_name.clone(),
                        tagname,
                    ));
                }
                if allow_empty || !string.is_empty() {
                    return Ok(string);
                }
                return Err(GpxError::NoStringContent);
            }
            _ => {}
        }
    }
    Err(GpxError::MissingClosingTag(tagname))
}

#[cfg(test)]
mod tests {
    use super::consume;
    use crate::GpxVersion;

    #[test]
    fn consume_simple_string() {
        let result = consume!(
            "<string>hello world</string>",
            GpxVersion::Gpx11,
            "string",
            false
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn consume_new_tag() {
        // cannot start new tag inside string
        let result = consume!("<foo>bar<baz></baz></foo>", GpxVersion::Gpx11, "foo", false);

        assert!(result.is_err());
    }

    #[test]
    fn consume_start_tag() {
        // must have starting tag
        let result = consume!("bar</foo>", GpxVersion::Gpx11, "foo", false);

        assert!(result.is_err());
    }

    #[test]
    fn consume_end_tag() {
        // must have ending tag
        let result = consume!("<foo>bar", GpxVersion::Gpx11, "foo", false);

        assert!(result.is_err());
    }

    #[test]
    fn consume_no_body() {
        // must have string content
        let result = consume!("<foo></foo>", GpxVersion::Gpx11, "foo", false);

        assert!(result.is_err());
    }

    #[test]
    fn consume_different_ending_tag() {
        // this is just illegal
        let result = consume!("<foo></foobar>", GpxVersion::Gpx11, "foo", false);

        assert!(result.is_err());
    }
}
