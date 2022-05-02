// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::prelude::*;

#[cfg(feature = "inspector")]
use {
    crate::player::Player,
    bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin},
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(
        &self,
        #[cfg_attr(not(feature = "inspector"), allow(unused_variables))] app: &mut App,
    ) {
        #[cfg(feature = "inspector")]
        app.add_plugin(WorldInspectorPlugin::new())
            .register_inspectable::<Player>();
    }
}
