use std::{error::Error, fmt::Display, num::TryFromIntError};

use manycore_parser::ManycoreError;
use quick_xml::DeError;

#[cfg(doc)]
use crate::SVG;

/// Enum to wrap possible errors that might arise when generating/updating an [`SVG`].
///
/// The string contained in each variant is a user friendly explanation of the error (or a call to `to_string()` on the error).
#[derive(Debug)]
pub enum SVGErrorKind {
    /// Related to router connections.
    ConnectionError(String),
    /// Something expected by the [`SVG`] is not actually in the ManyCore system.
    ManycoreMismatch(String),
    /// A manycore_parser error that we are bubbling up to the user.
    ManycoreError(String),
    /// A serde error that we are bubbling up to the user.
    SerialisationError(String),
    /// A conversion between types failed. You probably supplied an input outside of this library's scope.
    DataConversionError(String),
    /// A generic [`SVG`] generation error.
    GenerationError(String),
}

/// A generic error container used to keep results consistent within the library.
#[derive(Debug)]
pub struct SVGError {
    error_kind: SVGErrorKind,
}

impl SVGError {
    /// Instantiates a new [`SVGError`] instance.
    pub(crate) fn new(error_kind: SVGErrorKind) -> Self {
        Self { error_kind }
    }
}

impl Display for SVGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_kind {
            SVGErrorKind::ConnectionError(reason) => write!(f, "Connection Error: {reason}"),
            SVGErrorKind::ManycoreMismatch(reason) => {
                write!(f, "Mismatch with ManyCore System: {reason}")
            }
            SVGErrorKind::ManycoreError(reason) => write!(f, "ManyCore Error: {reason}"),
            SVGErrorKind::SerialisationError(reason) => write!(f, "Serialisation Error: {reason}"),
            SVGErrorKind::DataConversionError(reason) => {
                write!(f, "Data Conversion Error: {reason}")
            }
            SVGErrorKind::GenerationError(reason) => write!(f, "Generation Error: {reason}"),
        }
    }
}

impl Error for SVGError {}

impl From<ManycoreError> for SVGError {
    fn from(error: ManycoreError) -> Self {
        Self {
            error_kind: SVGErrorKind::ManycoreError(format!("{error}")),
        }
    }
}

impl From<DeError> for SVGError {
    fn from(error: DeError) -> Self {
        Self {
            error_kind: SVGErrorKind::SerialisationError(error.to_string()),
        }
    }
}

impl From<TryFromIntError> for SVGError {
    fn from(error: TryFromIntError) -> Self {
        Self {
            error_kind: SVGErrorKind::DataConversionError(error.to_string()),
        }
    }
}
