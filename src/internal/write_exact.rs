use crate::parse::ParseHandle;

pub fn write_exact(handle: &ParseHandle) -> &str {
    handle.source()
}
