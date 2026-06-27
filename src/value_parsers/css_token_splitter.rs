/// Paren-aware CSS value tokenizer.
/// Splits on top-level whitespace or commas so function values like
/// `calc(50% - 10px)` or `rgb(0, 0, 0)` stay intact as a single token.
pub struct CssTokenSplitter;

impl CssTokenSplitter {
    /// Split a CSS value on top-level whitespace runs (paren-aware).
    pub fn split_by_whitespace(value: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut depth: i32 = 0;

        for ch in value.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    depth = (depth - 1).max(0);
                    current.push(ch);
                }
                ' ' | '\t' | '\n' | '\r' if depth == 0 => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    /// Split a CSS value on top-level commas (paren-aware), trimming each segment.
    pub fn split_by_comma(value: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut depth: i32 = 0;

        for ch in value.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    depth = (depth - 1).max(0);
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    let trimmed = current.trim().to_string();
                    tokens.push(trimmed);
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        let last = current.trim().to_string();
        if !last.is_empty() {
            tokens.push(last);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_whitespace_simple() {
        assert_eq!(
            CssTokenSplitter::split_by_whitespace("10px 20px"),
            vec!["10px", "20px"]
        );
    }

    #[test]
    fn split_whitespace_keeps_calc_intact() {
        assert_eq!(
            CssTokenSplitter::split_by_whitespace("calc(100% - 10px) 20px"),
            vec!["calc(100% - 10px)", "20px"]
        );
    }

    #[test]
    fn split_whitespace_keeps_rgb_intact() {
        assert_eq!(
            CssTokenSplitter::split_by_whitespace("rgb(255 0 0)"),
            vec!["rgb(255 0 0)"]
        );
    }

    #[test]
    fn split_comma_simple() {
        assert_eq!(
            CssTokenSplitter::split_by_comma("red, green, blue"),
            vec!["red", "green", "blue"]
        );
    }

    #[test]
    fn split_comma_keeps_func_intact() {
        assert_eq!(
            CssTokenSplitter::split_by_comma("rgba(0, 0, 0, 0.5), red"),
            vec!["rgba(0, 0, 0, 0.5)", "red"]
        );
    }
}
