use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, Slider, Ui},
    EguiContexts,
};

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    mat_transform: Mat4,
    // x: f32,
    // y: f32,
    // z: f32,
    // w: f32,
    // xt: f32,
    // yt: f32,
    // zt: f32,
    // wt: f32,
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
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
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
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn a light so that it's easier to see the cube
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::ONE * 3.0),
        ..default()
    });
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("Foxy.gltf#Scene0"),
            transform: Transform::from_scale(Vec3::splat(10.)),
            ..default()
        },
        RotateFlag {},
    ));
}

// This is where the transform happens
fn transform_ui(
    mut foxies: Query<(&mut Transform, &RotateFlag)>,
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

    fn vec4_slider<'a>(ui: &mut Ui, value: &mut Vec4, prepend: impl Into<String>) {
        let prepend: String = prepend.into();
        ui.add(common_slider(
            &mut value.x,
            format!("{}: Vec4 X", prepend).as_str(),
        ));
        ui.add(common_slider(
            &mut value.y,
            format!("{}: Vec4 Y", prepend).as_str(),
        ));
        ui.add(common_slider(
            &mut value.z,
            format!("{}: Vec4 Z", prepend).as_str(),
        ));
        ui.add(common_slider(
            &mut value.w,
            format!("{}: Vec4 W", prepend).as_str(),
        ));
    }
    fn mat4_slider<'a>(mut ui: &mut Ui, value: &mut Mat4) {
        vec4_slider(&mut ui, &mut value.x_axis, "X axis");
        vec4_slider(&mut ui, &mut value.y_axis, "Y axis");
        vec4_slider(&mut ui, &mut value.z_axis, "Z axis");
        vec4_slider(&mut ui, &mut value.w_axis, "W axis");
    }

    // The floating EGUI window
    egui::Window::new("Quaternion control").show(ctx.ctx_mut(), |ui| {
        // Note that the code inside this block is part of a closure, similar to lambdas in Python.

        // Slider width style
        ui.style_mut().spacing.slider_width = 450.0;
        // Sliders are added here, passed mutable access to the variables storing their states
        mat4_slider(ui, &mut ui_state.mat_transform);
    });

    for (mut transform, _foxy) in &mut foxies {
        // let new_transform = Transform {
        //     rotation: Quat::from_xyzw(ui_state.x, ui_state.y, ui_state.z, ui_state.w).normalize(),
        //     translation: Quat::from_xyzw(ui_state.xt, ui_state.yt, ui_state.zt, ui_state.wt).xyz(),
        //     scale: Vec3::ONE * 10.0,
        // };
        // let mut mat = new_transform.compute_matrix();
        // dbg!(mat);
        // mat.x_axis.x = 0.;
        *transform = Transform::from_matrix(ui_state.mat_transform);
    }
}
