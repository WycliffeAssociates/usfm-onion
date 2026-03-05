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

pub fn default_attribute(marker: &str) -> Option<&'static str> {
    match marker {
        "w" => Some("lemma"),
        "rb" => Some("gloss"),
        "jmp" => Some("link-href"),
        "xt" => Some("link-href"),
        "ref" => Some("loc"),
        "fig" => Some("src"),
        _ => {
            let base = marker
                .strip_suffix("-s")
                .or_else(|| marker.strip_suffix("-e"));
            if let Some(base) = base {
                let base = base.trim_end_matches(|ch: char| ch.is_ascii_digit());
                if base == "qt" {
                    return Some("who");
                }
            }
            None
        }
    }
}

pub fn lookup_marker(name: &str) -> MarkerInfo {
    if name.starts_with('z') {
        return MarkerInfo::new(MarkerKind::Unknown);
    }

    if name.ends_with("-s") {
        return MarkerInfo::new(MarkerKind::MilestoneStart);
    }
    if name.ends_with("-e") {
        return MarkerInfo::new(MarkerKind::MilestoneEnd);
    }

    if is_table_cell_marker(name) {
        return MarkerInfo::new(MarkerKind::TableCell);
    }

    match name {
        "id" | "usfm" | "ide" | "h" | "h1" | "h2" | "h3" | "toc1" | "toc2" | "toc3" | "toca1"
        | "toca2" | "toca3" | "mt" | "mt1" | "mt2" | "mt3" | "mt4" | "mte" | "mte1" | "mte2"
        | "imt" | "imt1" | "imt2" | "imt3" | "imt4" | "imte" | "imte1" | "imte2" | "is" | "is1"
        | "is2" | "is3" | "cl" | "cp" | "cd" => MarkerInfo::new(MarkerKind::Header),

        "p" | "m" | "po" | "pr" | "cls" | "pmo" | "pm" | "pmc" | "pmr" | "pi" | "pi1" | "pi2"
        | "pi3" | "mi" | "nb" | "pc" | "ph" | "ph1" | "ph2" | "ph3" | "b" | "pb" | "q" | "q1"
        | "q2" | "q3" | "q4" | "qr" | "qc" | "qa" | "qm" | "qm1" | "qm2" | "qm3" | "qd" | "lh"
        | "li" | "li1" | "li2" | "li3" | "li4" | "lf" | "lim" | "lim1" | "lim2" | "lim3" | "ms"
        | "ms1" | "ms2" | "ms3" | "mr" | "s" | "s1" | "s2" | "s3" | "s4" | "s5" | "sr" | "r"
        | "sp" | "sd" | "sd1" | "sd2" | "sd3" | "sd4" | "d" | "ip" | "ipi" | "im" | "imi"
        | "ipq" | "imq" | "ipr" | "ib" | "iq" | "iq1" | "iq2" | "iq3" | "iex" | "iot" | "io"
        | "io1" | "io2" | "io3" | "io4" | "ili" | "ili1" | "ili2" | "ie" | "lit" => {
            MarkerInfo::new(MarkerKind::Paragraph)
        }

        "f" | "fe" | "x" | "ef" | "ex" => MarkerInfo::new(MarkerKind::Note),

        "fr" | "ft" | "fk" | "fq" | "fqa" | "fl" | "fw" | "fp" | "fv" | "fdc" | "xop" | "xot"
        | "xnt" | "xdc" | "xo" | "xt" | "xta" | "xk" | "xq" => {
            MarkerInfo::note_sub(MarkerKind::Character)
        }

        "add" | "bk" | "dc" | "ior" | "iqt" | "k" | "lik" | "liv1" | "litl" | "nd" | "ord"
        | "pn" | "png" | "qs" | "qac" | "qt" | "sig" | "sls" | "tl" | "wj" | "em" | "bd"
        | "bdit" | "it" | "no" | "sc" | "sup" | "rb" | "pro" | "w" | "wg" | "wh" | "wa"
        | "jmp" | "ref" | "rq" | "vp" | "ca" | "va" => MarkerInfo::new(MarkerKind::Character),

        "fig" => MarkerInfo::new(MarkerKind::Figure),
        "esb" => MarkerInfo::new(MarkerKind::SidebarStart),
        "esbe" => MarkerInfo::new(MarkerKind::SidebarEnd),
        "periph" => MarkerInfo::new(MarkerKind::Periph),
        "rem" | "sts" | "restore" | "cat" => MarkerInfo::new(MarkerKind::Meta),
        "tr" => MarkerInfo::new(MarkerKind::TableRow),
        "c" => MarkerInfo::new(MarkerKind::Chapter),
        "v" => MarkerInfo::new(MarkerKind::Verse),
        "ts" => MarkerInfo::new(MarkerKind::MilestoneStart),
        _ => MarkerInfo::new(MarkerKind::Unknown),
    }
}

fn is_table_cell_marker(name: &str) -> bool {
    let prefixes = ["th", "thr", "tc", "tcr"];
    prefixes.iter().any(|prefix| {
        name.strip_prefix(prefix)
            .map(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
            })
            .unwrap_or(false)
    })
}
