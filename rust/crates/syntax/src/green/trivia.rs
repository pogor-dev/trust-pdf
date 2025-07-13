use std::{
    borrow::Borrow,
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    arc::{arc_main::Arc, thin_arc::ThinArc},
    green::{
        GreenTriviaRepr, GreenTriviaReprThin, trivia_child::GreenTriviaChild,
        trivia_data::GreenTriviaData, trivia_head::GreenTriviaHead,
    },
};

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenTrivia {
    ptr: ThinArc<GreenTriviaHead, GreenTriviaChild>,
}

impl GreenTrivia {
    #[inline]
    pub(crate) fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = GreenTriviaChild>,
        I::IntoIter: ExactSizeIterator,
    {
        let data = ThinArc::from_header_and_iter(GreenTriviaHead::new(), pieces.into_iter());

        GreenTrivia { ptr: data }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenTrivia) -> ptr::NonNull<GreenTriviaData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenTriviaData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTriviaData>) -> GreenTrivia {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const GreenTriviaReprThin);
            mem::transmute::<Arc<GreenTriviaReprThin>, ThinArc<GreenTriviaHead, GreenTriviaChild>>(
                arc,
            )
        };
        GreenTrivia { ptr: arc }
    }
}

impl Borrow<GreenTriviaData> for GreenTrivia {
    #[inline]
    fn borrow(&self) -> &GreenTriviaData {
        self
    }
}

impl ops::Deref for GreenTrivia {
    type Target = GreenTriviaData;

    #[inline]
    fn deref(&self) -> &GreenTriviaData {
        unsafe {
            // Step 1: Get full memory representation
            let repr: &GreenTriviaRepr = &self.ptr;

            // Step 2: Normalize layout (remove metadata)
            //   &*(ptr as *const A as *const B) pattern:
            //   - Convert to raw pointer
            //   - Reinterpret type
            //   - Dereference and re-borrow
            let repr: &GreenTriviaReprThin =
                &*(repr as *const GreenTriviaRepr as *const GreenTriviaReprThin);

            // Step 3: Final API view (same bytes, API methods)
            mem::transmute::<&GreenTriviaReprThin, &GreenTriviaData>(repr)
        }
    }
}

impl std::fmt::Debug for GreenTrivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the Deref trait to access GreenTriviaData and its Debug impl
        (**self).fmt(f)
    }
}
