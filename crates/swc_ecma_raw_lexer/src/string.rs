use logos::{Lexer, Logos};

use crate::{LogosError, TokenType};

pub fn consume_str_single_quote(lex: &mut Lexer<TokenType>) -> Result<(), LogosError> {
    consume_str(lex, StrContent::SingleQuote)
}

pub fn consume_str_double_quote(lex: &mut Lexer<TokenType>) -> Result<(), LogosError> {
    consume_str(lex, StrContent::DoubleQuote)
}

fn consume_str(lex: &mut Lexer<TokenType>, stop_token: StrContent) -> Result<(), LogosError> {
    let remainder = lex.remainder();
    let total_len = remainder.len();

    let mut str_lexer = Lexer::<StrContent>::new(remainder);
    let mut terminated = false;

    while let Some(Ok(token)) = str_lexer.next() {
        if token == stop_token {
            terminated = true;
            break;
        }
    }

    let left_len = str_lexer.remainder().len();
    let consumed = total_len - left_len;
    lex.bump(consumed);

    if !terminated {
        return Err(LogosError::UnterminatedStr);
    }

    Ok(())
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum StrContent {
    #[regex(r#"\\["'\\bfnrtv]"#, priority = 100)]
    #[regex(r#"\\0[0-7]*"#, priority = 100)]
    #[regex(r#"\\x[0-9a-fA-F]{2}"#, priority = 100)]
    #[regex(r#"\\u[0-9a-fA-F]{4}"#, priority = 100)]
    #[regex(r#"\\[^'"\\]+"#)]
    Escape,

    #[regex(r#"[^'"\\]+"#)]
    Normal,

    #[regex(r#"'"#)]
    SingleQuote,

    #[regex(r#"""#)]
    DoubleQuote,
}

#[cfg(test)]
mod tests {
    use logos::Lexer;
    use pretty_assertions::assert_eq;

    use super::StrContent;

    fn assert_str(text: &str, expected: &[StrContent]) {
        dbg!(text);
        dbg!(expected);

        let actual = Lexer::<StrContent>::new(text)
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let mut lexer = Lexer::<StrContent>::new(text);

        while let Some(Ok(token)) = lexer.next() {
            dbg!(&token);
            dbg!(lexer.slice());
        }

        // Actual contains the last quote

        assert_eq!(expected.len() + 1, actual.len());
        assert_eq!(expected, &actual[..expected.len()]);

        assert!(matches!(
            actual.last(),
            Some(StrContent::SingleQuote | StrContent::DoubleQuote)
        ));
    }

    #[test]
    fn test_newline() {
        assert_str(
            "hello\\nworld'",
            &[StrContent::Normal, StrContent::Escape, StrContent::Normal],
        );
    }

    #[test]
    fn test_escape() {
        assert_str(r#"\''"#, &[StrContent::Escape]);
    }

    #[test]
    fn test_escape_escape() {
        assert_str(r#"\\'"#, &[StrContent::Escape]);
    }
}
