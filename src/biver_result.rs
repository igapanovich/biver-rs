use std::fmt::{Debug, Display, Formatter};

pub type BiverResult<T> = Result<T, BiverError>;

#[derive(Debug)]
pub struct BiverError {
    pub error_message: String,
    pub severity: BiverErrorSeverity,
}

#[derive(Debug)]
pub enum BiverErrorSeverity {
    Error,
    Warning,
}

impl Display for BiverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            BiverErrorSeverity::Error => "ERROR",
            BiverErrorSeverity::Warning => "WARNING",
        };

        write!(f, "{}: {}", severity, self.error_message)
    }
}

impl From<eframe::Error> for BiverError {
    fn from(value: eframe::Error) -> Self {
        Self {
            error_message: format!("eframe/egui failure: {}", value),
            severity: BiverErrorSeverity::Error,
        }
    }
}

impl From<image::ImageError> for BiverError {
    fn from(value: image::ImageError) -> Self {
        Self {
            error_message: format!("image failure: {}", value),
            severity: BiverErrorSeverity::Error,
        }
    }
}

impl From<serde_json::Error> for BiverError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            error_message: format!("serde_json failure: {}", value),
            severity: BiverErrorSeverity::Error,
        }
    }
}

impl From<std::io::Error> for BiverError {
    fn from(value: std::io::Error) -> Self {
        Self {
            error_message: format!("io failure: {}", value),
            severity: BiverErrorSeverity::Error,
        }
    }
}

pub fn error<T>(message: impl Into<String>) -> BiverResult<T> {
    Err(BiverError {
        error_message: message.into(),
        severity: BiverErrorSeverity::Error,
    }
    .into())
}

pub fn warning<T>(message: impl Into<String>) -> BiverResult<T> {
    Err(BiverError {
        error_message: message.into(),
        severity: BiverErrorSeverity::Warning,
    }
    .into())
}
