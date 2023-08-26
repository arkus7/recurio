use unicode_segmentation::UnicodeSegmentation;

pub struct NewService {
    name: ServiceName,
    image_url: Option<String>,
}

impl NewService {}

#[derive(Debug)]
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
