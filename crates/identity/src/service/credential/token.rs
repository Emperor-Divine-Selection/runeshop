use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use thiserror::Error;

const TOKEN_BYTES: usize = 32;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("[Technical] {0}")]
    Random(#[from] getrandom::Error),
}

pub fn generate_token() -> Result<String, TokenError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_43_chars() {
        assert_eq!(generate_token().unwrap().len(), 43)
    }

    #[test]
    fn token_has_no_padding() {
        assert!(!generate_token().unwrap().contains('='))
    }

    #[test]
    fn token_is_url_safe() {
        let token = generate_token().unwrap();
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        )
    }

    #[test]
    fn two_tokens_differ() {
        assert_ne!(generate_token().unwrap(), generate_token().unwrap())
    }
}
