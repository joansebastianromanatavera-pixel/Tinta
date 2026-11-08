//! XML text escaping.

/// Escapes text for element content and double-quoted attributes.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn markup_characters_are_escaped() {
        assert_eq!(
            escape(r#"a < b & "c" > 'd'"#),
            "a &lt; b &amp; &quot;c&quot; &gt; &apos;d&apos;"
        );
    }

    #[test]
    fn typographic_characters_pass_through() {
        assert_eq!(escape("«Él» — ‘sí’…"), "«Él» — ‘sí’…");
    }
}
