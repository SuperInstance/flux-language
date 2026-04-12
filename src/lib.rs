#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Word,
    Number,
    Punct,
    Space,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub text: String,
}

pub struct Engine {
    tokens: Vec<Token>,
    patterns: Vec<String>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            patterns: Vec::new(),
        }
    }

    pub fn tokenize(&mut self, text: &str) {
        self.tokens.clear();
        let mut current = String::new();
        let mut classify = |c: char, current: &mut String, tokens: &mut Vec<Token>| {
            if current.is_empty() {
                return;
            }
            let t = classify_token(current);
            tokens.push(t);
            current.clear();
        };

        for c in text.chars() {
            if c.is_whitespace() {
                classify(c, &mut current, &mut self.tokens);
                self.tokens.push(Token {
                    token_type: TokenType::Space,
                    text: c.to_string(),
                });
            } else if c == '.' && !current.is_empty() && current.chars().all(|x| x.is_ascii_digit()) {
                // decimal dot: attach to current number
                current.push(c);
            } else if c.is_alphanumeric() || c == '_' {
                if !current.is_empty() && c.is_ascii_digit() && !current.ends_with('.') && current.chars().next().map_or(false, |x| x.is_alphabetic()) {
                    classify(c, &mut current, &mut self.tokens);
                }
                current.push(c);
            } else {
                classify(c, &mut current, &mut self.tokens);
                self.tokens.push(Token {
                    token_type: TokenType::Punct,
                    text: c.to_string(),
                });
            }
        }
        if !current.is_empty() {
            self.tokens.push(classify_token(&current));
        }
    }

    pub fn tokenize_strict(&mut self, text: &str) {
        self.tokenize(text);
        self.tokens.retain(|t| t.token_type != TokenType::Space);
    }

    pub fn count_words(&self) -> usize {
        self.tokens.iter().filter(|t| t.token_type == TokenType::Word).count()
    }

    pub fn contains(&self, word: &str) -> bool {
        self.tokens.iter().any(|t| t.text == word)
    }

    pub fn add_pattern(&mut self, pattern: &str) {
        self.patterns.push(pattern.to_string());
    }

    pub fn match_pattern(&self, pattern: &str) -> bool {
        let haystack: String = self.tokens.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join("");
        haystack.contains(pattern)
    }

    pub fn match_exact(&self, word: &str) -> bool {
        self.tokens.iter().any(|t| t.token_type == TokenType::Word && t.text == word)
    }

    pub fn word_frequency(&self, word: &str) -> usize {
        self.tokens.iter().filter(|t| t.text == word).count()
    }

    pub fn most_common(&self) -> Option<String> {
        let mut freq = std::collections::HashMap::new();
        for t in &self.tokens {
            if t.token_type != TokenType::Space {
                *freq.entry(t.text.clone()).or_insert(0usize) += 1;
            }
        }
        freq.into_iter().max_by_key(|(_, c)| *c).map(|(w, _)| w)
    }

    pub fn similarity(a: &str, b: &str) -> usize {
        if a.is_empty() && b.is_empty() {
            return 100;
        }
        if a.is_empty() || b.is_empty() {
            return 0;
        }
        let set_a: std::collections::HashSet<char> = a.chars().collect();
        let set_b: std::collections::HashSet<char> = b.chars().collect();
        let union: std::collections::HashSet<char> = set_a.union(&set_b).cloned().collect();
        let intersection: std::collections::HashSet<char> = set_a.intersection(&set_b).cloned().collect();
        if union.is_empty() {
            return 100;
        }
        ((intersection.len() * 100) / union.len())
    }

    pub fn is_number(text: &str) -> bool {
        text.parse::<f64>().is_ok()
    }

    pub fn extract_number(&self, index: usize) -> Option<f64> {
        self.tokens.get(index).and_then(|t| {
            if t.token_type == TokenType::Number { t.text.parse::<f64>().ok() } else { None }
        })
    }
}

fn char_type(c: &char) -> TokenType {
    if c.is_ascii_digit() {
        TokenType::Number
    } else {
        TokenType::Word
    }
}

fn classify_token(s: &str) -> Token {
    if s.parse::<f64>().is_ok() {
        Token { token_type: TokenType::Number, text: s.to_string() }
    } else if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Token { token_type: TokenType::Word, text: s.to_string() }
    } else {
        Token { token_type: TokenType::Unknown, text: s.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let e = Engine::new();
        assert_eq!(e.count_words(), 0);
    }

    #[test]
    fn test_tokenize_words() {
        let mut e = Engine::new();
        e.tokenize("hello world");
        assert_eq!(e.count_words(), 2);
        assert_eq!(e.tokens[0].text, "hello");
        assert_eq!(e.tokens[0].token_type, TokenType::Word);
    }

    #[test]
    fn test_tokenize_numbers() {
        let mut e = Engine::new();
        e.tokenize("42 apples");
        assert_eq!(e.tokens[0].token_type, TokenType::Number);
        assert_eq!(e.tokens[0].text, "42");
        assert_eq!(e.tokens[1].token_type, TokenType::Space);
        assert_eq!(e.tokens[2].token_type, TokenType::Word);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let mut e = Engine::new();
        e.tokenize("hi, there!");
        assert_eq!(e.tokens[1].token_type, TokenType::Punct);
        assert_eq!(e.tokens[1].text, ",");
        assert_eq!(e.tokens[4].token_type, TokenType::Punct);
    }

    #[test]
    fn test_tokenize_spaces() {
        let mut e = Engine::new();
        e.tokenize("a  b");
        assert_eq!(e.tokens.len(), 4);
        assert_eq!(e.tokens[1].token_type, TokenType::Space);
        assert_eq!(e.tokens[2].token_type, TokenType::Space);
    }

    #[test]
    fn test_tokenize_strict_skips_spaces() {
        let mut e = Engine::new();
        e.tokenize_strict("hello world foo");
        assert!(e.tokens.iter().all(|t| t.token_type != TokenType::Space));
        assert_eq!(e.tokens.len(), 3);
    }

    #[test]
    fn test_count_words() {
        let mut e = Engine::new();
        e.tokenize("the quick brown fox");
        assert_eq!(e.count_words(), 4);
    }

    #[test]
    fn test_contains() {
        let mut e = Engine::new();
        e.tokenize("hello world");
        assert!(e.contains("hello"));
        assert!(!e.contains("bye"));
    }

    #[test]
    fn test_add_and_match_pattern() {
        let mut e = Engine::new();
        e.tokenize("hello world");
        e.add_pattern("world");
        assert!(e.match_pattern("world"));
        assert!(!e.match_pattern("planet"));
    }

    #[test]
    fn test_match_exact() {
        let mut e = Engine::new();
        e.tokenize("cat cats catalog");
        assert!(e.match_exact("cat"));
        assert!(!e.match_exact("catal"));
    }

    #[test]
    fn test_word_frequency() {
        let mut e = Engine::new();
        e.tokenize("the cat sat on the mat the cat");
        assert_eq!(e.word_frequency("the"), 3);
        assert_eq!(e.word_frequency("cat"), 2);
        assert_eq!(e.word_frequency("dog"), 0);
    }

    #[test]
    fn test_most_common() {
        let mut e = Engine::new();
        e.tokenize("a b a c a");
        assert_eq!(e.most_common(), Some("a".to_string()));
    }

    #[test]
    fn test_most_common_empty() {
        let e = Engine::new();
        assert_eq!(e.most_common(), None);
    }

    #[test]
    fn test_similarity_identical() {
        assert_eq!(Engine::similarity("abc", "abc"), 100);
    }

    #[test]
    fn test_similarity_different() {
        assert_eq!(Engine::similarity("abc", "xyz"), 0);
    }

    #[test]
    fn test_similarity_partial() {
        let sim = Engine::similarity("abc", "abd");
        assert!(sim > 0 && sim < 100);
    }

    #[test]
    fn test_similarity_empty() {
        assert_eq!(Engine::similarity("", "abc"), 0);
        assert_eq!(Engine::similarity("", ""), 100);
    }

    #[test]
    fn test_is_number() {
        assert!(Engine::is_number("42"));
        assert!(Engine::is_number("3.14"));
        assert!(!Engine::is_number("abc"));
        assert!(!Engine::is_number(""));
    }

    #[test]
    fn test_extract_number() {
        let mut e = Engine::new();
        e.tokenize("3.14 is pi");
        assert_eq!(e.extract_number(0), Some(3.14));
        assert_eq!(e.extract_number(1), None);
    }

    #[test]
    fn test_empty_tokenize() {
        let mut e = Engine::new();
        e.tokenize("");
        assert!(e.tokens.is_empty());
        assert_eq!(e.count_words(), 0);
    }
}
