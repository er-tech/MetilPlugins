/// Makes a text safe to put in an SVG, in a text or in an attribute: what could close a tag or an
/// attribute is replaced, and so are the characters XML does not allow at all.
pub fn escape_xml(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            // Control characters other than tab, newline and return cannot be in an XML document.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_could_break_out_of_a_tag_or_an_attribute_is_replaced() {
        assert_eq!(
            escape_xml("a < b & \"c\" > 'd'"),
            "a &lt; b &amp; &quot;c&quot; &gt; &apos;d&apos;"
        );
        assert_eq!(
            escape_xml("\"/><script>alert(1)</script>"),
            "&quot;/&gt;&lt;script&gt;alert(1)&lt;/script&gt;"
        );
    }

    #[test]
    fn ordinary_text_stays_and_control_characters_go() {
        assert_eq!(
            escape_xml("Hello, world. Olá! 你好"),
            "Hello, world. Olá! 你好"
        );
        assert_eq!(escape_xml("a\u{0}b\u{7}c\td"), "abc\td");
    }
}
