use bevy::prelude::{shape::Cylinder, system_adapter::new, *};
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};

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

// A dummy struct used for Query-ing the cube entity, for altering its transform.
#[derive(Component, Default, Debug)]
struct Transformable {
    base_transform: Option<Transform>,
}

impl From<Transform> for Transformable {
    fn from(transform: Transform) -> Self {
        Self {
            base_transform: Some(transform),
        }
    }
}

// Main entrypoint
fn main() {
    // App entrypoint
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Quaternion transform demo".to_string(),
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
    let base_cylinder = meshes.add(Mesh::new_typical_cylinder(1.5, 1.));
    let middle_cylinder = meshes.add(Mesh::new_typical_cylinder(0.5, 2.));
    let arm = meshes.add(Mesh::new_box(1.0, 0.9, 4.0));
    let transform = Transform::from_translation(Vec3::ZERO);
    let arm_transform = Transform::from_xyz(0.0, 0.0, -2.0);
    commands.spawn((
        PbrBundle {
            mesh: base_cylinder,
            material: material.clone(),
            transform,
            ..default()
        },
        Transformable::default(),
    ));
    commands.spawn((
        PbrBundle {
            mesh: middle_cylinder,
            material: material.clone(),
            transform,
            ..default()
        },
        Transformable::default(),
    ));
    commands.spawn((
        PbrBundle {
            mesh: arm,
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
    }); // Calculate the Dual part of the Dual Quaternion
        //let dual_part_real = -0.5 * (ui_state.x * ui_state.xt + ui_state.y * ui_state.yt + ui_state.z * ui_state.zt);
        //let dual_part_i = 0.5 * (ui_state.xt * normalized_rotation_quat.w + ui_state.zt * normalized_rotation_quat.y - ui_state.yt * normalized_rotation_quat.z);
        //let dual_part_j = 0.5 * (-ui_state.zt * normalized_rotation_quat.x + ui_state.xt * normalized_rotation_quat.z + ui_state.yt * normalized_rotation_quat.w);
        //let dual_part_k = 0.5 * (ui_state.yt * normalized_rotation_quat.x - ui_state.xt * normalized_rotation_quat.y + ui_state.zt * normalized_rotation_quat.w);

    // Iterate over all cubes. In this case, we only have one, but this boilerplate is still considered best practice
    for (mut transform, transformable) in &mut transformables {
        // The actual quaternion transform occurs here
        if let Some(base_transform) = transformable.base_transform {
            let new_transform = Transform {
                translation: Quat::from_xyzw(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt)
                    .xyz(),
                rotation: Quat::from_xyzw(ui_state.x, ui_state.y, ui_state.z, ui_state.w)
                    .normalize(),
                scale: Vec3::ONE,
            };
            *transform = new_transform.mul_transform(base_transform);
        } else {
            transform.rotation =
                Quat::from_xyzw(ui_state.x, ui_state.y, ui_state.z, ui_state.w).normalize();
            transform.translation =
                Quat::from_xyzw(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt).xyz();
        }
    }
}
