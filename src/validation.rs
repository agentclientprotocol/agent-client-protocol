//! Input validation utilities for Agent Client Protocol types.
//!
//! This module provides validation functions for authentication-related protocol types
//! to ensure data integrity and security for user registration and authentication flows.

use std::collections::HashSet;

/// Validation errors that can occur when validating protocol types.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[allow(
    clippy::exhaustive_enums,
    reason = "Validation errors are part of the protocol schema surface"
)]
pub enum ValidationError {
    #[error("Field '{field}' is required but was empty or missing")]
    RequiredFieldMissing { field: String },

    #[error("Field '{field}' is too short (minimum length: {min_length})")]
    TooShort { field: String, min_length: usize },

    #[error("Field '{field}' is too long (maximum length: {max_length})")]
    TooLong { field: String, max_length: usize },

    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },

    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Duplicate identifier found: {id}")]
    DuplicateId { id: String },

    #[error("Environment variable name '{name}' is invalid: {reason}")]
    InvalidEnvVarName { name: String, reason: String },

    #[error("URL '{url}' is invalid: {reason}")]
    InvalidUrl { url: String, reason: String },
}

pub type ValidationResult<T> = Result<T, ValidationError>;

/// Trait for types that can be validated.
pub trait Validate {
    /// Validates the type and returns any validation errors.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] when validation fails.
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Validation constraints for string fields.
#[derive(Debug, Clone, Default)]
#[allow(
    clippy::exhaustive_structs,
    reason = "Validation constraints are part of the protocol schema surface"
)]
pub struct StringConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub allow_empty: bool,
    pub pattern: Option<regex::Regex>,
}

/// Validates a string field against constraints.
///
/// # Errors
///
/// Returns [`ValidationError`] when the field is empty while required, outside
/// configured length limits, or does not match the configured pattern.
pub fn validate_string_field(
    value: &str,
    field_name: &str,
    constraints: &StringConstraints,
) -> ValidationResult<()> {
    // Check if empty when not allowed
    if value.is_empty() && !constraints.allow_empty {
        return Err(ValidationError::RequiredFieldMissing {
            field: field_name.to_string(),
        });
    }

    // Check minimum length
    if let Some(min_len) = constraints.min_length
        && value.len() < min_len
    {
        return Err(ValidationError::TooShort {
            field: field_name.to_string(),
            min_length: min_len,
        });
    }

    // Check maximum length
    if let Some(max_len) = constraints.max_length
        && value.len() > max_len
    {
        return Err(ValidationError::TooLong {
            field: field_name.to_string(),
            max_length: max_len,
        });
    }

    // Check pattern match
    if let Some(pattern) = &constraints.pattern
        && !pattern.is_match(value)
    {
        return Err(ValidationError::InvalidCharacters {
            field: field_name.to_string(),
        });
    }

    Ok(())
}

/// Validates an optional string field.
///
/// # Errors
///
/// Returns [`ValidationError`] when the value is present and fails
/// [`validate_string_field`].
pub fn validate_optional_string_field(
    value: &Option<String>,
    field_name: &str,
    constraints: &StringConstraints,
) -> ValidationResult<()> {
    if let Some(val) = value {
        validate_string_field(val, field_name, constraints)?;
    }
    Ok(())
}

/// Validates an authentication method ID.
///
/// # Errors
///
/// Returns [`ValidationError`] when the id is empty, too long, or contains
/// invalid characters.
pub fn validate_auth_method_id(id: &str) -> ValidationResult<()> {
    let pattern = regex::Regex::new(r"^[a-zA-Z0-9_\-\.]+$").map_err(|reason| {
        ValidationError::InvalidFormat {
            field: "method_id".to_string(),
            reason: reason.to_string(),
        }
    })?;

    let constraints = StringConstraints {
        min_length: Some(1),
        max_length: Some(128),
        allow_empty: false,
        pattern: Some(pattern),
    };

    validate_string_field(id, "method_id", &constraints)
}

/// Validates an environment variable name.
///
/// # Errors
///
/// Returns [`ValidationError`] when the name is empty, not POSIX-compliant, or
/// reserved.
pub fn validate_env_var_name(name: &str) -> ValidationResult<()> {
    if name.is_empty() {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "Environment variable name cannot be empty".to_string(),
        });
    }

    // Environment variable names should follow POSIX standards
    let env_var_regex = regex::Regex::new(r"^[A-Z][A-Z0-9_]*$").map_err(|reason| {
        ValidationError::InvalidFormat {
            field: "name".to_string(),
            reason: reason.to_string(),
        }
    })?;
    if !env_var_regex.is_match(name) {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "Environment variable name must start with a letter and contain only uppercase letters, numbers, and underscores".to_string(),
        });
    }

    // Check for reserved names
    let reserved_names = [
        "PATH", "HOME", "USER", "USERNAME", "SHELL", "TERM", "PWD", "TMPDIR", "TMP", "TEMP",
        "LANG", "LC_ALL", "LC_CTYPE", "TZ", "DISPLAY",
    ];

    if reserved_names.contains(&name) {
        return Err(ValidationError::InvalidEnvVarName {
            name: name.to_string(),
            reason: "This environment variable name is reserved by the system".to_string(),
        });
    }

    Ok(())
}

/// Validates a URL string.
///
/// # Errors
///
/// Returns [`ValidationError`] when the URL is empty, does not use HTTP(S), or
/// contains whitespace.
pub fn validate_url(url: &str) -> ValidationResult<()> {
    if url.is_empty() {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }

    // Basic URL validation - check for protocol
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        });
    }

    // Check for obvious malformed URLs
    if url.contains(' ') || url.contains('\n') || url.contains('\t') {
        return Err(ValidationError::InvalidUrl {
            url: url.to_string(),
            reason: "URL cannot contain whitespace characters".to_string(),
        });
    }

    Ok(())
}

/// Validates that all IDs in a collection are unique.
///
/// # Errors
///
/// Returns [`ValidationError::DuplicateId`] when duplicate identifiers are
/// detected.
pub fn validate_unique_ids<T>(items: &[T], get_id: impl Fn(&T) -> &str) -> ValidationResult<()> {
    let mut seen_ids = HashSet::new();

    for item in items {
        let id = get_id(item);
        if seen_ids.contains(id) {
            return Err(ValidationError::DuplicateId { id: id.to_string() });
        }
        seen_ids.insert(id.to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_field_empty_not_allowed() {
        let constraints = StringConstraints::default();
        let result = validate_string_field("", "test_field", &constraints);
        assert!(matches!(
            result,
            Err(ValidationError::RequiredFieldMissing { .. })
        ));
    }

    #[test]
    fn test_validate_string_field_empty_allowed() {
        let constraints = StringConstraints {
            allow_empty: true,
            ..Default::default()
        };
        let result = validate_string_field("", "test_field", &constraints);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_string_field_too_short() {
        let constraints = StringConstraints {
            min_length: Some(5),
            ..Default::default()
        };
        let result = validate_string_field("abc", "test_field", &constraints);
        assert!(matches!(result, Err(ValidationError::TooShort { .. })));
    }

    #[test]
    fn test_validate_string_field_too_long() {
        let constraints = StringConstraints {
            max_length: Some(5),
            allow_empty: true,
            ..Default::default()
        };
        let result = validate_string_field("abcdefgh", "test_field", &constraints);
        assert!(matches!(result, Err(ValidationError::TooLong { .. })));
    }

    #[test]
    fn test_validate_auth_method_id_valid() {
        assert!(validate_auth_method_id("my_auth_method").is_ok());
        assert!(validate_auth_method_id("auth-method-1").is_ok());
        assert!(validate_auth_method_id("com.example.auth").is_ok());
    }

    #[test]
    fn test_validate_auth_method_id_invalid() {
        assert!(validate_auth_method_id("").is_err());
        assert!(validate_auth_method_id("auth method").is_err()); // spaces not allowed
        assert!(validate_auth_method_id("auth@method").is_err()); // @ not allowed
    }

    #[test]
    fn test_validate_env_var_name_valid() {
        assert!(validate_env_var_name("API_KEY").is_ok());
        assert!(validate_env_var_name("MY_SECRET_TOKEN").is_ok());
        assert!(validate_env_var_name("CONFIG_VAR_123").is_ok());
    }

    #[test]
    fn test_validate_env_var_name_invalid() {
        assert!(validate_env_var_name("").is_err());
        assert!(validate_env_var_name("api_key").is_err()); // must be uppercase
        assert!(validate_env_var_name("123_KEY").is_err()); // must start with letter
        assert!(validate_env_var_name("PATH").is_err()); // reserved name
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080/path").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err()); // not http/https
        assert!(validate_url("https://example .com").is_err()); // contains space
    }

    #[test]
    fn test_validate_unique_ids() {
        let items = vec!["id1", "id2", "id3"];
        assert!(validate_unique_ids(&items, |item| item).is_ok());

        let items_with_duplicate = vec!["id1", "id2", "id1"];
        assert!(validate_unique_ids(&items_with_duplicate, |item| item).is_err());
    }
}
