use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum SVGErrorKind {
    ConnectionError(String),
    ManycoreMismatch(String),
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
        match self.error_kind {
            SVGErrorKind::ConnectionError(reason) => write!(f, "Connection Error: {reason}"),
            SVGErrorKind::ManycoreMismatch(reason) => {
                write!(f, "Mismatch with ManyCore System: {reason}")
            }
        }
    }
}

impl Error for SVGError {}
