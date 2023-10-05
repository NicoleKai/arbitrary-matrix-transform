use std::sync::Arc;

use bevy::{pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::{
    egui::{self, Color32, DragValue, Slider, Ui},
    EguiContexts,
};
use regex::Regex;
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
    NegSin,
    Cos,
    NegCos,
    Tan,
    NegTan,
}

impl CtrlMode {
    // fn get_color(&self) -> Color32 {
    //     match self {
    //         CtrlMode::Normal => Color32::default(),
    //         CtrlMode::Sin => Color32::LIGHT_GRAY,
    //         CtrlMode::Cos => Color32::BROWN,
    //         CtrlMode::Tan => Color32::YELLOW,
    //     }
    // }

    fn get_str(&self) -> &str {
        match self {
            CtrlMode::Normal => "",
            CtrlMode::Sin => "s",
            CtrlMode::NegSin => "-s",
            CtrlMode::Cos => "c",
            CtrlMode::NegCos => "-c",
            CtrlMode::Tan => "t",
            CtrlMode::NegTan => "-t",
        }
    }

    fn run_mode(&self, v: f32) -> f32 {
        match self {
            CtrlMode::Normal => v,
            CtrlMode::Sin => v.sin(),
            CtrlMode::NegSin => (-(v.abs())).sin(),
            CtrlMode::Cos => v.cos(),
            CtrlMode::NegCos => (-(v.abs())).cos(),
            CtrlMode::Tan => v.tan(),
            CtrlMode::NegTan => (-(v.abs())).tan(),
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
    value: f32,
}

#[derive(Debug, Default, Clone)]
struct CtrlsState(std::collections::HashMap<CtrlId, CtrlState>);

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    mat_transform: Mat4,
    ctrls_state: CtrlsState,
    rot_about_origin: f32,
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
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
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
        let rep_seg = ctrl_state.mode.get_str();
        let hover_text: String = hover_text.into();
        let drag = DragValue::new(value)
            .speed(0.08)
            .custom_formatter(|n, _| format!("{}{:.2}", rep_seg, n))
            .custom_parser(|s| {
                // TODO: fix recompiling regex every time parser runs
                let re = Regex::new(r"^(-[sct]|[sct])").unwrap();
                let cut = re.replace(s, "");
                str::parse::<f64>(cut.to_string().as_str()).ok()
            });
        let handle = self.add(drag);
        if handle.changed() {
            ctrl_state.value = *value;
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
    fn mat4_ui<'a>(ui: &mut Ui, ui_state: &mut UiState, value: &mut Mat4) {
        let s = &mut ui_state.ctrls_state;
        ui.group(|ui| {
            egui::Grid::new("mat4_grid").show(ui, |ui| {
                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "row");
                ui.colored_label(egui::Color32::GREEN, "i-hat");
                ui.colored_label(egui::Color32::RED, "j-hat");
                ui.colored_label(egui::Color32::from_rgb(0, 128, 128), "k-hat");
                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "trans");
                ui.end_row();

                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "X");
                ui.ext_drag(0, s, &mut value.x_axis.x, "Mat4: x_axis, Vec4: x");
                ui.ext_drag(1, s, &mut value.x_axis.y, "Mat4: x_axis, Vec4: y");
                ui.ext_drag(2, s, &mut value.x_axis.z, "Mat4: x_axis, Vec4: z");
                ui.ext_drag(3, s, &mut value.w_axis.x, "Mat4: w_axis, Vec4: x");
                ui.end_row();

                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Y");
                ui.ext_drag(4, s, &mut value.y_axis.x, "Mat4: y_axis, Vec4: x");
                ui.ext_drag(5, s, &mut value.y_axis.y, "Mat4: y_axis, Vec4: y");
                ui.ext_drag(6, s, &mut value.y_axis.z, "Mat4: y_axis, Vec4: z");
                ui.ext_drag(7, s, &mut value.w_axis.y, "Mat4: w_axis, Vec4: y");
                ui.end_row();

                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Z");
                ui.ext_drag(8, s, &mut value.z_axis.x, "Mat4: z_axis, Vec4: x");
                ui.ext_drag(9, s, &mut value.z_axis.y, "Mat4: z_axis, Vec4: y");
                ui.ext_drag(10, s, &mut value.z_axis.z, "Mat4: z_axis, Vec4: z");
                ui.ext_drag(11, s, &mut value.w_axis.z, "Mat4: w_axis, Vec4: z");
                ui.end_row();

                ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "W");
                ui.ext_drag(12, s, &mut value.x_axis.w, "Mat4: x_axis, Vec4: w");
                ui.ext_drag(13, s, &mut value.y_axis.w, "Mat4: y_axis, Vec4: w");
                ui.ext_drag(14, s, &mut value.z_axis.w, "Mat4: z_axis, Vec4: w");
                ui.ext_drag(15, s, &mut value.w_axis.w, "Mat4: w_axis, Vec4: w");
                ui.end_row();
            });
        });
        if ui.button("Reset").clicked() {
            *value = Mat4::default();
            *s = CtrlsState::default();
        }
        ui.label(format!(
            "Determinant: {}",
            value.determinant() // value.x_axis.x * value.y_axis.y * value.z_axis.z
        ))
        .on_hover_text("The change in volume applied by this transform (ignoring w_axis).");
        let handle = ui.add(Slider::new(&mut ui_state.rot_about_origin, -6.28..=6.28));
        if handle.changed() {
            value.x_axis.x = ui_state.rot_about_origin;
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
        mat4_ui(ui, &mut ui_state, &mut cloned_ui_mat);
        ui_state.mat_transform = cloned_ui_mat;
    });

    for (mut transform, _foxy) in &mut foxies {
        *transform = Transform::from_matrix(ui_state.mat_transform);
        // transform.scale = Vec3::ONE * 10.0;
    }
}
