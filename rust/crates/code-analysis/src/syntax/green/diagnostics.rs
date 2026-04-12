use std::{
    hash::BuildHasherDefault,
    sync::{Mutex, MutexGuard, OnceLock},
};

use rustc_hash::FxHasher;

use crate::GreenDiagnostic;

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
type GreenDiagnosticTable = HashMap<usize, Vec<GreenDiagnostic>>;

pub(crate) fn green_diagnostics_table() -> &'static Mutex<GreenDiagnosticTable> {
    static TABLE: OnceLock<Mutex<GreenDiagnosticTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::default()))
}

#[inline]
fn lock_diagnostics_table() -> MutexGuard<'static, GreenDiagnosticTable> {
    match green_diagnostics_table().lock() {
        Ok(table_guard) => table_guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[inline]
pub(crate) fn get_diagnostics(key: usize) -> Option<Vec<GreenDiagnostic>> {
    lock_diagnostics_table().get(&key).cloned()
}

#[inline]
pub(crate) fn insert_diagnostics(key: usize, diagnostics: Vec<GreenDiagnostic>) {
    lock_diagnostics_table().insert(key, diagnostics);
}

#[inline]
pub(crate) fn remove_diagnostics(key: usize) {
    // Keep value destruction outside the mutex critical section.
    // Root cause of the hang: removed diagnostics may drop green values whose
    // Drop paths touch this table again; dropping while locked can re-enter the
    // same mutex and deadlock.
    let removed = {
        let mut diagnostics_table_guard = lock_diagnostics_table();
        diagnostics_table_guard.remove(&key)
    };

    drop(removed);
}

#[inline]
#[allow(dead_code)]
pub(crate) fn contains_diagnostics(key: usize) -> bool {
    lock_diagnostics_table().contains_key(&key)
}
