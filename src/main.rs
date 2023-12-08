use bevy_panorbit_camera::{PanOrbitCameraPlugin, PanOrbitCamera};
use bevy_egui::{egui::{self, TextBuffer, Link, Hyperlink},EguiContexts,EguiPlugin};
use bevy::prelude::*;
use pdrust::{constraint::pulley::bundle::PulleyBundle, body::bundle::RigidBodyBundle};

use git_version::{git_version};
const GIT_VERSION: &str = git_version!();

#[derive(Resource, Debug, Component, PartialEq, Clone, Copy)]
struct DemonstrationSettings {
    pub m1: f32,
    pub m2: f32,
    pub mc: f32,
    pub l:  f32,
    pub x_0: f32,
}

#[derive(Event)]
struct RestartEvent;

impl Default for DemonstrationSettings {
    fn default() -> Self {
        Self {
            m1: 10.0,
            m2: 10.0,
            mc: 10.0,
            l: 10.0,
            x_0: -5.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(pdrust::PDRustPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(DemonstrationSettings::default())
        .add_systems(Startup, setup_camera_and_light)
        .add_systems(Update, demo_settings_ui)
        .add_event::<RestartEvent>()
        .add_systems(Update, restart_simulation)
        .run();
}

fn setup_camera_and_light(
    mut commands: Commands,
    ) {
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
}

fn demo_settings_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<DemonstrationSettings>,
    mut restart_event: EventWriter<RestartEvent>
    ) {
    egui::Window::new("Demonstration Settings").show(contexts.ctx_mut(), | ui | {
        ui.add(egui::Slider::new(&mut settings.m1, 5.0..=100.0).text("m1"));
        ui.add(egui::Slider::new(&mut settings.m2, 5.0..=100.0).text("m2"));
        ui.add(egui::Slider::new(&mut settings.mc, 5.0..=100.0).text("m_c"));
        ui.add(egui::Slider::new(&mut settings.l, 5.0..=20.0).text("L"));
        ui.add(egui::Slider::new(&mut settings.x_0, -5.0..=-10.0).text("x_0"));
        if ui.button("Start").clicked() {
            restart_event.send(RestartEvent);
        }

        ui.horizontal(|ui| {
            ui.label(format!("Git version:"));
            ui.add(Hyperlink::from_label_and_url(GIT_VERSION, format!("https://github.com/FEgor04/physics-project/tree/{}", GIT_VERSION)))
        })
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
    for ev in ev_restart.iter() {
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
