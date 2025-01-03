// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::{Index, IndexMut};

use crate::{
    ecmascript::{
        execution::{Agent, ProtoIntrinsics},
        types::{
            InternalMethods, InternalSlots, IntoObject, IntoValue, Object, OrdinaryObject, Value,
        },
    },
    heap::{
        indexes::{BaseIndex, WeakMapIndex},
        CreateHeapData, HeapMarkAndSweep,
    },
    Heap,
};

use self::data::WeakMapHeapData;

pub mod data;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct WeakMap(pub(crate) WeakMapIndex);

impl WeakMap {
    pub(crate) const fn _def() -> Self {
        Self(BaseIndex::from_u32_index(0))
    }

    pub(crate) const fn get_index(self) -> usize {
        self.0.into_index()
    }
}

impl From<WeakMap> for WeakMapIndex {
    fn from(val: WeakMap) -> Self {
        val.0
    }
}

impl From<WeakMapIndex> for WeakMap {
    fn from(value: WeakMapIndex) -> Self {
        Self(value)
    }
}

impl IntoValue for WeakMap {
    fn into_value(self) -> Value {
        self.into()
    }
}

impl IntoObject for WeakMap {
    fn into_object(self) -> Object {
        self.into()
    }
}

impl From<WeakMap> for Value {
    fn from(val: WeakMap) -> Self {
        Value::WeakMap(val)
    }
}

impl From<WeakMap> for Object {
    fn from(val: WeakMap) -> Self {
        Object::WeakMap(val)
    }
}

impl InternalSlots for WeakMap {
    const DEFAULT_PROTOTYPE: ProtoIntrinsics = ProtoIntrinsics::WeakMap;

    #[inline(always)]
    fn get_backing_object(self, agent: &Agent) -> Option<OrdinaryObject<'static>> {
        agent[self].object_index
    }

    fn set_backing_object(self, agent: &mut Agent, backing_object: OrdinaryObject<'static>) {
        assert!(agent[self]
            .object_index
            .replace(backing_object.unbind())
            .is_none());
    }
}

impl InternalMethods for WeakMap {}

impl Index<WeakMap> for Agent {
    type Output = WeakMapHeapData;

    fn index(&self, index: WeakMap) -> &Self::Output {
        &self.heap.weak_maps[index]
    }
}

impl IndexMut<WeakMap> for Agent {
    fn index_mut(&mut self, index: WeakMap) -> &mut Self::Output {
        &mut self.heap.weak_maps[index]
    }
}

impl Index<WeakMap> for Vec<Option<WeakMapHeapData>> {
    type Output = WeakMapHeapData;

    fn index(&self, index: WeakMap) -> &Self::Output {
        self.get(index.get_index())
            .expect("WeakMap out of bounds")
            .as_ref()
            .expect("WeakMap slot empty")
    }
}

impl IndexMut<WeakMap> for Vec<Option<WeakMapHeapData>> {
    fn index_mut(&mut self, index: WeakMap) -> &mut Self::Output {
        self.get_mut(index.get_index())
            .expect("WeakMap out of bounds")
            .as_mut()
            .expect("WeakMap slot empty")
    }
}

impl CreateHeapData<WeakMapHeapData, WeakMap> for Heap {
    fn create(&mut self, data: WeakMapHeapData) -> WeakMap {
        self.weak_maps.push(Some(data));
        // TODO: The type should be checked based on data or something equally stupid
        WeakMap(WeakMapIndex::last(&self.weak_maps))
    }
}

impl HeapMarkAndSweep for WeakMap {
    fn mark_values(&self, queues: &mut crate::heap::WorkQueues) {
        queues.weak_maps.push(*self);
    }

    fn sweep_values(&mut self, compactions: &crate::heap::CompactionLists) {
        compactions.weak_maps.shift_index(&mut self.0);
    }
}
