use std::sync::Arc;

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, Color32, DragValue, Slider, Ui},
    EguiContexts,
};
use strum::EnumIter;
use strum::IntoEnumIterator;

impl Into<CtrlId> for usize {
    fn into(self) -> CtrlId {
        CtrlId(self)
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
struct CtrlId(usize);

#[derive(Clone, Default, Debug, EnumIter, Eq, PartialEq)]
enum CtrlMode {
    #[default]
    Normal,
    Sin,
    Cos,
    Tan,
}

impl CtrlMode {
    fn get_color(&self) -> Color32 {
        match self {
            CtrlMode::Normal => Color32::default(),
            CtrlMode::Sin => Color32::LIGHT_GRAY,
            CtrlMode::Cos => Color32::BROWN,
            CtrlMode::Tan => Color32::YELLOW,
        }
    }

    fn get_char(&self) -> char {
        match self {
            CtrlMode::Normal => 'n',
            CtrlMode::Sin => 's',
            CtrlMode::Cos => 'c',
            CtrlMode::Tan => 't',
        }
    }

    fn run_mode(&self, v: f32) -> f32 {
        match self {
            CtrlMode::Normal => v,
            CtrlMode::Sin => v.sin(),
            CtrlMode::Cos => v.cos(),
            CtrlMode::Tan => v.tan(),
        }
    }

    fn toggle(&mut self) {
        let first = Self::iter().next().expect("Could not get first value!");

        let next = Self::iter()
            .skip_while(|x| *self != *x)
            .skip(1)
            .next()
            .unwrap_or(first);
        dbg!(&next);
        *self = next;
    }
}

#[derive(Debug, Clone, Default)]
struct CtrlState {
    mode: CtrlMode,
    raw_value: f32,
}

#[derive(Debug, Default, Clone)]
struct CtrlsState(std::collections::HashMap<CtrlId, CtrlState>);

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    mat_transform: Mat4,
    ctrls_state: CtrlsState,
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

trait EguiExtras {
    fn ext_drag(
        &mut self,
        id: impl Into<CtrlId>,
        s: &mut CtrlsState,
        value: &mut f32,
        hover_text: impl Into<String>,
    );
}

impl EguiExtras for Ui {
    #[inline]
    fn ext_drag(
        &mut self,
        id: impl Into<CtrlId>,
        s: &mut CtrlsState,
        value: &mut f32,
        hover_text: impl Into<String>,
    ) {
        let id: CtrlId = id.into();
        if !s.0.contains_key(&id) {
            s.0.insert(id.clone(), CtrlState::default());
        }

        let ctrl_state = s.0.get_mut(&id).expect("Wha! How? O_o");
        let rep_char = ctrl_state.mode.get_char();
        // let raw_value = ctrl_state.raw_value;
        // let mode_runner = ctrl_state.mode.clone();
        // let ctrl_state = std::sync::Mutex::new(ctrl_state);
        let hover_text: String = hover_text.into();
        let drag = DragValue::new(value)
            .speed(0.08)
            .custom_formatter(|n, _| format!("{}{:.2}", rep_char, n))
            .custom_parser(
                |s| str::parse::<f64>(s.trim_start_matches(rep_char)).ok(), //{
                                                                            // Ok(v) => {
                                                                            //     let mut guard = ctrl_state.lock().unwrap();
                                                                            //     guard.raw_value = v;
                                                                            //     dbg!("parse!");
                                                                            //     Some(guard.mode.run_mode(guard.raw_value))
                                                                            // }
                                                                            // Err(_) => None,
                                                                            // },
            );
        let handle = self.add(drag);
        if handle.changed() {
            ctrl_state.raw_value = *value;
            *value = ctrl_state.mode.run_mode(*value);
        }
        if handle.secondary_clicked() {
            ctrl_state.mode.toggle();
        }

        handle.on_hover_text(hover_text);
    }
}

// This is where the transform happens
fn transform_ui(
    mut foxies: Query<(&mut Transform, &RotateFlag)>,
    mut ui_state: ResMut<UiState>,
    mut ctx: EguiContexts,
) {
    #[inline]
    fn mat4_slider<'a>(ui: &mut Ui, mut s: &mut CtrlsState, value: &mut Mat4) {
        egui::Grid::new("mat4_grid").show(ui, |ui| {
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "row");
            ui.colored_label(egui::Color32::GREEN, "i-hat");
            ui.colored_label(egui::Color32::RED, "j-hat");
            ui.colored_label(egui::Color32::from_rgb(0, 128, 128), "k-hat");
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "trans");

            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "X");
            ui.ext_drag(0, &mut s, &mut value.x_axis.x, "Mat4: x_axis, Vec4: x");
            ui.ext_drag(1, &mut s, &mut value.x_axis.y, "Mat4: x_axis, Vec4: y");
            ui.ext_drag(2, &mut s, &mut value.x_axis.z, "Mat4: x_axis, Vec4: z");
            ui.ext_drag(3, &mut s, &mut value.w_axis.x, "Mat4: w_axis, Vec4: x");

            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Y");
            ui.ext_drag(4, &mut s, &mut value.y_axis.x, "Mat4: y_axis, Vec4: x");
            ui.ext_drag(5, &mut s, &mut value.y_axis.y, "Mat4: y_axis, Vec4: y");
            ui.ext_drag(6, &mut s, &mut value.y_axis.z, "Mat4: y_axis, Vec4: z");
            ui.ext_drag(7, &mut s, &mut value.w_axis.y, "Mat4: w_axis, Vec4: y");

            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Z");
            ui.ext_drag(8, &mut s, &mut value.z_axis.x, "Mat4: z_axis, Vec4: x");
            ui.ext_drag(9, &mut s, &mut value.z_axis.y, "Mat4: z_axis, Vec4: y");
            ui.ext_drag(10, &mut s, &mut value.z_axis.z, "Mat4: z_axis, Vec4: z");
            ui.ext_drag(11, &mut s, &mut value.w_axis.z, "Mat4: w_axis, Vec4: z");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "W");

            ui.ext_drag(12, &mut s, &mut value.x_axis.w, "Mat4: x_axis, Vec4: w");
            ui.ext_drag(13, &mut s, &mut value.y_axis.w, "Mat4: y_axis, Vec4: w");
            ui.ext_drag(14, &mut s, &mut value.z_axis.w, "Mat4: z_axis, Vec4: w");
            ui.ext_drag(15, &mut s, &mut value.w_axis.w, "Mat4: w_axis, Vec4: w");

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
            *s = CtrlsState::default();
        }
    }

    // The floating EGUI window
    egui::Window::new("Mat4 Control").show(ctx.ctx_mut(), |ui| {
        // Note that the code inside this block is part of a closure, similar to lambdas in Python.

        // Slider width style
        ui.style_mut().spacing.slider_width = 450.0;
        // Sliders are added here, passed mutable access to the variables storing their states
        // Moooooom. The borrow checker is bullying me Y~Y
        let mut cloned_ui_mat = ui_state.mat_transform;
        mat4_slider(ui, &mut ui_state.ctrls_state, &mut cloned_ui_mat);
        ui_state.mat_transform = cloned_ui_mat;
    });

    for (mut transform, _foxy) in &mut foxies {
        *transform = Transform::from_matrix(ui_state.mat_transform);
        // transform.scale = Vec3::ONE * 10.0;
    }
}
