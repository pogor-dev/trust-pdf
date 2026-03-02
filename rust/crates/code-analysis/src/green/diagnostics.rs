use std::{
    hash::BuildHasherDefault,
    sync::{Mutex, OnceLock},
};

use rustc_hash::FxHasher;

use crate::GreenDiagnostic;

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
type GreenDiagnosticTable = HashMap<usize, Vec<GreenDiagnostic>>;

pub(crate) fn green_diagnostics_table() -> &'static Mutex<GreenDiagnosticTable> {
    static TABLE: OnceLock<Mutex<GreenDiagnosticTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::default()))
}
