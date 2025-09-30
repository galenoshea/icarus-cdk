//! Error types for procedural macros following `rust_best_practices.md` patterns.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use thiserror::Error;

/// Error type for macro compilation failures.
///
/// Provides structured error handling for procedural macro compilation
/// with helpful error messages and span information.
#[derive(Error, Debug)]
pub(crate) enum MacroError {
    /// Syn parsing error
    #[error("Parse error: {0}")]
    Parse(#[from] syn::Error),

    /// Invalid function signature
    #[error("Invalid function signature: {message}")]
    InvalidSignature { message: String, span: Option<Span> },

    /// Unsupported feature
    #[error("Unsupported feature: {feature} - {reason}")]
    UnsupportedFeature {
        feature: String,
        reason: String,
        span: Option<Span>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String, span: Option<Span> },

    /// Code generation error
    #[error("Code generation error: {message}")]
    #[allow(dead_code)]
    CodeGeneration { message: String, span: Option<Span> },
}

impl MacroError {
    /// Creates an invalid signature error.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn invalid_signature(message: impl Into<String>) -> Self {
        Self::InvalidSignature {
            message: message.into(),
            span: None,
        }
    }

    /// Creates an invalid signature error with span information.
    #[inline]
    pub(crate) fn invalid_signature_spanned(message: impl Into<String>, span: Span) -> Self {
        Self::InvalidSignature {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates an unsupported feature error.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn unsupported_feature(
        feature: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
            reason: reason.into(),
            span: None,
        }
    }

    /// Creates an unsupported feature error with span information.
    #[inline]
    pub(crate) fn unsupported_feature_spanned(
        feature: impl Into<String>,
        reason: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::UnsupportedFeature {
            feature: feature.into(),
            reason: reason.into(),
            span: Some(span),
        }
    }

    /// Creates a configuration error.
    #[inline]
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            span: None,
        }
    }

    /// Creates a configuration error with span information.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn configuration_spanned(message: impl Into<String>, span: Span) -> Self {
        Self::Configuration {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a code generation error.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn code_generation(message: impl Into<String>) -> Self {
        Self::CodeGeneration {
            message: message.into(),
            span: None,
        }
    }

    /// Creates a code generation error with span information.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn code_generation_spanned(message: impl Into<String>, span: Span) -> Self {
        Self::CodeGeneration {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Converts the error into a compile error token stream.
    ///
    /// If span information is available, the error will point to the specific
    /// location in the source code for better IDE integration.
    #[inline]
    pub(crate) fn to_compile_error(&self) -> TokenStream {
        let message = self.to_string();

        // Use span information if available for precise error location
        let span = match self {
            Self::Parse(_) => None,
            Self::InvalidSignature { span, .. }
            | Self::UnsupportedFeature { span, .. }
            | Self::Configuration { span, .. }
            | Self::CodeGeneration { span, .. } => *span,
        };

        if let Some(span) = span {
            quote::quote_spanned! { span =>
                compile_error!(#message);
            }
        } else {
            quote! {
                compile_error!(#message);
            }
        }
    }
}

/// Result type for macro operations.
pub(crate) type MacroResult<T> = Result<T, MacroError>;
