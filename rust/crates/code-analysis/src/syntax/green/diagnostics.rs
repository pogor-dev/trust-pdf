use std::{
    hash::BuildHasherDefault,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use rustc_hash::FxHasher;

use crate::GreenDiagnostic;

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
type GreenDiagnosticTable = HashMap<usize, Vec<GreenDiagnostic>>;

pub(crate) fn green_diagnostics_table() -> &'static RwLock<GreenDiagnosticTable> {
    static TABLE: OnceLock<RwLock<GreenDiagnosticTable>> = OnceLock::new();
    TABLE.get_or_init(|| RwLock::new(HashMap::default()))
}

#[inline]
fn read_diagnostics_table() -> RwLockReadGuard<'static, GreenDiagnosticTable> {
    match green_diagnostics_table().read() {
        Ok(table_guard) => table_guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[inline]
fn write_diagnostics_table() -> RwLockWriteGuard<'static, GreenDiagnosticTable> {
    match green_diagnostics_table().write() {
        Ok(table_guard) => table_guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[inline]
pub(crate) fn get_diagnostics(key: usize) -> Option<Vec<GreenDiagnostic>> {
    read_diagnostics_table().get(&key).cloned()
}

#[inline]
pub(crate) fn insert_diagnostics(key: usize, diagnostics: Vec<GreenDiagnostic>) {
    write_diagnostics_table().insert(key, diagnostics);
}

#[inline]
pub(crate) fn remove_diagnostics(key: usize) {
    // Keep value destruction outside the write lock critical section.
    // Root cause of the hang: removed diagnostics may drop green values whose
    // Drop paths touch this table again; dropping while locked can re-enter the
    // same write lock and deadlock.
    let removed = write_diagnostics_table().remove(&key);
    drop(removed);
}

#[inline]
#[allow(dead_code)]
pub(crate) fn contains_diagnostics(key: usize) -> bool {
    read_diagnostics_table().contains_key(&key)
}
