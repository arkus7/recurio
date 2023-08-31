use chrono::{DateTime, Utc};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct Service {
    pub id: Uuid,
    pub name: ServiceName,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct ServiceId(Uuid);

pub struct NewService {
    pub name: ServiceName,
}

#[derive(Debug, serde::Serialize)]
pub struct ServiceName(String);

impl ServiceName {
    pub fn parse(s: String) -> Result<ServiceName, String> {
        const SERVICE_NAME_MAX_LENGTH: usize = 64;
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > SERVICE_NAME_MAX_LENGTH;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid service name.", s))
        } else {
            Ok(Self(s.trim().to_owned()))
        }
    }
}

impl AsRef<str> for ServiceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// FIXME: without implementing From<String>, sqlx has problem with mapping DB results
// Current approach allows to create `ServiceName` without using `ServiceName::parse` function
// via `String::new("   ").into::<ServiceName>()` which might cause some issues (no validation)
impl From<String> for ServiceName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ServiceName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_64_grapheme_long_name_is_valid() {
        let name = "ё".repeat(64);
        assert_ok!(ServiceName::parse(name));
    }

    #[test]
    fn name_longer_than_64_graphemes_is_invalid() {
        let name = "ё".repeat(128);
        assert_err!(ServiceName::parse(name));
    }

    #[test]
    fn whitespaces_only_are_rejected() {
        let name = " ".to_string();
        assert_err!(ServiceName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(ServiceName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(ServiceName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Disney+".to_string();
        assert_ok!(ServiceName::parse(name));
    }

    #[test]
    fn additional_whitespace_gets_trimmed() {
        let name = " Netflix ".to_string();
        let service_name = ServiceName::parse(name).unwrap();
        assert_eq!(service_name.as_ref(), "Netflix");
    }

    #[test]
    #[ignore] // Check comment above `impl From<String>`
    fn from_string() {
        let name = "   ".to_string();
        let service_name = Into::<ServiceName>::into(name.clone());
        assert_ne!(service_name.as_ref(), name.as_str());
    }
}
