
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    text: Arc<String>,
    start: usize,
    end: usize,
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.text[self.start..self.end].as_ref()
    }
}

impl Text {
    pub fn new(text: &dyn AsRef<str>) -> Self {
        Self {
            text: Arc::new(text.as_ref().to_string()),
            start: 0,
            end: text.as_ref().len(),
        }
    }
    pub fn char(&self, pos: usize) -> Option<char> {
        self.as_ref().chars().collect::<Vec<_>>().get(pos).copied()
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn starts_with(&self, needle: &str) -> bool {
        self.as_ref().starts_with(needle)
    }
    pub fn starts_with_any(&self, needles: &[&str]) -> bool {
        let subtext = self.as_ref();
        needles.iter().any(|needle| subtext.starts_with(needle))
    }
    pub fn skip_chars(&self, start: usize) -> Self {
        Self {
            text: self.text.clone(),
            start: (self.start + start).min(self.end),
            end: self.end
        }
    }
    pub fn subtext(&self, start: usize, len: usize) -> Self {
        Self {
            text: self.text.clone(),
            start: (self.start + start).min(self.end),
            end: (self.start + start + len).min(self.end)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn as_ref() {
        let text1 = Text::new(&"Hello");

        assert_eq!(text1.as_ref(), "Hello");

        let text2 = Text::new(&"Hello");

        assert_eq!(text1, text2);
    }

    #[test]
    fn char() {
        assert_eq!(Text::new(&"Hello").char(3), Some('l'));
    }

    #[test]
    fn starts_with() {
        let text = Text::new(&"the brown fox jumps over the lazy dog");
        assert!(text.starts_with("the brown fox jumps"));
        assert!(!text.starts_with("brown fox jumps"));
        assert!(!text.starts_with("the brown fox jumps over the lazy dog again"));
    }

    #[test]
    fn starts_with_any() {
        let text = Text::new(&"the brown fox jumps over the lazy dog");
        assert!(text.starts_with_any(&["the", "brown", "fox", "jumps"]));
        assert!(!text.starts_with_any(&["some", "other", "text"]));
    }

    #[test]
    fn skip_chars() {
        let text = Text::new(&"the brown fox jumps over the lazy dog");
        let sub1 = text.skip_chars(4);
        assert_eq!(sub1.as_ref(), "brown fox jumps over the lazy dog");
    }

    #[test]
    fn subtext() {
        let text = Text::new(&"the brown fox jumps over the lazy dog");
        let sub1 = text.subtext(4, 15);
        assert_eq!(sub1.as_ref(), "brown fox jumps");
        assert_eq!(sub1.subtext(6, 3).as_ref(), "fox");
        assert_eq!(sub1.subtext(6, 20).as_ref(), "fox jumps");
        assert_eq!(sub1.subtext(60, 20).as_ref(), "");
        assert_eq!(sub1.subtext(6, 0).as_ref(), "");
    }

}