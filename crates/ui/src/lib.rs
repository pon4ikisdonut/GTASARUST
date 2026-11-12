//! Authentic GTA: San Andreas HUD, radar, menus, map screen, save/load slots,
//! and `egui` developer diagnostics overlay.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Stroke, Vec2};
use gtasarust_engine::{EngineConfig, GameClock};
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_localization::LocalizationTable;
use gtasarust_missions::MissionManager;
use gtasarust_save_system::MAX_SAVE_SLOTS;
use gtasarust_vehicles::VehicleManager;
use gtasarust_world::{WeatherType, WorldManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    MainMenu,
    Playing,
    Paused,
    Map,
    Settings,
    SaveGame,
    LoadGame,
}

#[derive(Debug, Default)]
pub struct UiActionQueue {
    pub start_new_game: bool,
    pub resume_game: bool,
    pub quit_requested: bool,
    pub save_to_slot: Option<usize>,
    pub load_from_slot: Option<usize>,
    pub set_weather: Option<WeatherType>,
    pub set_hour: Option<u8>,
    pub add_wanted_star: bool,
    pub clear_wanted: bool,
    pub teleport_to: Option<glam::Vec3>,
}

pub struct GameUiState {
    pub screen: ScreenState,
    pub show_dev_overlay: bool,
    pub status_message: Option<String>,
}

impl Default for GameUiState {
    fn default() -> Self {
        Self {
            screen: ScreenState::Playing,
            show_dev_overlay: false,
            status_message: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_game_ui(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    config: &mut EngineConfig,
    clock: &GameClock,
    player: &PlayerEntity,
    vehicles: &VehicleManager,
    world: &WorldManager,
    missions: &MissionManager,
    loc: &LocalizationTable,
    fps: f32,
    backend_name: &str,
    adapter_name: &str,
    radio_name: &str,
) -> UiActionQueue {
    let mut actions = UiActionQueue::default();

    match ui_state.screen {
        ScreenState::MainMenu => {
            draw_menu_backdrop(ctx, "GRAND THEFT AUTO: SAN ANDREAS — GTASARUST");
            egui::Window::new("main_menu")
                .title_bar(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
                .show(ctx, |ui| {
                    ui.set_min_width(300.0);
                    ui.vertical_centered(|ui| {
                        if menu_btn(ui, loc.get("MAIN", "MENU_NEW")) {
                            actions.start_new_game = true;
                            ui_state.screen = ScreenState::Playing;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_LOAD")) {
                            ui_state.screen = ScreenState::LoadGame;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_OPTS")) {
                            ui_state.screen = ScreenState::Settings;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_QUIT")) {
                            actions.quit_requested = true;
                        }
                    });
                });
        }
        ScreenState::Paused => {
            draw_menu_backdrop(ctx, "PAUSED");
            egui::Window::new("pause_menu")
                .title_bar(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
                .show(ctx, |ui| {
                    ui.set_min_width(300.0);
                    ui.vertical_centered(|ui| {
                        if menu_btn(ui, "RESUME") {
                            ui_state.screen = ScreenState::Playing;
                            actions.resume_game = true;
                        }
                        if menu_btn(ui, "MAP") {
                            ui_state.screen = ScreenState::Map;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_SAVE")) {
                            ui_state.screen = ScreenState::SaveGame;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_LOAD")) {
                            ui_state.screen = ScreenState::LoadGame;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_OPTS")) {
                            ui_state.screen = ScreenState::Settings;
                        }
                        if menu_btn(ui, "MAIN MENU") {
                            ui_state.screen = ScreenState::MainMenu;
                        }
                        if menu_btn(ui, loc.get("MAIN", "MENU_QUIT")) {
                            actions.quit_requested = true;
                        }
                    });
                });
        }
        ScreenState::Map => {
            draw_map_screen(ctx, ui_state, player, missions, &mut actions);
        }
        ScreenState::Settings => {
            draw_settings_screen(ctx, ui_state, config);
        }
        ScreenState::SaveGame | ScreenState::LoadGame => {
            draw_save_load_screen(ctx, ui_state, &mut actions);
        }
        ScreenState::Playing => {
            if config.game.show_hud {
                draw_sa_hud(
                    ctx,
                    clock,
                    player,
                    vehicles,
                    world,
                    missions,
                    radio_name,
                    config.game.show_radar,
                );
            }
        }
    }

    if ui_state.show_dev_overlay {
        draw_dev_inspector(
            ctx,
            ui_state,
            clock,
            player,
            vehicles,
            world,
            missions,
            fps,
            backend_name,
            adapter_name,
            &mut actions,
        );
    }

    actions
}

fn menu_btn(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add_sized(
        [260.0, 34.0],
        egui::Button::new(RichText::new(label).size(18.0).strong()),
    )
    .clicked()
}

fn draw_menu_backdrop(ctx: &egui::Context, title: &str) {
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    painter.rect_filled(
        screen,
        0.0,
        Color32::from_rgba_unmultiplied(10, 12, 16, 200),
    );
    painter.text(
        Pos2::new(screen.center().x, 90.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(30.0),
        Color32::from_rgb(240, 195, 95),
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_sa_hud(
    ctx: &egui::Context,
    clock: &GameClock,
    player: &PlayerEntity,
    vehicles: &VehicleManager,
    world: &WorldManager,
    missions: &MissionManager,
    radio_name: &str,
    show_radar: bool,
) {
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::unique("sa_hud"),
    ));

    // Top-right GTA:SA cluster: Weapon box + Clock + Money + Health + Armor + Wanted stars
    let right_x = rect.max.x - 28.0;
    let top_y = 26.0;

    // Weapon slot box
    let w_box = Rect::from_min_size(Pos2::new(right_x - 78.0, top_y), Vec2::new(78.0, 68.0));
    painter.rect_filled(
        w_box,
        CornerRadius::same(4),
        Color32::from_rgba_unmultiplied(0, 0, 0, 150),
    );
    painter.rect_stroke(
        w_box,
        CornerRadius::same(4),
        Stroke::new(2.0, Color32::from_rgb(210, 210, 210)),
        egui::StrokeKind::Inside,
    );
    if let Some(w) = player.weapons.active() {
        painter.text(
            w_box.center() - Vec2::new(0.0, 6.0),
            Align2::CENTER_CENTER,
            &w.spec.name,
            FontId::proportional(14.0),
            Color32::WHITE,
        );
        if w.spec.clip_size > 0 {
            painter.text(
                Pos2::new(w_box.center().x, w_box.max.y - 14.0),
                Align2::CENTER_CENTER,
                format!("{}-{}", w.reserve_ammo, w.ammo_in_clip),
                FontId::monospace(12.0),
                Color32::from_rgb(160, 220, 255),
            );
        }
    }

    // Clock
    painter.text(
        Pos2::new(right_x - 94.0, top_y + 10.0),
        Align2::RIGHT_TOP,
        clock.hud_string(),
        FontId::monospace(24.0),
        Color32::from_rgb(235, 235, 235),
    );

    // Health bar
    let bar_w = 135.0;
    let bar_x = right_x - 94.0 - bar_w;
    let hp_frac = (player.health / player.max_health.max(1.0)).clamp(0.0, 1.0);
    draw_sa_bar(
        &painter,
        Rect::from_min_size(Pos2::new(bar_x, top_y + 42.0), Vec2::new(bar_w, 12.0)),
        hp_frac,
        Color32::from_rgb(180, 25, 29),
    );

    // Armor bar (if > 0)
    if player.armor > 0.0 {
        let arm_frac = (player.armor / 100.0).clamp(0.0, 1.0);
        draw_sa_bar(
            &painter,
            Rect::from_min_size(Pos2::new(bar_x, top_y + 58.0), Vec2::new(bar_w, 10.0)),
            arm_frac,
            Color32::from_rgb(220, 228, 230),
        );
    }

    // Money `$00000350` in classic SA dollar green
    painter.text(
        Pos2::new(right_x, top_y + 78.0),
        Align2::RIGHT_TOP,
        format!("${:08}", player.money.max(0)),
        FontId::monospace(28.0),
        Color32::from_rgb(46, 138, 60),
    );

    // Wanted stars (6 stars)
    if player.wanted_stars > 0 {
        for i in 0..6u8 {
            let star_x = right_x - (f32::from(5 - i) * 26.0) - 10.0;
            let col = if i < player.wanted_stars {
                Color32::from_rgb(230, 195, 60)
            } else {
                Color32::from_rgba_unmultiplied(60, 60, 60, 140)
            };
            painter.text(
                Pos2::new(star_x, top_y + 112.0),
                Align2::CENTER_TOP,
                "★",
                FontId::proportional(24.0),
                col,
            );
        }
    }

    // Bottom-left circular radar
    if show_radar {
        let center = Pos2::new(98.0, rect.max.y - 98.0);
        let radius = 72.0;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(24, 34, 42, 210),
        );
        painter.circle_stroke(center, radius, Stroke::new(4.0, Color32::BLACK));
        painter.circle_stroke(
            center,
            radius - 2.0,
            Stroke::new(1.5, Color32::from_rgb(180, 180, 180)),
        );
        // North tick
        painter.text(
            center - Vec2::new(0.0, radius - 4.0),
            Align2::CENTER_CENTER,
            "N",
            FontId::monospace(12.0),
            Color32::WHITE,
        );
        // Nearby vehicles on radar
        let scale = 0.25f32;
        for v in &vehicles.vehicles {
            let rel = (v.position - player.position).truncate() * scale;
            if rel.length() < radius - 8.0 {
                painter.circle_filled(
                    Pos2::new(center.x + rel.x, center.y - rel.y),
                    3.0,
                    Color32::from_rgb(130, 200, 255),
                );
            }
        }
        // Mission target blip
        if let Some(target) = missions.current_target_position() {
            let rel = (target - player.position).truncate() * scale;
            let clamped = if rel.length() > radius - 6.0 {
                rel.normalize() * (radius - 6.0)
            } else {
                rel
            };
            painter.circle_filled(
                Pos2::new(center.x + clamped.x, center.y - clamped.y),
                5.0,
                Color32::from_rgb(245, 205, 45),
            );
        }
        // Player arrow
        painter.circle_filled(center, 4.5, Color32::WHITE);
    }

    // Bottom-right zone & vehicle/radio popup
    let zone = world.zone_name_at(player.position);
    painter.text(
        Pos2::new(rect.max.x - 28.0, rect.max.y - 34.0),
        Align2::RIGHT_BOTTOM,
        zone,
        FontId::proportional(24.0),
        Color32::from_rgb(220, 220, 220),
    );
    if let Some(vid) = player.current_vehicle
        && let Some(veh) = vehicles.get(vid)
    {
        painter.text(
            Pos2::new(rect.max.x - 28.0, rect.max.y - 66.0),
            Align2::RIGHT_BOTTOM,
            format!("{}  |  {}", veh.name, radio_name),
            FontId::proportional(18.0),
            Color32::from_rgb(160, 210, 120),
        );
    }

    // Mission objective / Wasted / Busted banners
    if player.life_state == PlayerLifeState::Wasted {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "WASTED",
            FontId::proportional(54.0),
            Color32::from_rgb(200, 40, 40),
        );
    } else if player.life_state == PlayerLifeState::Busted {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "BUSTED",
            FontId::proportional(54.0),
            Color32::from_rgb(70, 130, 230),
        );
    } else if let Some(banner) = &missions.banner_text {
        painter.text(
            Pos2::new(rect.center().x, rect.center().y - 80.0),
            Align2::CENTER_CENTER,
            banner,
            FontId::proportional(30.0),
            Color32::from_rgb(245, 195, 65),
        );
    }

    if let Some(obj) = missions.current_objective_text() {
        let clean = LocalizationTable::strip_sa_tags(obj);
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 36.0),
            Align2::CENTER_BOTTOM,
            clean,
            FontId::proportional(20.0),
            Color32::WHITE,
        );
    }
}

fn draw_sa_bar(painter: &egui::Painter, rect: Rect, fraction: f32, fill: Color32) {
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let inner = rect.shrink(2.0);
    let fill_rect = Rect::from_min_size(
        inner.min,
        Vec2::new(inner.width() * fraction.clamp(0.0, 1.0), inner.height()),
    );
    painter.rect_filled(fill_rect, 0.0, fill);
}

fn draw_map_screen(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    player: &PlayerEntity,
    missions: &MissionManager,
    actions: &mut UiActionQueue,
) {
    draw_menu_backdrop(ctx, "SAN ANDREAS MAP");
    egui::Window::new("San Andreas Map")
        .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
        .resizable(false)
        .show(ctx, |ui| {
            let (resp, painter) =
                ui.allocate_painter(Vec2::new(420.0, 420.0), egui::Sense::hover());
            let r = resp.rect;
            painter.rect_filled(r, 4.0, Color32::from_rgb(25, 48, 72));

            let map_pt = |wx: f32, wy: f32| -> Pos2 {
                let nx = (wx + 3000.0) / 6000.0;
                let ny = 1.0 - ((wy + 3000.0) / 6000.0);
                Pos2::new(r.min.x + nx * r.width(), r.min.y + ny * r.height())
            };

            // City labels
            for (name, x, y) in [
                ("LOS SANTOS", 1500.0, -1600.0),
                ("SAN FIERRO", -2100.0, 400.0),
                ("LAS VENTURAS", 2100.0, 1500.0),
            ] {
                painter.text(
                    map_pt(x, y),
                    Align2::CENTER_CENTER,
                    name,
                    FontId::proportional(14.0),
                    Color32::from_rgb(180, 210, 180),
                );
            }

            // Mission triggers
            for t in missions.available_triggers() {
                painter.circle_filled(
                    map_pt(t.position.x, t.position.y),
                    5.0,
                    Color32::from_rgb(245, 200, 50),
                );
            }

            // Player position
            painter.circle_filled(
                map_pt(player.position.x, player.position.y),
                6.0,
                Color32::from_rgb(255, 60, 60),
            );

            ui.horizontal(|ui| {
                if ui.button("Teleport: Los Santos").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(2495.3, -1688.2, 2.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Teleport: San Fierro").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(-2026.0, 156.0, 2.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Teleport: Las Venturas").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(2170.0, 1285.0, 2.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Back").clicked() {
                    ui_state.screen = ScreenState::Paused;
                }
            });
        });
}

fn draw_settings_screen(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    config: &mut EngineConfig,
) {
    draw_menu_backdrop(ctx, "SETTINGS");
    egui::Window::new("Options")
        .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.heading("Graphics");
            ui.checkbox(&mut config.graphics.vsync, "VSync");
            ui.add(
                egui::Slider::new(&mut config.graphics.draw_distance, 200.0..=3000.0)
                    .text("Draw Distance"),
            );
            ui.add(
                egui::Slider::new(&mut config.graphics.fov_degrees, 50.0..=110.0)
                    .text("Field of View"),
            );

            ui.separator();
            ui.heading("Audio");
            ui.add(egui::Slider::new(&mut config.audio.master_volume, 0.0..=1.0).text("Master"));
            ui.add(
                egui::Slider::new(&mut config.audio.music_volume, 0.0..=1.0).text("Music / Radio"),
            );
            ui.add(egui::Slider::new(&mut config.audio.sfx_volume, 0.0..=1.0).text("SFX"));
            ui.add(egui::Slider::new(&mut config.audio.voice_volume, 0.0..=1.0).text("Voice"));

            ui.separator();
            ui.heading("Controls & HUD");
            ui.add(
                egui::Slider::new(&mut config.controls.mouse_sensitivity, 0.2..=4.0)
                    .text("Mouse Sensitivity"),
            );
            ui.checkbox(&mut config.controls.invert_y, "Invert Mouse Y");
            ui.checkbox(&mut config.game.show_hud, "Show HUD");
            ui.checkbox(&mut config.game.show_radar, "Show Radar");

            if ui.button("Back").clicked() {
                ui_state.screen = ScreenState::Paused;
            }
        });
}

fn draw_save_load_screen(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    actions: &mut UiActionQueue,
) {
    let is_save = ui_state.screen == ScreenState::SaveGame;
    draw_menu_backdrop(ctx, if is_save { "SAVE GAME" } else { "LOAD GAME" });
    egui::Window::new("Save Slots")
        .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            for slot in 1..=MAX_SAVE_SLOTS {
                let label = if is_save {
                    format!("Save to Slot {slot}")
                } else {
                    format!("Load Slot {slot}")
                };
                if ui.button(label).clicked() {
                    if is_save {
                        actions.save_to_slot = Some(slot);
                    } else {
                        actions.load_from_slot = Some(slot);
                    }
                    ui_state.screen = ScreenState::Playing;
                }
            }
            if ui.button("Back").clicked() {
                ui_state.screen = ScreenState::Paused;
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_dev_inspector(
    ctx: &egui::Context,
    ui_state: &GameUiState,
    clock: &GameClock,
    player: &PlayerEntity,
    vehicles: &VehicleManager,
    world: &WorldManager,
    missions: &MissionManager,
    fps: f32,
    backend_name: &str,
    adapter_name: &str,
    actions: &mut UiActionQueue,
) {
    egui::Window::new("GTASARUST Dev Inspector (F1)")
        .default_pos([18.0, 18.0])
        .show(ctx, |ui| {
            ui.label(format!("FPS: {fps:.1} | Backend: {backend_name}"));
            ui.label(format!("Adapter: {adapter_name}"));
            ui.label(format!(
                "Pos: ({:.1}, {:.1}, {:.1}) | Mode: {:?} | Anim: {:?}",
                player.position.x,
                player.position.y,
                player.position.z,
                player.control_mode,
                player.anim.current
            ));
            ui.label(format!(
                "Original World: {} | Active Sectors: {} | Total Placements: {}",
                world.loaded_from_original_data,
                world.active_sectors.len(),
                world.total_placements()
            ));
            ui.label(format!(
                "Vehicles: {} | Completed Missions: {}",
                vehicles.vehicles.len(),
                missions.completed.len()
            ));
            if let Some(msg) = &ui_state.status_message {
                ui.colored_label(Color32::LIGHT_GREEN, msg);
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("08:00").clicked() {
                    actions.set_hour = Some(8);
                }
                if ui.button("18:30 (Dusk)").clicked() {
                    actions.set_hour = Some(18);
                }
                if ui.button("00:00 (Night)").clicked() {
                    actions.set_hour = Some(0);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Sunny LA").clicked() {
                    actions.set_weather = Some(WeatherType::ExtraSunnyLA);
                }
                if ui.button("Rainy").clicked() {
                    actions.set_weather = Some(WeatherType::Rainy);
                }
                if ui.button("Foggy").clicked() {
                    actions.set_weather = Some(WeatherType::Foggy);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("+1 Wanted Star").clicked() {
                    actions.add_wanted_star = true;
                }
                if ui.button("Clear Wanted").clicked() {
                    actions.clear_wanted = true;
                }
            });
            ui.label(format!("Clock: {} (Day {})", clock.hud_string(), clock.day));
        });
}
