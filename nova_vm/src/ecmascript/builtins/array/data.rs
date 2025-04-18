// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    ecmascript::types::{OrdinaryObject, Value},
    engine::context::{Bindable, NoGcScope},
    heap::{
        CompactionLists, HeapMarkAndSweep, WorkQueues,
        element_array::{ElementArrayKey, ElementArrays, ElementDescriptor, ElementsVector},
        indexes::ElementIndex,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct SealableElementsVector<'a> {
    pub(crate) elements_index: ElementIndex<'a>,
    pub(crate) cap: ElementArrayKey,
    pub(crate) len: u32,
    /// Array length property can be set to unwritable
    pub(crate) len_writable: bool,
}

impl<'a> SealableElementsVector<'a> {
    #[inline(always)]
    pub fn cap(&self) -> u32 {
        self.cap.cap()
    }

    pub(crate) fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.cap()
    }

    pub(crate) fn writable(&self) -> bool {
        self.len_writable
    }

    /// A sealable elements vector is simple if it contains no accessor descriptors.
    pub(crate) fn is_simple(&self, agent: &impl AsRef<ElementArrays>) -> bool {
        let elements_vector: ElementsVector = (*self).into();
        elements_vector.is_simple(agent)
    }

    /// A sealable elements vector is trivial if it contains no descriptors.
    pub(crate) fn is_trivial(&self, agent: &impl AsRef<ElementArrays>) -> bool {
        let elements_vector: ElementsVector = (*self).into();
        elements_vector.is_trivial(agent)
    }

    pub(crate) fn is_dense(&self, agent: &impl AsRef<ElementArrays>) -> bool {
        let elements_vector: ElementsVector = (*self).into();
        elements_vector.is_dense(agent)
    }

    pub(crate) fn from_elements_vector(elements: ElementsVector<'a>) -> Self {
        Self {
            elements_index: elements.elements_index,
            cap: elements.cap,
            len: elements.len,
            len_writable: true,
        }
    }

    pub fn reserve(&mut self, elements: &mut ElementArrays, new_len: u32) {
        let mut elements_vector: ElementsVector = (*self).into();
        elements_vector.reserve(elements, new_len);
        self.cap = elements_vector.cap;
        self.elements_index = elements_vector.elements_index;
    }

    pub fn push(
        &mut self,
        elements: &mut ElementArrays,
        value: Option<Value>,
        descriptor: Option<ElementDescriptor>,
    ) {
        let mut elements_vector: ElementsVector = (*self).into();
        elements_vector.push(elements, value, descriptor);
        self.cap = elements_vector.cap;
        self.len = elements_vector.len;
        self.elements_index = elements_vector.elements_index;
    }
}

impl Default for SealableElementsVector<'static> {
    fn default() -> Self {
        Self {
            elements_index: ElementIndex::from_u32_index(0),
            cap: ElementArrayKey::Empty,
            len: 0,
            len_writable: true,
        }
    }
}

impl<'a> From<SealableElementsVector<'a>> for ElementsVector<'a> {
    #[inline(always)]
    fn from(value: SealableElementsVector<'a>) -> Self {
        Self {
            elements_index: value.elements_index,
            cap: value.cap,
            len: value.len,
        }
    }
}

/// An Array is an exotic object that gives special treatment to array index
/// property keys (see 6.1.7). A property whose property name is an array index
/// is also called an element. Every Array has a non-configurable "**length**"
/// property whose value is always a non-negative integral Number whose
/// mathematical value is strictly less than 2**32.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArrayHeapData<'a> {
    pub object_index: Option<OrdinaryObject<'a>>,
    // TODO: Use enum { ElementsVector, SmallVec<[Value; 3]> }
    // to get some inline benefit together with a 32 byte size
    // for ArrayHeapData to fit two in one cache line.
    pub elements: SealableElementsVector<'a>,
}

// SAFETY: Property implemented as a lifetime transmute.
unsafe impl Bindable for SealableElementsVector<'_> {
    type Of<'a> = SealableElementsVector<'a>;

    #[inline(always)]
    fn unbind(self) -> Self::Of<'static> {
        unsafe { core::mem::transmute::<Self, Self::Of<'static>>(self) }
    }

    #[inline(always)]
    fn bind<'a>(self, _gc: NoGcScope<'a, '_>) -> Self::Of<'a> {
        unsafe { core::mem::transmute::<Self, Self::Of<'a>>(self) }
    }
}

impl HeapMarkAndSweep for SealableElementsVector<'static> {
    fn mark_values(&self, queues: &mut WorkQueues) {
        let elements: ElementsVector = (*self).into();
        elements.mark_values(queues)
    }

    fn sweep_values(&mut self, compactions: &CompactionLists) {
        let mut elements: ElementsVector = (*self).into();
        elements.sweep_values(compactions);
        self.elements_index = elements.elements_index;
    }
}

// SAFETY: Property implemented as a lifetime transmute.
unsafe impl Bindable for ArrayHeapData<'_> {
    type Of<'a> = ArrayHeapData<'a>;

    #[inline(always)]
    fn unbind(self) -> Self::Of<'static> {
        unsafe { core::mem::transmute::<Self, Self::Of<'static>>(self) }
    }

    #[inline(always)]
    fn bind<'a>(self, _gc: NoGcScope<'a, '_>) -> Self::Of<'a> {
        unsafe { core::mem::transmute::<Self, Self::Of<'a>>(self) }
    }
}

impl HeapMarkAndSweep for ArrayHeapData<'static> {
    fn mark_values(&self, queues: &mut WorkQueues) {
        let Self {
            object_index,
            elements,
        } = self;
        object_index.mark_values(queues);
        elements.mark_values(queues);
    }

    fn sweep_values(&mut self, compactions: &CompactionLists) {
        let Self {
            object_index,
            elements,
        } = self;
        object_index.sweep_values(compactions);
        elements.sweep_values(compactions);
    }
}
