/// Errors produced by the `forge-core` crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// A trust score value was outside the valid range `[0.0, 1.0]`.
    #[error("invalid trust score {value}: must be in [0.0, 1.0]")]
    InvalidTrustScore { value: f64 },

    /// A block ID could not be parsed or validated.
    #[error("invalid block id: {reason}")]
    InvalidBlockId { reason: String },

    /// A block manifest field failed validation.
    #[error("manifest validation failed for field '{field}': {reason}")]
    ManifestValidation { field: String, reason: String },
}
