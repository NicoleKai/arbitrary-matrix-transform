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
#[derive(Component)]
struct RotateFlag;

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

trait NewCylinder {
    fn new(radius: f32, height: f32, resolution: u32, segments: u32) -> shape::Cylinder;
    fn new_typical(radius: f32, height: f32) -> shape::Cylinder;
}

impl NewCylinder for Cylinder {
    fn new(radius: f32, height: f32, resolution: u32, segments: u32) -> shape::Cylinder {
        shape::Cylinder {
            radius,
            height,
            resolution,
            segments,
        }
    }
    fn new_typical(radius: f32, height: f32) -> shape::Cylinder {
        shape::Cylinder {
            radius,
            height,
            resolution: 64,
            segments: 1,
        }
    }
}

// Setup basic facilities
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::WHITE.into());
    let base_cylinder = meshes.add(Mesh::from(Cylinder::new_typical(1.5, 1.)));
    let middle_cylinder = meshes.add(Mesh::from(Cylinder::new_typical(0.5, 2.)));
    let arm = meshes.add(Mesh::from(Box::new_typical(0.5, 2.)));
    let transform = Transform::from_translation(Vec3::ZERO);
    commands.spawn((
        PbrBundle {
            mesh: base_cylinder,
            material,
            transform,
            ..default()
        },
        RotateFlag {},
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
    mut cubes: Query<(&mut Transform, &RotateFlag)>,
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
    for (mut transform, _cube) in &mut cubes {
        // The actual quaternion transform occurs here
        transform.rotation =
            Quat::from_xyzw(ui_state.x, ui_state.y, ui_state.z, ui_state.w).normalize();
        transform.translation =
            Quat::from_xyzw(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt).xyz();
    }
}
