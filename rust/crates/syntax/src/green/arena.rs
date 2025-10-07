use bumpalo::Bump;
use hashbrown::HashMap;

use crate::DiagnosticInfo;

pub(crate) struct GreenTree {
    arena: Bump,
    diagnostics: HashMap<*const u8, Vec<DiagnosticInfo>>,
}
