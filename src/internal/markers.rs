use crate::internal::marker_defs::lookup_marker_def;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarkerKind {
    Paragraph,
    Note,
    Character,
    Header,
    Chapter,
    Verse,
    MilestoneStart,
    MilestoneEnd,
    SidebarStart,
    SidebarEnd,
    Figure,
    Meta,
    Periph,
    TableRow,
    TableCell,
    Unknown,
}

pub struct MarkerInfo {
    pub kind: MarkerKind,
    #[allow(dead_code)]
    pub valid_in_note: bool,
}

impl MarkerInfo {
    const fn new(kind: MarkerKind) -> Self {
        Self {
            kind,
            valid_in_note: false,
        }
    }

    const fn note_sub(kind: MarkerKind) -> Self {
        Self {
            kind,
            valid_in_note: true,
        }
    }
}

pub fn lookup_marker(name: &str) -> MarkerInfo {
    if name.starts_with('z') {
        return MarkerInfo::new(MarkerKind::Unknown);
    }

    if let Some(def) = lookup_marker_def(name) {
        let kind = def.kind.to_marker_kind(name);
        if def.note_subkind.is_some() {
            return MarkerInfo::note_sub(kind);
        }
        return MarkerInfo::new(kind);
    }

    if is_table_cell_marker(name) {
        return MarkerInfo::new(MarkerKind::TableCell);
    }

    MarkerInfo::new(MarkerKind::Unknown)
}

fn is_table_cell_marker(name: &str) -> bool {
    let prefixes = ["th", "thr", "thc", "tc", "tcr", "tcc"];
    prefixes.iter().any(|prefix| {
        name.strip_prefix(prefix)
            .map(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
            })
            .unwrap_or(false)
    })
}
