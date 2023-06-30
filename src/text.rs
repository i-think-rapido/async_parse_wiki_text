
use std::sync::Arc;

/// This is the text wrapper struct for the parse input
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Text {
    text: Arc<String>,
}

impl Text
{
    /// Creation of this struct by providing an `AsRef<str>`
    /// In general just a &String or &str
    /// This struct can then be shared among threads
    pub fn new<T: AsRef<str> + Sync + Send>(value: T) -> Self {
        let s = value.as_ref().to_owned(); // essentially a clone
        Self { text: Arc::new(s) }
    }
    /// The length of the text data
    pub fn len(&self) -> usize {
        self.text.len()
    }
    /// Test if if the text data is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.text.as_str()
    }
}
impl TextSlice for Text {
    fn as_str(&self, pos: usize) -> TextSliceResult {
        if pos >= self.text.len() {
            TextSliceResult::None
        }
        else {
            TextSliceResult::Some(&self.text[pos..=pos])
        }
    }
    fn as_slice(&self, start: usize, end: usize) -> TextSliceResult {
        if end < start || start >= self.text.len() {
            TextSliceResult::None
        }
        else if end >= self.text.len() {
            TextSliceResult::Partial(&self.text[start..self.text.len()])
        }
        else {
            TextSliceResult::Some(&self.text[start..end])
        }
    }
}

pub trait TextSlice {
    fn as_str(&self, pos: usize) -> TextSliceResult;
    fn as_slice(&self, start: usize, end: usize) -> TextSliceResult;
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TextSliceResult<'a> {
    Some(&'a str),
    Partial(&'a str),
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

        let origin = "this is a text";

        let text = Text::new(origin);

        assert_eq!(text.as_ref(), origin);
        let t = text.clone();
        assert_eq!(t.as_ref(), origin);

        let string = origin.to_string();
        let text = Text::new(string);

        assert_eq!(text.as_ref(), origin);
        let t = text.clone();
        assert_eq!(t.as_ref(), origin);

        assert_eq!(t.as_str(2), TextSliceResult::Some("i"));
        assert_eq!(t.as_str(20), TextSliceResult::None);
        assert_eq!(t.as_slice(0, 4), TextSliceResult::Some("this"));
        assert_eq!(t.as_slice(5, 9), TextSliceResult::Some("is a"));
        assert_eq!(t.as_slice(10, 20), TextSliceResult::Partial("text"));
        assert_eq!(t.as_slice(20, 30), TextSliceResult::None);
        assert_eq!(t.as_slice(3, 1), TextSliceResult::None);
    }
}
