/// Captures which action failed during dry-run validation.
#[derive(Debug)]
pub struct ValidationStepError {
    pub action_id: String,
    pub action_type: String,
    pub error: crate::models::Error,
}

impl std::fmt::Display for ValidationStepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation failed at step '{}' ({}): {}",
            self.action_id, self.action_type, self.error
        )
    }
}
