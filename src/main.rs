use bevy::{prelude::*, window::WindowTheme, ecs::query, render::extract_resource::ExtractResource};
use bevy_egui::{
    egui::{self, Hyperlink},
    EguiContexts, EguiPlugin,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use pdrust::settings::SettingsResource;
use pdrust::{body::bundle::RigidBodyBundle, constraint::pulley::bundle::PulleyBundle};

use git_version::git_version;
const GIT_VERSION: &str = git_version!();

#[derive(Resource, Debug, Component, PartialEq, Clone)]
struct DemonstrationSettings {
    pub m1: f32,
    pub m2: f32,
    pub mc: f32,
    pub l: f32,
    pub x_0: f32,
    pub enable_tracing: bool,
    tracing_material: Option<Handle<StandardMaterial>>,
    tracing_mesh: Option<Handle<Mesh>>,
}

#[derive(Event)]
struct RestartEvent;

#[derive(Event)]
struct CleanTraceEvent;

#[derive(Component)]
struct LeaveTrace;

#[derive(Component)]
struct MeshTrace;

impl Default for DemonstrationSettings {
    fn default() -> Self {
        Self {
            m1: 10.0,
            m2: 10.0,
            mc: 10.0,
            l: 10.0,
            x_0: -5.0,
            enable_tracing: false,
            tracing_material: None,
            tracing_mesh: None,
        }
    }
}

fn leave_trace_system(
    transforms: Query<&Transform, With<LeaveTrace>>,
    mut settings: ResMut<DemonstrationSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if settings.tracing_mesh.is_none() {
        settings.tracing_mesh = Some(meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.1,
            ..default()
        })));
    }

    if settings.tracing_material.is_none() {
        settings.tracing_material = Some(materials.add(StandardMaterial {
            base_color: Color::rgba(0.0, 1.0, 0.0, 0.5),
            alpha_mode: AlphaMode::Add,
            ..default()
        }));
    }

    if !settings.enable_tracing {
        return;
    }

    for t in &transforms {
        commands.spawn((
            PbrBundle {
                mesh: settings.tracing_mesh.clone().unwrap(),
                material: settings.tracing_material.clone().unwrap(),
                transform: *t,
                ..default()
            },
            MeshTrace,
        ));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "physics projects".into(),
                resolution: (1920., 1080.).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(pdrust::PDRustPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(DemonstrationSettings { ..default() })
        .add_event::<RestartEvent>()
        .add_event::<CleanTraceEvent>()
        .add_systems(Startup, setup_camera_and_light)
        .add_systems(Update, demo_settings_ui)
        .add_systems(Update, simulation_settings_ui.after(demo_settings_ui))
        .add_systems(Update, restart_simulation)
        .add_systems(FixedUpdate, leave_trace_system)
        .add_systems(Update, clean_trace)
        .run();
}

fn setup_camera_and_light(
    mut commands: Commands,
    mut restart_event: EventWriter<RestartEvent>,
    mut sim_settings: ResMut<SettingsResource>,
) {
    sim_settings.integration_substeps = 32;
    sim_settings.constraints_substeps = 32;
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, -10.0, 30.0)
                .looking_at(Vec3::from_array([0.0, 0.0, 0.0]), Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::from_array([0.0, -10.0, 0.0]),
            ..default()
        },
    ));

    restart_event.send(RestartEvent);
}

fn clean_trace(
    mut clean_trace_event: EventReader<CleanTraceEvent>,
    mut commands: Commands,
    query: Query<Entity, With<MeshTrace>>,
    ) {
    for _ in clean_trace_event.read() {
        query.for_each(|e| commands.entity(e).despawn());
    }
}

fn demo_settings_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<DemonstrationSettings>,
    mut restart_event: EventWriter<RestartEvent>,
    mut clean_trace_event: EventWriter<CleanTraceEvent>,
) {
    egui::Window::new("Demonstration Settings").show(contexts.ctx_mut(), |ui| {
        let l = settings.l;
        ui.add(egui::Slider::new(&mut settings.m1, 5.0..=100.0).text("m1"));
        ui.add(egui::Slider::new(&mut settings.m2, 5.0..=100.0).text("m2"));
        ui.add(egui::Slider::new(&mut settings.mc, 5.0..=100.0).text("m_c"));
        ui.add(egui::Slider::new(&mut settings.l, 5.0..=20.0).text("L"));
        ui.add(egui::Slider::new(&mut settings.x_0, -l..=l).text("x_0"));
        if ui.add(egui::Checkbox::new(
            &mut settings.enable_tracing,
            "Enable tracing (may cause severe perfomance loss!)",
        )).clicked() {
            if !settings.enable_tracing {
                clean_trace_event.send(CleanTraceEvent);
            }
        };
        if ui.button("Start").clicked() {
            restart_event.send(RestartEvent);
        }

        ui.horizontal(|ui| {
            ui.label(format!("Git version:"));
            ui.add(Hyperlink::from_label_and_url(
                GIT_VERSION,
                format!(
                    "https://github.com/FEgor04/physics-project/tree/{}",
                    GIT_VERSION
                ),
            ))
        })
    });
}

fn simulation_settings_ui(mut contexts: EguiContexts, mut settings: ResMut<SettingsResource>) {
    egui::Window::new("Simulation Settings").show(contexts.ctx_mut(), |ui| {
        ui.add(
            egui::Slider::new(&mut settings.integration_substeps, 1..=32)
                .text("Integration substeps"),
        );
        ui.add(
            egui::Slider::new(&mut settings.constraints_substeps, 1..=32)
                .text("Constraints integration substeps"),
        );
        ui.add(
            egui::Slider::new(&mut settings.baumgarte_constant, 0.0..=0.1)
                .text("Baumgarte constant"),
        );
        ui.add(
            egui::Slider::new(&mut settings.slow_motion_koef, 1.0..=16.0)
                .text("Slow Motion coefficient"),
        );
    });
}

fn restart_simulation(
    mut ev_restart: EventReader<RestartEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes_query: Query<Entity, With<Handle<Mesh>>>,
    settings: Res<DemonstrationSettings>,
) {
    for _ev in ev_restart.read() {
        for e in meshes_query.iter() {
            commands.entity(e).despawn();
        }

        let half_l = settings.l / 2.0;
        let m1 = settings.m1;
        let m2 = settings.m2;
        let m_central = settings.mc;

        let equilibrium_pos = Vec3::new(0.0, -half_l / f32::sqrt(3.0), 0.0);
        let equilibrium_offset: f32 = settings.x_0;
        let b_central_pos = equilibrium_pos + Vec3::new(0.0, equilibrium_offset, 0.0);

        let pulley1_pos = Vec3::new(-half_l, 0.0, 0.0);
        let pulley2_pos = Vec3::new(half_l, 0.0, 0.0);

        let constraint_distance = 3.0 * half_l;
        let vertical_offset = constraint_distance - (b_central_pos - pulley1_pos).length();
        let b1_pos = pulley1_pos + Vec3::new(0.0, -vertical_offset, 0.0);
        let b2_pos = pulley2_pos + Vec3::new(0.0, -vertical_offset, 0.0);

        let b1 = RigidBodyBundle::spawn_new_box(
            &mut commands,
            &mut meshes,
            materials.add(Color::RED.into()),
            m1,
            1.0,
            1.0,
            1.0,
            Transform::from_translation(b1_pos),
            Vec3::ZERO,
            Vec3::ZERO,
        );

        let b2 = RigidBodyBundle::spawn_new_box(
            &mut commands,
            &mut meshes,
            materials.add(Color::RED.into()),
            m2,
            1.0,
            1.0,
            1.0,
            Transform::from_translation(b2_pos),
            Vec3::ZERO,
            Vec3::ZERO,
        );

        let central_body = RigidBodyBundle::spawn_new_sphere(
            &mut commands,
            &mut meshes,
            materials.add(Color::GREEN.into()),
            m_central,
            0.5,
            Transform::from_translation(b_central_pos),
            Vec3::ZERO,
            Vec3::ZERO,
        );
        commands.entity(central_body).insert(LeaveTrace);

        PulleyBundle::spawn_new(
            &mut commands,
            &mut meshes,
            materials.add(Color::MIDNIGHT_BLUE.into()),
            materials.add(Color::MIDNIGHT_BLUE.into()),
            materials.add(Color::BEIGE.into()),
            b1,
            central_body,
            Vec3::ZERO,
            Vec3::ZERO,
            constraint_distance,
            pulley1_pos,
        );

        PulleyBundle::spawn_new(
            &mut commands,
            &mut meshes,
            materials.add(Color::MIDNIGHT_BLUE.into()),
            materials.add(Color::MIDNIGHT_BLUE.into()),
            materials.add(Color::BEIGE.into()),
            b2,
            central_body,
            Vec3::ZERO,
            Vec3::ZERO,
            constraint_distance,
            pulley2_pos,
        );

        let pulley_radius = 0.25;

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: pulley_radius,
                ..default()
            })),
            material: materials.add(Color::CYAN.into()),
            transform: Transform::from_translation(equilibrium_pos),
            ..default()
        });
    }
}
