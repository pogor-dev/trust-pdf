/// Generates the mechanical boilerplate impls shared by all green tree element types.
///
/// Each green element follows a mix of Roslyn and Rust analyzer pattern of `Data` (borrowed view) /
/// `Owned` (ref-counted handle) pairs backed by a `ThinArc<Head, Tail>`.
/// This macro generates the repetitive plumbing that is identical across all
/// green element types:
///
/// - `type Repr` / `type ReprThin` aliases
/// - `ToOwned for Data`
/// - `Borrow<Data> for Owned`
/// - `Display` / `Debug` delegation from `Owned` to `Data`
/// - `into_raw` / `from_raw` pointer conversion methods
/// - `Deref` from `Owned` to `Data`
macro_rules! impl_green_boilerplate {
    // Non-generic variant
    ($head:ident, $data:ident, $owned:ident, $tail:ty) => {
        type Repr = HeaderSlice<$head, [$tail]>;
        type ReprThin = HeaderSlice<$head, [$tail; 0]>;

        impl ToOwned for $data {
            type Owned = $owned;

            #[inline]
            fn to_owned(&self) -> $owned {
                let green = unsafe { $owned::from_raw(ptr::NonNull::from(self)) };
                let green = ManuallyDrop::new(green);
                $owned::clone(&green)
            }
        }

        impl Borrow<$data> for $owned {
            #[inline]
            fn borrow(&self) -> &$data {
                self
            }
        }

        impl fmt::Display for $owned {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let data: &$data = self;
                fmt::Display::fmt(data, f)
            }
        }

        impl fmt::Debug for $owned {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let data: &$data = self;
                fmt::Debug::fmt(data, f)
            }
        }

        impl $owned {
            /// Consumes the handle and returns a raw non-null pointer to the data.
            #[inline]
            pub(crate) fn into_raw(this: $owned) -> ptr::NonNull<$data> {
                let green = ManuallyDrop::new(this);
                let green: &$data = &green;
                ptr::NonNull::from(green)
            }

            /// Reconstructs an owned handle from a raw pointer.
            ///
            /// # Safety
            ///
            /// The raw pointer must have been produced by `into_raw` and not yet
            /// consumed. The underlying `Arc` allocation must still be live.
            #[inline]
            pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<$data>) -> $owned {
                let arc = unsafe {
                    let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
                    mem::transmute::<Arc<ReprThin>, ThinArc<$head, $tail>>(arc)
                };
                $owned { ptr: arc }
            }

            #[inline]
            pub(crate) fn diagnostics(&self) -> Option<Vec<crate::GreenDiagnostic>> {
                use crate::syntax::green::diagnostics;

                diagnostics::get_diagnostics(self.diagnostics_key())
            }

            #[inline]
            fn clear_diagnostics(&self) {
                use crate::syntax::green::diagnostics;

                diagnostics::remove_diagnostics(self.diagnostics_key());
            }

            #[inline]
            fn diagnostics_key(&self) -> usize {
                let data: &$data = self;
                data as *const $data as usize
            }
        }

        impl Drop for $owned {
            #[inline]
            fn drop(&mut self) {
                // Clear side-table diagnostics only for the final owner.
                // This avoids duplicate removals while cloned green handles are
                // still alive and keeps diagnostics lifetime tied to green data.
                let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
                if should_clear {
                    self.clear_diagnostics();
                }
            }
        }

        impl ops::Deref for $owned {
            type Target = $data;

            #[inline]
            fn deref(&self) -> &$data {
                unsafe {
                    let repr: &Repr = &*self.ptr;
                    let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
                    mem::transmute::<&ReprThin, &$data>(repr)
                }
            }
        }
    };

    // Generic variant (single type parameter, Clone bound on ToOwned)
    (generic $head:ident, $data:ident, $owned:ident, $tail:ty) => {
        type Repr<T> = HeaderSlice<$head<T>, [$tail]>;
        type ReprThin<T> = HeaderSlice<$head<T>, [$tail; 0]>;

        impl<T: Clone> ToOwned for $data<T> {
            type Owned = $owned<T>;

            #[inline]
            fn to_owned(&self) -> $owned<T> {
                let green = unsafe { $owned::from_raw(ptr::NonNull::from(self)) };
                let green = ManuallyDrop::new(green);
                $owned::<T>::clone(&green)
            }
        }

        impl<T> Borrow<$data<T>> for $owned<T> {
            #[inline]
            fn borrow(&self) -> &$data<T> {
                self
            }
        }

        impl<T> fmt::Display for $owned<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let data: &$data<T> = self;
                fmt::Display::fmt(data, f)
            }
        }

        impl<T> fmt::Debug for $owned<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let data: &$data<T> = self;
                fmt::Debug::fmt(data, f)
            }
        }

        impl<T> $owned<T> {
            /// Consumes the handle and returns a raw non-null pointer to the data.
            #[inline]
            pub(crate) fn into_raw(this: $owned<T>) -> ptr::NonNull<$data<T>> {
                let green = ManuallyDrop::new(this);
                let green: &$data<T> = &green;
                ptr::NonNull::from(green)
            }

            /// Reconstructs an owned handle from a raw pointer.
            ///
            /// # Safety
            ///
            /// The raw pointer must have been produced by `into_raw` and not yet
            /// consumed. The underlying `Arc` allocation must still be live.
            #[inline]
            pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<$data<T>>) -> $owned<T> {
                let arc = unsafe {
                    let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin<T>);
                    mem::transmute::<Arc<ReprThin<T>>, ThinArc<$head<T>, $tail>>(arc)
                };
                $owned { ptr: arc }
            }

            #[inline]
            pub(crate) fn diagnostics(&self) -> Option<Vec<crate::GreenDiagnostic>> {
                use crate::syntax::green::diagnostics;

                diagnostics::get_diagnostics(self.diagnostics_key())
            }

            #[inline]
            fn clear_diagnostics(&self) {
                use crate::syntax::green::diagnostics;

                diagnostics::remove_diagnostics(self.diagnostics_key());
            }

            #[inline]
            fn diagnostics_key(&self) -> usize {
                let data: &$data<T> = self;
                data as *const $data<T> as usize
            }
        }

        impl<T> Drop for $owned<T> {
            #[inline]
            fn drop(&mut self) {
                // Same rationale as non-generic variant: remove diagnostics on
                // last-owner drop so cleanup is deterministic and race-free.
                let should_clear = self.ptr.with_arc(|arc| arc.is_unique());
                if should_clear {
                    self.clear_diagnostics();
                }
            }
        }

        impl<T> ops::Deref for $owned<T> {
            type Target = $data<T>;

            #[inline]
            fn deref(&self) -> &$data<T> {
                unsafe {
                    let repr: &Repr<T> = &*self.ptr;
                    let repr: &ReprThin<T> = &*(repr as *const Repr<T> as *const ReprThin<T>);
                    mem::transmute::<&ReprThin<T>, &$data<T>>(repr)
                }
            }
        }
    };
}

/// Dispatches over all `TokenType` variants.
///
/// All variants evaluate the same expression:
/// `match_token_type!(self, t => t.kind())`
macro_rules! match_token_type {
    // Uniform: every variant uses the same expression.
    ($self:expr, $t:ident => $expr:expr) => {
        match $self {
            Self::Token($t) => $expr,
            Self::TokenWithTrivia($t) => $expr,
            Self::TokenWithIntValue($t) => $expr,
            Self::TokenWithFloatValue($t) => $expr,
            Self::TokenWithStringValue($t) => $expr,
            Self::TokenWithTrailingTrivia($t) => $expr,
            Self::TokenWithIntValueAndTrivia($t) => $expr,
            Self::TokenWithFloatValueAndTrivia($t) => $expr,
            Self::TokenWithStringValueAndTrivia($t) => $expr,
            Self::TokenWithIntValueAndTrailingTrivia($t) => $expr,
            Self::TokenWithFloatValueAndTrailingTrivia($t) => $expr,
            Self::TokenWithStringValueAndTrailingTrivia($t) => $expr,
        }
    };
}

/// Generates `into_*` / `as_*` accessor pairs for `TokenType` variants.
///
/// Each pair extracts one variant and returns `None` for all others.
macro_rules! impl_token_type_accessors {
    ($( ($into_fn:ident, $as_fn:ident, $variant:ident, $T:ident) ),* $(,)?) => {
        $(
            pub fn $into_fn(self) -> Option<$T> {
                match self {
                    Self::$variant(v) => Some(v),
                    _ => None,
                }
            }

            pub fn $as_fn(&self) -> Option<&$T> {
                match self {
                    Self::$variant(v) => Some(v),
                    _ => None,
                }
            }
        )*
    };
}

/// Generates `From<ConcreteToken> for GreenTokenElement` impls for all variants.
macro_rules! impl_from_token_variant {
    ($( $concrete:ty => $variant:ident ),* $(,)?) => {
        $(
            impl From<$concrete> for GreenTokenElement {
                fn from(token: $concrete) -> GreenTokenElement {
                    GreenTokenElement::$variant(token)
                }
            }
        )*
    };
}

