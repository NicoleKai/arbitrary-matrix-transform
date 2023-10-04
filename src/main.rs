use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, DragValue, Slider, Ui},
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
    // fn common_slider<'a>(value: &'a mut f32, text: &str) -> Slider<'a> {
    //     Slider::new(value, -10.0..=10.0)
    //         .text(text)
    //         .clamp_to_range(false)
    //         .drag_value_speed(0.01)
    //         .step_by(0.01)
    //     DragValue::new(value);
    // }

    fn vec4_slider<'a>(ui: &mut Ui, value: &mut Vec4, prepend: impl Into<String>) {
        let prepend: String = "X".to_string();
        ui.add(DragValue::new(&mut value.x).speed(0.5))
            .on_hover_text(format!("Mat4: {}, Vec4: x", prepend));
        ui.add(DragValue::new(&mut value.y).speed(0.5))
            .on_hover_text(format!("Mat4: {}, Vec4: y", prepend));
        ui.add(DragValue::new(&mut value.z).speed(0.5))
            .on_hover_text(format!("Mat4: {}, Vec4: z", prepend));
        ui.add(DragValue::new(&mut value.w).speed(0.5))
            .on_hover_text(format!("Mat4: {}, Vec4: w", prepend));
    }
    fn mat4_slider<'a>(mut ui: &mut Ui, value: &mut Mat4) {
        egui::Grid::new("mat4_grid").show(ui, |mut ui| {
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "row");
            ui.colored_label(egui::Color32::GREEN, "i-hat");
            ui.colored_label(egui::Color32::RED, "j-hat");
            ui.colored_label(egui::Color32::from_rgb(0, 128, 128), "k-hat");
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "trans");
            ui.end_row();
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "X");

            ui.add(DragValue::new(&mut value.x_axis.x).speed(0.5))
                .on_hover_text("Mat4: x_axis, Vec4: x");
            ui.add(DragValue::new(&mut value.x_axis.y).speed(0.5))
                .on_hover_text("Mat4: x_axis, Vec4: y");
            ui.add(DragValue::new(&mut value.x_axis.z).speed(0.5))
                .on_hover_text("Mat4: x_axis, Vec4: z");
            ui.add(DragValue::new(&mut value.w_axis.x).speed(0.5))
                .on_hover_text("Mat4: w_axis, Vec4: x");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Y");
            ui.add(DragValue::new(&mut value.y_axis.x).speed(0.5))
                .on_hover_text("Mat4: y_axis, Vec4: x");
            ui.add(DragValue::new(&mut value.y_axis.y).speed(0.5))
                .on_hover_text("Mat4: y_axis, Vec4: y");
            ui.add(DragValue::new(&mut value.y_axis.z).speed(0.5))
                .on_hover_text("Mat4: y_axis, Vec4: z");
            ui.add(DragValue::new(&mut value.w_axis.y).speed(0.5))
                .on_hover_text("Mat4: w_axis, Vec4: y");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Z");
            ui.add(DragValue::new(&mut value.z_axis.x).speed(0.5))
                .on_hover_text("Mat4: z_axis, Vec4: x");
            ui.add(DragValue::new(&mut value.z_axis.y).speed(0.5))
                .on_hover_text("Mat4: z_axis, Vec4: y");
            ui.add(DragValue::new(&mut value.z_axis.z).speed(0.5))
                .on_hover_text("Mat4: z_axis, Vec4: z");
            ui.add(DragValue::new(&mut value.w_axis.z).speed(0.5))
                .on_hover_text("Mat4: w_axis, Vec4: z");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "W");

            ui.add(DragValue::new(&mut value.x_axis.w).speed(0.5))
                .on_hover_text("Mat4: x_axis, Vec4: w");
            ui.add(DragValue::new(&mut value.y_axis.w).speed(0.5))
                .on_hover_text("Mat4: y_axis, Vec4: w");
            ui.add(DragValue::new(&mut value.z_axis.w).speed(0.5))
                .on_hover_text("Mat4: z_axis, Vec4: w");
            ui.add(DragValue::new(&mut value.w_axis.w).speed(0.5))
                .on_hover_text("Mat4: w_axis, Vec4: w");
            ui.end_row();
        });
        // ui.with_layout(Layout::, )
        ui.label(format!(
            "Determinant: {}",
            value.determinant() // value.x_axis.x * value.y_axis.y * value.z_axis.z
        ))
        .on_hover_text("The change in volume applied by this transform (ignoring w_axis).");
        if ui.button("Reset").clicked() {
            *value = Mat4::default();
        }
    }

    // The floating EGUI window
    egui::Window::new("Mat4 Control").show(ctx.ctx_mut(), |ui| {
        // Note that the code inside this block is part of a closure, similar to lambdas in Python.

        // Slider width style
        ui.style_mut().spacing.slider_width = 450.0;
        // Sliders are added here, passed mutable access to the variables storing their states
        mat4_slider(ui, &mut ui_state.mat_transform);
    });

    for (mut transform, _foxy) in &mut foxies {
        *transform = Transform::from_matrix(ui_state.mat_transform);
        // transform.scale = Vec3::ONE * 10.0;
    }
}
