// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::prelude::*;

pub trait Screen {
    fn max_x(&self) -> f32;
    fn max_y(&self) -> f32;
    fn middle_x(&self) -> f32;
    fn middle_y(&self) -> f32;
    fn middle(&self) -> Vec2;
    fn middle_with_z(&self, z: f32) -> Vec3;
    fn bounds_box_inside(&self, size: Vec2) -> UiRect<f32>;
    fn bounds_box_outside(&self, size: Vec2) -> UiRect<f32>;
}

impl Screen for WindowDescriptor {
    fn max_x(&self) -> f32 {
        return self.width / self.height / self.max_y();
    }

    fn max_y(&self) -> f32 {
        return 1.0;
    }

    fn middle_x(&self) -> f32 {
        return self.max_x() / 2.0;
    }

    fn middle_y(&self) -> f32 {
        return self.max_y() / 2.0;
    }

    fn middle(&self) -> Vec2 {
        return Vec2::new(self.middle_x(), self.middle_y());
    }

    fn middle_with_z(&self, z: f32) -> Vec3 {
        return Vec3::new(self.middle_x(), self.middle_y(), z);
    }

    fn bounds_box_inside(&self, size: Vec2) -> UiRect<f32> {
        let rect = UiRect::<f32> {
            top: 1.0 - size.y / 2.0,
            bottom: size.y / 2.0,
            left: size.x / 2.0,
            right: self.width / self.height - size.x / 2.0,
        };
        return rect;
    }

    fn bounds_box_outside(&self, size: Vec2) -> UiRect<f32> {
        let rect = UiRect::<f32> {
            top: 1.0 + size.y / 2.0,
            bottom: -size.y / 2.0,
            left: -size.x / 2.0,
            right: self.width / self.height + size.x / 2.0,
        };
        return rect;
    }
}

#[derive(Component)]
pub struct DestroyOutsideScreen {
    pub size: Vec2,
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(destroy_outside_screen);
    }
}

fn destroy_outside_screen(
    mut commands: Commands,
    q: Query<(Entity, &DestroyOutsideScreen, &Transform)>,
    win: Res<WindowDescriptor>,
) {
    for (entity, destr, transform) in q.iter() {
        let bounds = win.bounds_box_outside(destr.size);
        if transform.translation.x > bounds.right
            || transform.translation.x < bounds.left
            || transform.translation.y > bounds.top
            || transform.translation.y < bounds.bottom
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}
