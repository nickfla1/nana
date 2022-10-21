use std::fmt::{Debug, Display};

use crate::commands::install::InstallCommand;

pub type NanaResult<T> = Result<T, NanaError>;

#[derive(Debug, Clone)]
pub enum NanaError {
    IO(String),
    Lock(LockError),
    Network(String),
    Package(PackageError),
    Runtime(String),
}

#[derive(Debug, Clone)]
pub enum PackageError {
    Invalid(validator::ValidationErrors),
    NotFound,
    ScriptNotFound(String),
}

#[derive(Debug, Clone)]
pub enum LockError {
    NotFound,
}

impl std::error::Error for NanaError {}

impl Display for NanaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "IO error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            Self::Package(error) => std::fmt::Display::fmt(&error, f),
            Self::Lock(error) => std::fmt::Display::fmt(&error, f),
        }
    }
}

impl Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Package not found"),
            Self::ScriptNotFound(name) => write!(f, "Could not find script '{}' in package", name),
            Self::Invalid(e) => write!(f, "Package is in an invalid format. Errors: {}", e),
        }
    }
}

impl Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Lock file  not found"),
        }
    }
}

impl From<reqwest::Error> for NanaError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<reqwest_middleware::Error> for NanaError {
    fn from(e: reqwest_middleware::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<semver_rs::Error> for NanaError {
    fn from(e: semver_rs::Error) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<std::io::Error> for NanaError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e.to_string())
    }
}

impl From<std::path::StripPrefixError> for NanaError {
    fn from(e: std::path::StripPrefixError) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<indicatif::style::TemplateError> for NanaError {
    fn from(e: indicatif::style::TemplateError) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<package_json_schema::Error> for NanaError {
    fn from(e: package_json_schema::Error) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<validator::ValidationErrors> for NanaError {
    fn from(e: validator::ValidationErrors) -> Self {
        Self::Package(PackageError::Invalid(e))
    }
}

impl<T> From<std::sync::PoisonError<T>> for NanaError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<serde_yaml::Error> for NanaError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<std::str::Utf8Error> for NanaError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<tokio::task::JoinError> for NanaError {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Runtime(e.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<InstallCommand>> for NanaError {
    fn from(e: tokio::sync::mpsc::error::SendError<InstallCommand>) -> Self {
        Self::Runtime(e.to_string())
    }
}
