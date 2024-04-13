use std::{error::Error, fmt::Display, num::TryFromIntError};

use manycore_parser::ManycoreError;
use quick_xml::DeError;

#[derive(Debug)]
pub enum SVGErrorKind {
    ConnectionError(String),
    ManycoreMismatch(String),
    ManycoreError(String),
    SerialisationError(String),
    DataConversionError(String),
}

#[derive(Debug)]
pub struct SVGError {
    error_kind: SVGErrorKind,
}

impl SVGError {
    pub fn new(error_kind: SVGErrorKind) -> Self {
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
