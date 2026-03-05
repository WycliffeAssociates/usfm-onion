use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryPayload {
    Marker { marker: String },
    Close { open: String, close: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
