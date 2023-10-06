use std::{f64::consts::PI, sync::Arc, time::Duration};

use bevy::{
    core::FrameCount,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    pbr::DirectionalLightShadowMap,
    prelude::{IntoSystem, *},
};
use bevy_egui::{
    egui::{self, DragValue, Ui},
    EguiContexts, EguiSettings,
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

#[derive(Clone, Default, Debug, EnumIter, Eq, PartialEq, strum::Display)]
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

    // fn toggle(&mut self) {
    //     let first = Self::iter().next().expect("Could not get first value!");

    //     let next = Self::iter()
    //         .skip_while(|x| *self != *x)
    //         .skip(1)
    //         .next()
    //         .unwrap_or(first);
    //     dbg!(&next);
    //     *self = next;
    // }
}

#[derive(Debug, Clone, Default)]
struct CtrlState {
    is_changed: bool,
    mode: CtrlMode,
    default_value: f32,
    value: f32,
}

impl CtrlState {
    fn reset_value(&mut self) {
        self.value = self.default_value;
    }
}

const FOUR_PI: f64 = PI * 4.;

#[derive(Debug, Default, Clone)]
struct CtrlsState(std::collections::HashMap<CtrlId, CtrlState>);

impl CtrlsState {
    fn reset_modes(&mut self) {
        for (_id, state) in self.0.iter_mut() {
            state.mode = CtrlMode::default();
        }
    }

    fn reset_values(&mut self) {
        for (_id, state) in self.0.iter_mut() {
            state.reset_value();
        }
    }
}

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Clone)]
struct UiState {
    // scale: f64,
    mat_transform: Mat4,
    ctrls_state: CtrlsState,
    theta: f32,
    ambient_brightness: f32,
}

#[derive(Resource, Clone)]
struct WndState {
    is_open_help_wnd: bool,
    is_open_ctrl_wnd: bool,
    is_open_status_wnd: bool,
}

impl Default for WndState {
    fn default() -> Self {
        Self {
            is_open_help_wnd: true,
            is_open_ctrl_wnd: true,
            is_open_status_wnd: true,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            // scale: 1.0,
            ambient_brightness: 0.25,
            // trying to do ..default() would cause a stack overflow here ;)
            mat_transform: default(),
            ctrls_state: default(),
            theta: default(),
        }
    }
}

// A dummy struct used for Query-ing the cube entity, for altering its transform.
#[derive(Component)]
struct Transformable {
    transform: Transform,
}

#[derive(Resource, Default)]
struct AssetsLoading(Vec<HandleUntyped>);

#[derive(Resource, Default)]
struct PgmStatus {
    last_fps: f64,
    last_frame_count: u32,
    update_timer: Timer,
}

// Main entrypoint
fn main() {
    // App entrypoint
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Matrix Manipulation Demo".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        // Resources (live data that can be accessed from any system)
        .init_resource::<AssetsLoading>()
        .init_resource::<UiState>()
        .init_resource::<WndState>()
        .init_resource::<PgmStatus>()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(bevy_egui::EguiPlugin)
        // Systems (functions that are called at regular intervals)
        .add_systems(Startup, setup)
        .add_systems(Update, window_view)
        .add_systems(
            Update,
            window_help.run_if(IntoSystem::into_system(|wnd_state: Res<WndState>| {
                wnd_state.is_open_help_wnd == true
            })),
        )
        .add_systems(
            Update,
            window_ctrl.run_if(IntoSystem::into_system(|wnd_state: Res<WndState>| {
                wnd_state.is_open_ctrl_wnd == true
            })),
        )
        .add_systems(
            Update,
            ui_status.run_if(IntoSystem::into_system(|wnd_state: Res<WndState>| {
                wnd_state.is_open_status_wnd == true
            })),
        )
        .add_systems(Update, keyboard_input)
        .run(); // Event loop etc occurs here
}

/// Some helper implementations for quickly making meshes of basic objects
trait NewMesh {
    fn new_box(x: f32, y: f32, z: f32) -> Mesh;
    fn new_typical_cylinder(radius: f32, height: f32) -> Mesh;
    fn new_cylinder(radius: f32, height: f32, resolution: u32, segments: u32) -> Mesh;
}

/// Some helper implementations for quickly making meshes of basic objects
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
    mut ambient_light: ResMut<AmbientLight>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    ambient_light.color = Color::WHITE;
    commands.insert_resource(PgmStatus {
        update_timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::ONE * 3.0),
        ..default()
    });

    let foxy_handle = asset_server.load("Foxy.gltf#Scene0");
    loading.0.push(foxy_handle.clone_untyped());

    commands.spawn((
        SceneBundle {
            scene: foxy_handle,
            transform: Transform::default(),
            ..default()
        },
        Transformable {
            transform: Transform::from_scale(Vec3::splat(10.))
                .with_translation(Vec3::new(0., -10., -3.)),
        },
    ));
}

trait EguiExtras {
    fn matrix_drag(
        &mut self,
        id: impl Into<CtrlId>,
        s: &mut CtrlsState,
        value: &mut f32,
        default_value: f32,
        hover_text: impl Into<String>,
    );
}

impl EguiExtras for Ui {
    #[inline]
    fn matrix_drag(
        &mut self,
        id: impl Into<CtrlId>,
        s: &mut CtrlsState,
        sync_value: &mut f32,
        default_value: f32,
        hover_text: impl Into<String>,
    ) {
        let id: CtrlId = id.into();
        if !s.0.contains_key(&id) {
            s.0.insert(
                id.clone(),
                CtrlState {
                    value: default_value,
                    default_value,
                    ..Default::default()
                },
            );
        }
        let ctrl_state = s.0.get_mut(&id).expect("Wha! How? O_o");
        let rep_seg = ctrl_state.mode.get_str();
        let hover_text: String = hover_text.into();
        let drag = DragValue::new(&mut ctrl_state.value)
            .speed(0.02)
            .prefix(rep_seg)
            .fixed_decimals(2);
        let handle = self.add(drag);
        if handle.changed() {
            ctrl_state.is_changed = true;
        }
        if ctrl_state.is_changed {
            ctrl_state.value = ctrl_state.mode.run_mode(ctrl_state.value);
            ctrl_state.is_changed = false;
        }

        handle.on_hover_text(hover_text).context_menu(|ui| {
            for possible_modes in CtrlMode::iter() {
                if ui
                    .radio_value(
                        &mut ctrl_state.mode,
                        possible_modes.clone(),
                        possible_modes.to_string(),
                    )
                    .clicked()
                {
                    ctrl_state.mode = possible_modes;
                    ui.close_menu();
                }
            }
        });
        *sync_value = ctrl_state.value;
    }
}

#[inline]
fn mat4_ui<'a>(ui: &mut Ui, ui_state: &mut UiState, value: &mut Mat4) {
    let s = &mut ui_state.ctrls_state;
    ui.strong("Direct Matrix Control");
    ui.group(|ui| {
        egui::Grid::new("mat4_grid").show(ui, |ui| {
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "row");
            ui.colored_label(egui::Color32::GREEN, "i-hat");
            ui.colored_label(egui::Color32::RED, "j-hat");
            ui.colored_label(egui::Color32::from_rgb(0, 128, 128), "k-hat");
            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "trans");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "X");
            ui.matrix_drag(0, s, &mut value.x_axis.x, 1.0, "x_axis.x");
            ui.matrix_drag(1, s, &mut value.x_axis.y, 0., "x_axis.y");
            ui.matrix_drag(2, s, &mut value.x_axis.z, 0., "x_axis.z");
            ui.matrix_drag(3, s, &mut value.w_axis.x, 0., "w_axis.x");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Y");
            ui.matrix_drag(4, s, &mut value.y_axis.x, 0., "y_axis.x");
            ui.matrix_drag(5, s, &mut value.y_axis.y, 1.0, "y_axis.y");
            ui.matrix_drag(6, s, &mut value.y_axis.z, 0., "y_axis.z");
            ui.matrix_drag(7, s, &mut value.w_axis.y, 0., "w_axis.y");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "Z");
            ui.matrix_drag(8, s, &mut value.z_axis.x, 0., "z_axis.x");
            ui.matrix_drag(9, s, &mut value.z_axis.y, 0., "z_axis.y");
            ui.matrix_drag(10, s, &mut value.z_axis.z, 1.0, "z_axis.z");
            ui.matrix_drag(11, s, &mut value.w_axis.z, 0., "w_axis.z");
            ui.end_row();

            ui.colored_label(egui::Color32::from_rgb(128, 128, 64), "W");
            ui.matrix_drag(12, s, &mut value.x_axis.w, 0., "x_axis.w");
            ui.matrix_drag(13, s, &mut value.y_axis.w, 0., "y_axis.w");
            ui.matrix_drag(14, s, &mut value.z_axis.w, 0., "z_axis.w");
            ui.matrix_drag(15, s, &mut value.w_axis.w, 1.0, "w_axis.w");
            ui.end_row();
        });
    });
    ui.horizontal(|ui| {
        if ui.button("Reset All").clicked() {
            *s = CtrlsState::default();
        }
        if ui.button("Reset Matrix").clicked() {
            s.reset_values();
        }
        if ui.button("Reset Mode").clicked() {
            s.reset_modes();
        }
    });
    ui.separator();
    ui.strong("High-level Controls");
    ui.horizontal(|ui| {
        let label = ui.label("Theta");
        let handle = ui
            .add(
                DragValue::new(&mut ui_state.theta)
                    .speed(0.01)
                    .clamp_range(-FOUR_PI..=FOUR_PI),
            )
            .labelled_by(label.id);
        if handle.changed() {
            for (_, state) in ui_state.ctrls_state.0.iter_mut() {
                match state.mode {
                    CtrlMode::Normal => {}
                    _ => {
                        state.value = ui_state.theta;
                        state.is_changed = true;
                    }
                }
            }
        }
    });
    ui.separator();
    ui.strong("Matrix Info");
    ui.label(format!("Determinant: {}", value.determinant()))
        .on_hover_text("The change in volume applied by this transform (ignoring w_axis).");
}

fn ui_status(
    mut ctx: EguiContexts,
    time: Res<Time>,
    frame_count: Res<FrameCount>,
    egui_settings: Res<EguiSettings>,
    mut status: ResMut<PgmStatus>,
    // mut ui_state: ResMut<UiState>,
    loading: Option<Res<AssetsLoading>>,
    server: Res<AssetServer>,
    mut wnd_state: ResMut<WndState>,
    mut commands: Commands, // mut ui_state: ResMut<UiState>,
) {
    let delta_frame_count = frame_count.0 - status.last_frame_count;
    status.last_frame_count = frame_count.0;
    status.update_timer.tick(time.delta());
    if status.update_timer.finished() {
        let t = time.raw_delta_seconds_f64();
        let fps = delta_frame_count as f64 / t;
        status.last_fps = fps;
    }
    egui::Window::new("Status")
        .open(&mut wnd_state.is_open_status_wnd)
        .resize(|r| r.default_size(bevy_egui::egui::Vec2::ZERO))
        .show(ctx.ctx_mut(), |ui| {
            ui.label(format!("FPS: {:.2}", status.last_fps));
            ui.separator();

            ui.label(format!("Scale factor: {:.2}", egui_settings.scale_factor));
            if let Some(loading) = loading {
                ui.separator();
                match server.get_group_load_state(loading.0.iter().map(|h| h.id())) {
                    bevy::asset::LoadState::Loaded => {
                        commands.remove_resource::<AssetsLoading>();
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new());
                            ui.label("Still loading assets...");
                        });
                    }
                }
            }
        });
}

fn keyboard_input(keys: Res<Input<KeyCode>>, mut egui_settings: ResMut<EguiSettings>) {
    if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keys.any_just_pressed([KeyCode::Equals, KeyCode::Plus]) {
            egui_settings.scale_factor = (egui_settings.scale_factor + 0.2).clamp(0.5, 3.5);
        }
        if keys.any_just_pressed([KeyCode::Key0, KeyCode::Numpad0]) {
            egui_settings.scale_factor = 1.2;
        }
        if keys.any_just_pressed([KeyCode::Minus]) {
            egui_settings.scale_factor = (egui_settings.scale_factor - 0.2).clamp(0.5, 3.5);
        }
    }
}

fn window_help(mut ctx: EguiContexts, mut wnd_state: ResMut<WndState>) {
    egui::Window::new("Help")
        .open(&mut wnd_state.is_open_help_wnd)
        .show(ctx.ctx_mut(), |ui| {
            ui.label("Ctrl+Plus to increase UI size.");
            ui.label("Ctrl+Minus to decrease UI size.");
        });
}

fn window_view(mut wnd_state: ResMut<WndState>, mut ctx: EguiContexts) {
    egui::Window::new("View").show(ctx.ctx_mut(), |ui| {
        ui.checkbox(&mut wnd_state.is_open_help_wnd, "Show Help Window");
        ui.checkbox(&mut wnd_state.is_open_ctrl_wnd, "Show Control Window");
        ui.checkbox(&mut wnd_state.is_open_status_wnd, "Show Status Window");
    });
}

fn window_ctrl(
    mut transformable: Query<(&mut Transform, &Transformable)>,
    mut ui_state: ResMut<UiState>,
    mut wnd_state: ResMut<WndState>,
    mut ctx: EguiContexts,
    mut ambient_light: ResMut<AmbientLight>,
) {
    egui::Window::new("Controls")
        .open(&mut wnd_state.is_open_ctrl_wnd)
        .resize(|r| r.default_size(bevy_egui::egui::Vec2::ZERO))
        .show(ctx.ctx_mut(), |ui| {
            // Moooooom. The borrow checker is bullying me Y~Y
            let mut cloned_ui_mat = ui_state.mat_transform;
            mat4_ui(ui, &mut ui_state, &mut cloned_ui_mat);
            ui_state.mat_transform = cloned_ui_mat;
            ambient_light.brightness = ui_state.ambient_brightness;
            ui.separator();
            ui.strong("Display Settings");
            ui.horizontal(|ui| {
                let label = ui.label("Ambient Brightness:");
                ui.add(DragValue::new(&mut ui_state.ambient_brightness).speed(0.001))
                    .labelled_by(label.id);
            });
        });

    for (mut transform, transformable) in &mut transformable {
        *transform = transformable.transform * Transform::from_matrix(ui_state.mat_transform);
    }
}
