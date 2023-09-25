use std::ops::Add;

use bevy::prelude::{
    shape::{Cylinder, Quad},
    system_adapter::new,
    *,
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};
use static_math;

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    xt: f32,
    yt: f32,
    zt: f32,
    wt: f32,
}

#[derive(Component, Default, Debug)]
struct Transformable {
    node_transform: Transform,
}
impl From<Transform> for Transformable {
    fn from(node_transform: Transform) -> Self {
        Self { node_transform }
    }
}

pub trait InternalFrom<T>: Sized {
    fn ext_from(value: T) -> Self;
}

impl InternalFrom<Quat> for static_math::Quaternion<f32> {
    fn ext_from(quat: Quat) -> Self {
        static_math::Quaternion::new_from(quat.w, quat.x, quat.y, quat.z)
    }
}

impl InternalFrom<static_math::Quaternion<f32>> for Quat {
    fn ext_from(quaternion: static_math::Quaternion<f32>) -> Self {
        let real: f32 = quaternion.real();
        let imaginary: static_math::V3<f32> = quaternion.imag();
        Quat::from_xyzw(imaginary[0], imaginary[1], imaginary[2], real)
    }
}

// trait BevyQuatToStaticQuaternion {
//     fn as_static_quat(&mut self) -> static_math::Quaternion<f32>;
// }

// impl BevyQuatToStaticQuaternion for Quat {
//     fn as_static_quat(&mut self) -> static_math::Quaternion<f32> {
//         static_math::Quaternion::new_from(self.w, self.x, self.y, self.z)
//     }
// }
// trait StaticQuaternionToBevyQuat {
//     fn as_bevy_quat(&mut self) -> static_math::Quaternion<f32>;
// }

// impl BevyQuatToStaticQuaternion for Quat {
//     fn as_static_quat(&mut self) -> static_math::Quaternion<f32> {
//         static_math::Quaternion::new_from(self.w, self.x, self.y, self.z)
//     }
// }

// Main entrypoint
fn main() {
    // App entrypoint
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Quaternion Forward Kinematics Demo".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        // Systems (functions that are called at regular intervals)
        .add_systems(Startup, setup)
        .add_systems(Update, transform_ui)
        // Resources (live data that can be accessed from any system)
        .init_resource::<UiState>()
        .run(); // Event loop etc occurs here
}

trait NewMesh {
    fn new_box(x: f32, y: f32, z: f32) -> Mesh;
    fn new_typical_cylinder(radius: f32, height: f32) -> Mesh;
    fn new_cylinder(radius: f32, height: f32, resolution: u32, segments: u32) -> Mesh;
}

impl NewMesh for Mesh {
    fn new_box(x: f32, y: f32, z: f32) -> Mesh {
        Mesh::from(shape::Box::new(x, y, z))
    }

    fn new_typical_cylinder(radius: f32, height: f32) -> Mesh {
        Mesh::from(shape::Cylinder {
            radius,
            height,
            resolution: 64,
            segments: 1,
        })
    }

    fn new_cylinder(radius: f32, height: f32, resolution: u32, segments: u32) -> Mesh {
        Mesh::from(shape::Cylinder {
            radius,
            height,
            resolution,
            segments,
        })
    }
}

// Setup basic facilities
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::WHITE.into());
    let mesh_base = meshes.add(Mesh::new_typical_cylinder(1.5, 1.));
    let mesh_middle = meshes.add(Mesh::new_typical_cylinder(0.5, 2.));
    let mesh_arm = meshes.add(Mesh::new_box(1.0, 0.9, 4.0));
    let transform = Transform::from_translation(Vec3::ZERO);
    let arm_transform = Transform::from_xyz(0.0, 0.0, -2.0);

    commands.spawn((
        PbrBundle {
            mesh: mesh_base,
            material: material.clone(),
            transform,
            ..default()
        },
        Transformable::default(),
    ));

    commands.spawn((
        PbrBundle {
            mesh: mesh_middle,
            material: material.clone(),
            transform,
            ..default()
        },
        Transformable::default(),
    ));

    commands.spawn((
        PbrBundle {
            mesh: mesh_arm,
            material: material.clone(),
            transform: arm_transform,
            ..default()
        },
        Transformable::from(arm_transform),
    ));

    // Camera is necessary to render anything
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn a light so that it's easier to see the cube
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::ONE * 3.0),
        ..default()
    });
}

// This is where the transform happens
fn transform_ui(
    mut transformables: Query<(&mut Transform, &mut Transformable)>,
    mut ui_state: ResMut<UiState>,
    mut ctx: EguiContexts,
) {
    // A wrapper function for creating a slider with common settings,
    // e.g. range, clamp, step_by, etc
    fn common_slider<'a>(value: &'a mut f32, text: &str) -> Slider<'a> {
        Slider::new(value, -10.0..=10.0)
            .text(text)
            .clamp_to_range(false)
            .drag_value_speed(0.01)
            .step_by(0.01)
    }

    // The floating EGUI window
    egui::Window::new("Quaternion control").show(ctx.ctx_mut(), |ui| {
        // Note that the code inside this block is part of a closure, similar to lambdas in Python.

        // Slider width style
        ui.style_mut().spacing.slider_width = 450.0;
        // Sliders are added here, passed mutable access to the variables storing their states
        ui.add(common_slider(&mut ui_state.x, "x"));
        ui.add(common_slider(&mut ui_state.y, "y"));
        ui.add(common_slider(&mut ui_state.z, "z"));
        ui.add(common_slider(&mut ui_state.w, "w"));
        ui.add(common_slider(&mut ui_state.xt, "xt"));
        ui.add(common_slider(&mut ui_state.yt, "yt"));
        ui.add(common_slider(&mut ui_state.zt, "zt"));
        ui.add(common_slider(&mut ui_state.wt, "wt"));
    });

    // Iterate over all transformables
    for (mut transform, transformable) in &mut transformables {
        let translation =
            static_math::Quaternion::new_from(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt);
        let rotation =
            static_math::Quaternion::new_from(ui_state.x, ui_state.y, ui_state.z, ui_state.w);
        let base_transform = Transform {
            translation: Quat::ext_from(translation).xyz(),
            rotation: Quat::ext_from(rotation).normalize(),
            scale: Vec3::ONE,
        };
        // The actual quaternion transform occurs here
        *transform = base_transform * transformable.node_transform;
    }
}
