//! Validation implementations for v2 protocol types.

use super::{AuthMethod, AuthMethodAgent, AuthMethodId, AuthenticateRequest};
use crate::validation::{
    StringConstraints, Validate, ValidationError, validate_auth_method_id,
    validate_optional_string_field, validate_string_field, validate_unique_ids,
};

#[cfg(feature = "unstable_auth_methods")]
use crate::validation::validate_env_var_name;
#[cfg(any(feature = "unstable_auth_methods", feature = "unstable_elicitation"))]
use crate::validation::validate_url;

#[cfg(feature = "unstable_auth_methods")]
use super::{AuthEnvVar, AuthMethodEnvVar};

#[cfg(feature = "unstable_elicitation")]
use super::{CreateElicitationRequest, ElicitationFormMode, ElicitationId, ElicitationUrlMode};

impl Validate for AuthenticateRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate the method_id format
        validate_auth_method_id(&self.method_id.0)?;
        Ok(())
    }
}

impl Validate for AuthMethodId {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_auth_method_id(&self.0)
    }
}

impl Validate for AuthMethodAgent {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate the ID
        self.id.validate()?;

        // Validate the name
        let name_constraints = StringConstraints {
            min_length: Some(1),
            max_length: Some(256),
            allow_empty: false,
            ..Default::default()
        };
        validate_string_field(&self.name, "name", &name_constraints)?;

        // Validate optional description
        let description_constraints = StringConstraints {
            min_length: None,
            max_length: Some(1024),
            allow_empty: true,
            ..Default::default()
        };
        validate_optional_string_field(&self.description, "description", &description_constraints)?;

        Ok(())
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl Validate for AuthMethodEnvVar {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate the ID
        self.id.validate()?;

        // Validate the name
        let name_constraints = StringConstraints {
            min_length: Some(1),
            max_length: Some(256),
            allow_empty: false,
            ..Default::default()
        };
        validate_string_field(&self.name, "name", &name_constraints)?;

        // Validate optional description
        let description_constraints = StringConstraints {
            min_length: None,
            max_length: Some(1024),
            allow_empty: true,
            ..Default::default()
        };
        validate_optional_string_field(&self.description, "description", &description_constraints)?;

        // Validate environment variables
        for env_var in &self.vars {
            env_var.validate()?;
        }

        // Check for duplicate environment variable names
        validate_unique_ids(&self.vars, |var| &var.name)?;

        // Validate optional link URL
        if let Some(link) = &self.link {
            validate_url(link)?;
        }

        Ok(())
    }
}

#[cfg(feature = "unstable_auth_methods")]
impl Validate for AuthEnvVar {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate environment variable name
        validate_env_var_name(&self.name)?;

        // Validate optional label
        let label_constraints = StringConstraints {
            min_length: None,
            max_length: Some(256),
            allow_empty: true,
            ..Default::default()
        };
        validate_optional_string_field(&self.label, "label", &label_constraints)?;

        Ok(())
    }
}

impl Validate for AuthMethod {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            AuthMethod::Agent(agent) => agent.validate(),
            #[cfg(feature = "unstable_auth_methods")]
            AuthMethod::EnvVar(env_var) => env_var.validate(),
            #[cfg(feature = "unstable_auth_methods")]
            AuthMethod::Terminal(terminal) => {
                // Validate terminal auth method (similar to agent)
                terminal.id.validate()?;

                let name_constraints = StringConstraints {
                    min_length: Some(1),
                    max_length: Some(256),
                    allow_empty: false,
                    ..Default::default()
                };
                validate_string_field(&terminal.name, "name", &name_constraints)?;

                let description_constraints = StringConstraints {
                    min_length: None,
                    max_length: Some(1024),
                    allow_empty: true,
                    ..Default::default()
                };
                validate_optional_string_field(
                    &terminal.description,
                    "description",
                    &description_constraints,
                )?;

                Ok(())
            }
        }
    }
}

/// Validates a collection of authentication methods.
///
/// # Errors
///
/// Returns [`ValidationError`] when any method is invalid or when duplicate
/// method IDs are present.
pub fn validate_auth_methods(methods: &[AuthMethod]) -> Result<(), ValidationError> {
    // Validate each method individually
    for method in methods {
        method.validate()?;
    }

    // Check for duplicate method IDs
    validate_unique_ids(methods, |method| &method.id().0)?;

    Ok(())
}

// Elicitation validation implementations

#[cfg(feature = "unstable_elicitation")]
impl Validate for ElicitationId {
    fn validate(&self) -> Result<(), ValidationError> {
        let pattern = regex::Regex::new(r"^[a-zA-Z0-9_\-\.]+$").map_err(|reason| {
            ValidationError::InvalidFormat {
                field: "elicitation_id".to_string(),
                reason: reason.to_string(),
            }
        })?;

        let constraints = StringConstraints {
            min_length: Some(1),
            max_length: Some(128),
            allow_empty: false,
            pattern: Some(pattern),
        };
        validate_string_field(&self.0, "elicitation_id", &constraints)
    }
}

#[cfg(feature = "unstable_elicitation")]
impl Validate for CreateElicitationRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate the message field
        let message_constraints = StringConstraints {
            min_length: Some(1),
            max_length: Some(2048),
            allow_empty: false,
            ..Default::default()
        };
        validate_string_field(&self.message, "message", &message_constraints)?;

        // Validate mode-specific fields
        match &self.mode {
            #[cfg(feature = "unstable_elicitation")]
            crate::v2::ElicitationMode::Form(form) => form.validate()?,
            #[cfg(feature = "unstable_elicitation")]
            crate::v2::ElicitationMode::Url(url) => url.validate()?,
        }

        Ok(())
    }
}

#[cfg(feature = "unstable_elicitation")]
impl Validate for ElicitationFormMode {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate that the requested_schema has at least one property
        if self.requested_schema.properties.is_empty() {
            return Err(ValidationError::RequiredFieldMissing {
                field: "requested_schema.properties".to_string(),
            });
        }

        // Validate title if present
        if let Some(title) = &self.requested_schema.title {
            let title_constraints = StringConstraints {
                min_length: Some(1),
                max_length: Some(256),
                allow_empty: false,
                ..Default::default()
            };
            validate_string_field(title, "title", &title_constraints)?;
        }

        // Validate description if present
        if let Some(description) = &self.requested_schema.description {
            let description_constraints = StringConstraints {
                min_length: None,
                max_length: Some(1024),
                allow_empty: true,
                ..Default::default()
            };
            validate_string_field(description, "description", &description_constraints)?;
        }

        Ok(())
    }
}

#[cfg(feature = "unstable_elicitation")]
impl Validate for ElicitationUrlMode {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate the elicitation ID
        self.elicitation_id.validate()?;

        // Validate the URL
        validate_url(&self.url)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticate_request_validation() {
        let valid_request = AuthenticateRequest {
            method_id: AuthMethodId::new("valid_method"),
            meta: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AuthenticateRequest {
            method_id: AuthMethodId::new(""),
            meta: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_auth_method_agent_validation() {
        let valid_method = AuthMethodAgent {
            id: AuthMethodId::new("test_method"),
            name: "Test Method".to_string(),
            description: Some("A test authentication method".to_string()),
            meta: None,
        };
        assert!(valid_method.validate().is_ok());

        let invalid_method = AuthMethodAgent {
            id: AuthMethodId::new(""),
            name: "Test Method".to_string(),
            description: None,
            meta: None,
        };
        assert!(invalid_method.validate().is_err());

        let empty_name_method = AuthMethodAgent {
            id: AuthMethodId::new("test_method"),
            name: String::new(),
            description: None,
            meta: None,
        };
        assert!(empty_name_method.validate().is_err());
    }

    #[cfg(feature = "unstable_elicitation")]
    #[test]
    fn test_elicitation_id_validation() {
        let valid_id = ElicitationId::new("registration_form");
        assert!(valid_id.validate().is_ok());

        let valid_id_with_dots = ElicitationId::new("com.example.registration");
        assert!(valid_id_with_dots.validate().is_ok());

        let invalid_empty_id = ElicitationId::new("");
        assert!(invalid_empty_id.validate().is_err());

        let invalid_id_with_spaces = ElicitationId::new("registration form");
        assert!(invalid_id_with_spaces.validate().is_err());
    }

    #[cfg(feature = "unstable_elicitation")]
    #[test]
    fn test_elicitation_url_mode_validation() {
        use crate::v2::{ElicitationScope, ElicitationSessionScope};

        let valid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "https://example.com/register".to_string(),
        };
        assert!(valid_url_mode.validate().is_ok());

        let invalid_url_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new("test_elicitation"),
            url: "not_a_url".to_string(),
        };
        assert!(invalid_url_mode.validate().is_err());

        let invalid_id_mode = ElicitationUrlMode {
            scope: ElicitationScope::Session(ElicitationSessionScope {
                session_id: crate::v2::SessionId::new("test_session"),
                tool_call_id: None,
            }),
            elicitation_id: ElicitationId::new(""),
            url: "https://example.com/register".to_string(),
        };
        assert!(invalid_id_mode.validate().is_err());
    }
}
