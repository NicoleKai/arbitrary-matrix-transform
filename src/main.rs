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
use static_math::{self, DualQuaternion, Quaternion};

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    // x: f32,
    // y: f32,
    // z: f32,
    // w: f32,
    // xt: f32,
    // yt: f32,
    // zt: f32,
    // wt: f32,
    slider: f32,
    pitch: f32,
    yaw: f32,
    roll: f32,
}

#[derive(Resource, Default, Clone)]
struct JointTrans {
    dual_quat: Vec<DualQuaternion<f32>>,
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

impl InternalFrom<static_math::matrix3x3::M33<f32>> for Mat3 {
    fn ext_from(static_mat3: static_math::matrix3x3::M33<f32>) -> Self {
        let s = static_mat3.get_rows();
        let arr: [f32; 9] = [
            s[0][0], s[0][1], s[0][2], s[1][0], s[1][1], s[1][2], s[2][0], s[2][1], s[2][2],
        ];
        // TODO: check if we need to transpose
        Self::from_cols_array(&arr)
    }
}

// impl InternalFrom<static_math::matrix3x3::M33<f32>> for Mat3 {
//     fn ext_from(static_mat3: static_math::matrix3x3::M33<f32>) -> Self {
//         let s = static_mat3.get_rows();
//         let arr: [f32; 9] = [
//             s[0][0], s[0][1], s[0][2], s[1][0], s[1][1], s[1][2], s[2][0], s[2][1], s[2][2],
//         ];
//         // TODO: check if we need to transpose
//         Self::from_cols_array(&arr)
//     }
// }

impl InternalFrom<Mat3> for static_math::matrix3x3::M33<f32> {
    fn ext_from(m: Mat3) -> Self {
        let m = m.to_cols_array_2d();
        Self::new([m[0], m[1], m[2]])
    }
}

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
        .init_resource::<JointTrans>()
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
    mut joint_trans: ResMut<JointTrans>,
) {
    let material = materials.add(Color::WHITE.into());
    let mesh_base = meshes.add(Mesh::new_typical_cylinder(1.5, 1.));
    let mesh_middle = meshes.add(Mesh::new_typical_cylinder(0.5, 2.));
    let mesh_arm = meshes.add(Mesh::new_box(1.0, 0.9, 4.0));
    let transform = Transform::from_translation(Vec3::ZERO);
    let arm_transform = Transform::from_xyz(0.0, 0.0, -2.0);

    let q_real = Quaternion::new_from(0., 0., 0., 0.);
    let q_dual = Quaternion::new_from(0., 0., 0., 0.);
    q_real * q_dual;
    joint_trans
        .dual_quat
        .push(DualQuaternion::new(q_real, q_dual));
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
    mut joint_trans: ResMut<JointTrans>,
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
        ui.add(common_slider(&mut ui_state.slider, "slider"));
        ui.add(common_slider(&mut ui_state.pitch, "pitch"));
        ui.add(common_slider(&mut ui_state.yaw, "yaw"));
        ui.add(common_slider(&mut ui_state.roll, "roll"));
        // ui.add(common_slider(&mut ui_state.x, "x"));
        // ui.add(common_slider(&mut ui_state.y, "y"));
        // ui.add(common_slider(&mut ui_state.z, "z"));
        // ui.add(common_slider(&mut ui_state.w, "w"));
        // ui.add(common_slider(&mut ui_state.xt, "xt"));
        // ui.add(common_slider(&mut ui_state.yt, "yt"));
        // ui.add(common_slider(&mut ui_state.zt, "zt"));
        // ui.add(common_slider(&mut ui_state.wt, "wt"));
    });

    let i_hat = Vec3::new(1., 0., 0.);
    let j_hat = Vec3::new(0., 1., 0.);
    let k_hat = Vec3::new(0., 0., 1.);
    let u_x = 
    // Iterate over all transformables
    for (mut transform, transformable) in &mut transformables {
        // let translation =
        //     static_math::Quaternion::new_from(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt);
        // let rotation =
        //     static_math::Quaternion::new_from(ui_state.x, ui_state.y, ui_state.z, ui_state.w);
        // // let a =

        let first_joint = *joint_trans.dual_quat.get(0).unwrap();
        let dt = DualQuaternion::new_from_rotation(&Quaternion::<f32>::from_euler_angles(
            ui_state.slider,
            0.,
            0.,
        ));
        let (rot, trans) = dt.to_rotation_translation();
        // rotation.exp()

        // let base_transform = Transform {
        //     translation: Quat::ext_from(translation).xyz(),
        //     rotation: Quat::ext_from(rotation).normalize(),
        //     scale: Vec3::ONE,
        // };
        // The actual quaternion transform occurs here
        // *transform =
        // let base_transform = Transform {
        Transform::from_matrix()
        // Mat3::ext_from(rot))
        // transformable.node_transform;
    }
}
