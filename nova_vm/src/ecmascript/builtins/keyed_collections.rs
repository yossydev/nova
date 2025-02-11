// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(crate) mod map_objects;
#[cfg(feature = "set")]
pub(crate) mod set_objects;
#[cfg(feature = "weak-refs")]
pub(crate) mod weak_map_objects;
#[cfg(feature = "weak-refs")]
pub(crate) mod weak_set_objects;
