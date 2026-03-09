use crate::model::token::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryCode {
    MissingChapterNumber,
    MissingVerseNumber,
    MissingMilestoneSelfClose,
    ImplicitlyClosedMarker,
    StrayCloseMarker,
    MisnestedCloseMarker,
    UnclosedNote,
    UnclosedMarkerAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryPayload {
    Marker { marker: String },
    Close { open: String, close: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseRecovery {
    pub code: RecoveryCode,
    pub span: Span,
    pub related_span: Option<Span>,
    pub payload: Option<RecoveryPayload>,
}

impl ParseRecovery {
    pub(crate) fn marker(code: RecoveryCode, span: Span, marker: impl Into<String>) -> Self {
        Self {
            code,
            span,
            related_span: None,
            payload: Some(RecoveryPayload::Marker {
                marker: marker.into(),
            }),
        }
    }

    pub(crate) fn marker_with_related(
        code: RecoveryCode,
        span: Span,
        related_span: Option<Span>,
        marker: impl Into<String>,
    ) -> Self {
        Self {
            code,
            span,
            related_span,
            payload: Some(RecoveryPayload::Marker {
                marker: marker.into(),
            }),
        }
    }

    pub(crate) fn close(
        code: RecoveryCode,
        span: Span,
        related_span: Option<Span>,
        open: impl Into<String>,
        close: impl Into<String>,
    ) -> Self {
        Self {
            code,
            span,
            related_span,
            payload: Some(RecoveryPayload::Close {
                open: open.into(),
                close: close.into(),
            }),
        }
    }
}

impl RecoveryCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingChapterNumber => "missing-chapter-number",
            Self::MissingVerseNumber => "missing-verse-number",
            Self::MissingMilestoneSelfClose => "missing-milestone-self-close",
            Self::ImplicitlyClosedMarker => "implicitly-closed-marker",
            Self::StrayCloseMarker => "stray-close-marker",
            Self::MisnestedCloseMarker => "misnested-close-marker",
            Self::UnclosedNote => "unclosed-note",
            Self::UnclosedMarkerAtEof => "unclosed-marker-at-eof",
        }
    }
}
