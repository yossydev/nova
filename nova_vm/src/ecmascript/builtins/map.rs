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
        indexes::{BaseIndex, MapIndex},
        CompactionLists, CreateHeapData, HeapMarkAndSweep, WorkQueues,
    },
    Heap,
};

use self::data::MapHeapData;

pub mod data;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Map(pub(crate) MapIndex);

impl Map {
    pub(crate) const fn _def() -> Self {
        Self(BaseIndex::from_u32_index(0))
    }

    pub(crate) const fn get_index(self) -> usize {
        self.0.into_index()
    }
}

impl From<Map> for MapIndex {
    fn from(val: Map) -> Self {
        val.0
    }
}

impl From<MapIndex> for Map {
    fn from(value: MapIndex) -> Self {
        Self(value)
    }
}

impl IntoValue for Map {
    fn into_value(self) -> Value {
        self.into()
    }
}

impl IntoObject for Map {
    fn into_object(self) -> Object {
        self.into()
    }
}

impl From<Map> for Value {
    fn from(val: Map) -> Self {
        Value::Map(val)
    }
}

impl From<Map> for Object {
    fn from(val: Map) -> Self {
        Object::Map(val)
    }
}

impl TryFrom<Object> for Map {
    type Error = ();

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value {
            Object::Map(data) => Ok(data),
            _ => Err(()),
        }
    }
}

impl InternalSlots for Map {
    const DEFAULT_PROTOTYPE: ProtoIntrinsics = ProtoIntrinsics::Map;

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

impl InternalMethods for Map {}

impl HeapMarkAndSweep for Map {
    fn mark_values(&self, queues: &mut WorkQueues) {
        queues.maps.push(*self);
    }

    fn sweep_values(&mut self, compactions: &CompactionLists) {
        compactions.maps.shift_index(&mut self.0);
    }
}

impl Index<Map> for Agent {
    type Output = MapHeapData;

    fn index(&self, index: Map) -> &Self::Output {
        &self.heap.maps[index]
    }
}

impl IndexMut<Map> for Agent {
    fn index_mut(&mut self, index: Map) -> &mut Self::Output {
        &mut self.heap.maps[index]
    }
}

impl Index<Map> for Vec<Option<MapHeapData>> {
    type Output = MapHeapData;

    fn index(&self, index: Map) -> &Self::Output {
        self.get(index.get_index())
            .expect("Map out of bounds")
            .as_ref()
            .expect("Map slot empty")
    }
}

impl IndexMut<Map> for Vec<Option<MapHeapData>> {
    fn index_mut(&mut self, index: Map) -> &mut Self::Output {
        self.get_mut(index.get_index())
            .expect("Map out of bounds")
            .as_mut()
            .expect("Map slot empty")
    }
}

impl CreateHeapData<MapHeapData, Map> for Heap {
    fn create(&mut self, data: MapHeapData) -> Map {
        self.maps.push(Some(data));
        Map(MapIndex::last(&self.maps))
    }
}
