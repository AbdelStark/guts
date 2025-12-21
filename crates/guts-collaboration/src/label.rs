//! Label types for categorizing issues and pull requests.

use serde::{Deserialize, Serialize};

/// A label that can be applied to issues and pull requests.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label {
    /// Label name (e.g., "bug", "enhancement", "documentation").
    pub name: String,
    /// Label color in hex format (e.g., "ff0000" for red).
    pub color: String,
    /// Optional description of the label.
    pub description: Option<String>,
}

impl Label {
    /// Creates a new label.
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            description: None,
        }
    }

    /// Creates a new label with a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Predefined labels for common use cases.
impl Label {
    /// Bug label.
    pub fn bug() -> Self {
        Self::new("bug", "d73a4a").with_description("Something isn't working")
    }

    /// Enhancement label.
    pub fn enhancement() -> Self {
        Self::new("enhancement", "a2eeef").with_description("New feature or request")
    }

    /// Documentation label.
    pub fn documentation() -> Self {
        Self::new("documentation", "0075ca").with_description("Improvements or additions to documentation")
    }

    /// Good first issue label.
    pub fn good_first_issue() -> Self {
        Self::new("good first issue", "7057ff").with_description("Good for newcomers")
    }

    /// Help wanted label.
    pub fn help_wanted() -> Self {
        Self::new("help wanted", "008672").with_description("Extra attention is needed")
    }

    /// Invalid label.
    pub fn invalid() -> Self {
        Self::new("invalid", "e4e669").with_description("This doesn't seem right")
    }

    /// Question label.
    pub fn question() -> Self {
        Self::new("question", "d876e3").with_description("Further information is requested")
    }

    /// Wontfix label.
    pub fn wontfix() -> Self {
        Self::new("wontfix", "ffffff").with_description("This will not be worked on")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label = Label::new("test", "ff0000").with_description("A test label");
        assert_eq!(label.name, "test");
        assert_eq!(label.color, "ff0000");
        assert_eq!(label.description, Some("A test label".to_string()));
    }

    #[test]
    fn test_predefined_labels() {
        let bug = Label::bug();
        assert_eq!(bug.name, "bug");
        assert_eq!(bug.color, "d73a4a");
        assert!(bug.description.is_some());

        let enhancement = Label::enhancement();
        assert_eq!(enhancement.name, "enhancement");
    }
}
