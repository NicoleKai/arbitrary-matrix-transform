use bevy::prelude::*;
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

// Setup basic facilities
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube, with color settings so that it's easier to view
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::ZERO),
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
    });
    // Iterate over all cubes. In this case, we only have one, but this boilerplate is still considered best practice
    for (mut transform, _cube) in &mut cubes {
        // The actual quaternion transform occurs here
        let unnormalized_quat = Quat::from_xyzw(ui_state.x, ui_state.y, ui_state.z, ui_state.w);
        transform.rotation = unnormalized_quat.normalize();
    }
}
