// #![allow(dead_code)]

// use crate::internal::markers::MarkerKind;
// use crate::model::token::Span;

// #[derive(Debug, Clone, Default)]
// pub(crate) struct Document {
//     pub children: Vec<Node>,
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub(crate) enum Node {
//     Container(ContainerNode),
//     Chapter {
//         marker_span: Span,
//         number_span: Option<Span>,
//     },
//     Verse {
//         marker_span: Span,
//         number_span: Option<Span>,
//     },
//     Milestone {
//         marker: String,
//         marker_span: Span,
//         attribute_spans: Vec<Span>,
//         end_span: Option<Span>,
//         closed: bool,
//     },
//     Leaf {
//         kind: LeafKind,
//         span: Span,
//     },
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub(crate) struct ContainerNode {
//     pub kind: ContainerKind,
//     pub marker: String,
//     pub marker_span: Span,
//     pub close_span: Option<Span>,
//     pub special_span: Option<Span>,
//     pub attribute_spans: Vec<Span>,
//     pub children: Vec<Node>,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub(crate) enum ContainerKind {
//     Book,
//     Paragraph,
//     Character,
//     Note,
//     Figure,
//     Sidebar,
//     Periph,
//     TableRow,
//     TableCell,
//     Header,
//     Meta,
//     Unknown,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub(crate) enum LeafKind {
//     Text,
//     Whitespace,
//     Newline,
//     OptBreak,
//     Attributes,
// }

// impl ContainerKind {
//     pub fn from_marker_kind(kind: MarkerKind, marker: &str) -> Self {
//         match kind {
//             MarkerKind::Paragraph => Self::Paragraph,
//             MarkerKind::Character => Self::Character,
//             MarkerKind::Note => Self::Note,
//             MarkerKind::Figure => Self::Figure,
//             MarkerKind::SidebarStart => Self::Sidebar,
//             MarkerKind::Periph => Self::Periph,
//             MarkerKind::TableRow => Self::TableRow,
//             MarkerKind::TableCell => Self::TableCell,
//             MarkerKind::Header if marker == "id" => Self::Book,
//             MarkerKind::Header => Self::Header,
//             MarkerKind::Meta => Self::Meta,
//             MarkerKind::Unknown => Self::Unknown,
//             _ => Self::Unknown,
//         }
//     }
// }
