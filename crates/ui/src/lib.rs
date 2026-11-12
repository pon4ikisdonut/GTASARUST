//! Authentic GTA: San Andreas HUD, radar, menus, map screen, save/load slots,
//! and `egui` developer diagnostics overlay.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Stroke, TextureId, Vec2};
use gtasarust_engine::{EngineConfig, GameClock};
use gtasarust_entities::{PlayerEntity, PlayerLifeState};
use gtasarust_localization::LocalizationTable;
use gtasarust_missions::MissionManager;
use gtasarust_save_system::MAX_SAVE_SLOTS;
use gtasarust_vehicles::VehicleManager;
use gtasarust_world::{WeatherType, WorldManager};
use std::collections::HashMap;

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

/// CPU + lazy GPU-uploaded San Andreas HUD, splash backdrop, and `radarXX.txd` map tiles.
pub struct UiMediaCache {
    pub raw_backdrop: Option<(u32, u32, Vec<u8>)>,
    pub raw_hud_sprites: HashMap<String, (u32, u32, Vec<u8>)>,
    pub raw_radar_tiles: Vec<Option<(u32, u32, Vec<u8>)>>,
    backdrop_tex: Option<egui::TextureHandle>,
    hud_tex: HashMap<String, egui::TextureHandle>,
    radar_tex: [Option<egui::TextureHandle>; 144],
}

impl Default for UiMediaCache {
    fn default() -> Self {
        Self {
            raw_backdrop: None,
            raw_hud_sprites: HashMap::new(),
            raw_radar_tiles: vec![None; 144],
            backdrop_tex: None,
            hud_tex: HashMap::new(),
            radar_tex: [const { None }; 144],
        }
    }
}

impl UiMediaCache {
    fn backdrop_id(&mut self, ctx: &egui::Context) -> Option<TextureId> {
        if self.backdrop_tex.is_none()
            && let Some((w, h, rgba)) = &self.raw_backdrop
            && *w > 0
            && *h > 0
        {
            let img = egui::ColorImage::from_rgba_unmultiplied([*w as usize, *h as usize], rgba);
            self.backdrop_tex =
                Some(ctx.load_texture("sa_backdrop", img, egui::TextureOptions::LINEAR));
        }
        self.backdrop_tex.as_ref().map(|t| t.id())
    }

    fn hud_sprite_id(&mut self, ctx: &egui::Context, name: &str) -> Option<TextureId> {
        let key = name.to_ascii_lowercase();
        if !self.hud_tex.contains_key(&key)
            && let Some((w, h, rgba)) = self.raw_hud_sprites.get(&key)
            && *w > 0
            && *h > 0
        {
            let img = egui::ColorImage::from_rgba_unmultiplied([*w as usize, *h as usize], rgba);
            let hnd = ctx.load_texture(format!("hud_{key}"), img, egui::TextureOptions::LINEAR);
            self.hud_tex.insert(key.clone(), hnd);
        }
        self.hud_tex.get(&key).map(|t| t.id())
    }

    fn radar_tile_id(&mut self, ctx: &egui::Context, tile_idx: usize) -> Option<TextureId> {
        if tile_idx >= 144 {
            return None;
        }
        if self.radar_tex[tile_idx].is_none()
            && let Some(Some((w, h, rgba))) = self.raw_radar_tiles.get(tile_idx)
            && *w > 0
            && *h > 0
        {
            let img = egui::ColorImage::from_rgba_unmultiplied([*w as usize, *h as usize], rgba);
            let mut opts = egui::TextureOptions::LINEAR;
            opts.wrap_mode = egui::TextureWrapMode::ClampToEdge;
            self.radar_tex[tile_idx] =
                Some(ctx.load_texture(format!("radar_{tile_idx:02}"), img, opts));
        }
        self.radar_tex[tile_idx].as_ref().map(|t| t.id())
    }
}

pub struct GameUiState {
    pub screen: ScreenState,
    pub show_dev_overlay: bool,
    pub status_message: Option<String>,
    pub media: UiMediaCache,
}

impl Default for GameUiState {
    fn default() -> Self {
        Self {
            screen: ScreenState::Playing,
            show_dev_overlay: false,
            status_message: None,
            media: UiMediaCache::default(),
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
            draw_menu_backdrop(
                ctx,
                &mut ui_state.media,
                "GRAND THEFT AUTO: SAN ANDREAS — GTASARUST",
            );
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
                        if menu_btn(ui, "RESUME FREE ROAM") {
                            actions.resume_game = true;
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
            draw_menu_backdrop(ctx, &mut ui_state.media, "PAUSED");
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
                    &mut ui_state.media,
                    clock,
                    player,
                    vehicles,
                    world,
                    missions,
                    radio_name,
                    ui_state.status_message.as_deref(),
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

fn draw_menu_backdrop(ctx: &egui::Context, media: &mut UiMediaCache, title: &str) {
    let screen = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    if let Some(tex_id) = media.backdrop_id(ctx) {
        painter.image(
            tex_id,
            screen,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::from_rgb(200, 200, 200),
        );
        painter.rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(6, 8, 12, 130));
    } else {
        painter.rect_filled(
            screen,
            0.0,
            Color32::from_rgba_unmultiplied(10, 12, 16, 205),
        );
    }
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
    media: &mut UiMediaCache,
    clock: &GameClock,
    player: &PlayerEntity,
    vehicles: &VehicleManager,
    world: &WorldManager,
    missions: &MissionManager,
    radio_name: &str,
    status_message: Option<&str>,
    show_radar: bool,
) {
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::unique("sa_hud"),
    ));

    if let Some(msg) = status_message {
        let help_box = Rect::from_min_size(Pos2::new(26.0, 26.0), Vec2::new(360.0, 44.0));
        painter.rect_filled(
            help_box,
            CornerRadius::same(4),
            Color32::from_rgba_unmultiplied(8, 10, 14, 200),
        );
        painter.text(
            help_box.left_center() + Vec2::new(12.0, 0.0),
            Align2::LEFT_CENTER,
            msg,
            FontId::proportional(15.0),
            Color32::from_rgb(238, 238, 238),
        );
    }

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
        let is_unarmed = w.spec.name.eq_ignore_ascii_case("Fist");
        if is_unarmed && let Some(fist_id) = media.hud_sprite_id(ctx, "fist") {
            painter.image(
                fist_id,
                w_box.shrink(8.0),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.text(
                w_box.center() - Vec2::new(0.0, 6.0),
                Align2::CENTER_CENTER,
                &w.spec.name,
                FontId::proportional(14.0),
                Color32::WHITE,
            );
        }
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

    // Bottom-left circular radar using real `radar00..radar143` tiles + `radardisc`
    if show_radar {
        let center = Pos2::new(104.0, rect.max.y - 104.0);
        let radius = 76.0;
        let radar_range_m = 210.0f32;
        let scale = radius / radar_range_m;
        let yaw = player.camera_yaw;
        let cam_fwd = glam::Vec2::new(yaw.cos(), yaw.sin());
        let cam_right = glam::Vec2::new(yaw.sin(), -yaw.cos());
        let world_to_radar = |wx: f32, wy: f32| -> Pos2 {
            let d = glam::Vec2::new(wx - player.position.x, wy - player.position.y);
            Pos2::new(
                center.x + d.dot(cam_right) * scale,
                center.y - d.dot(cam_fwd) * scale,
            )
        };

        painter.circle_filled(center, radius - 2.0, Color32::from_rgb(48, 86, 118));

        let min_col = (((player.position.x - radar_range_m - 20.0) + 3000.0) / 500.0)
            .floor()
            .clamp(0.0, 11.0) as usize;
        let max_col = (((player.position.x + radar_range_m + 20.0) + 3000.0) / 500.0)
            .floor()
            .clamp(0.0, 11.0) as usize;
        let min_row = ((3000.0 - (player.position.y + radar_range_m + 20.0)) / 500.0)
            .floor()
            .clamp(0.0, 11.0) as usize;
        let max_row = ((3000.0 - (player.position.y - radar_range_m - 20.0)) / 500.0)
            .floor()
            .clamp(0.0, 11.0) as usize;

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                let tile_idx = row * 12 + col;
                if let Some(tex_id) = media.radar_tile_id(ctx, tile_idx) {
                    let x0 = -3000.0 + (col as f32) * 500.0;
                    let x1 = x0 + 500.0;
                    let y0 = 3000.0 - (row as f32) * 500.0;
                    let y1 = y0 - 500.0;
                    let quad = [
                        (world_to_radar(x0, y0), Pos2::new(0.0, 0.0)),
                        (world_to_radar(x1, y0), Pos2::new(1.0, 0.0)),
                        (world_to_radar(x1, y1), Pos2::new(1.0, 1.0)),
                        (world_to_radar(x0, y1), Pos2::new(0.0, 1.0)),
                    ];
                    if let Some(mesh) =
                        clip_quad_to_circle_mesh(&quad, center, radius - 3.5, tex_id)
                    {
                        painter.add(egui::Shape::mesh(mesh));
                    }
                }
            }
        }

        if let Some(disc_id) = media.hud_sprite_id(ctx, "radardisc") {
            let disc_rect = Rect::from_center_size(center, Vec2::splat(radius * 2.08));
            painter.image(
                disc_id,
                disc_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.circle_stroke(center, radius, Stroke::new(4.0, Color32::BLACK));
            painter.circle_stroke(
                center,
                radius - 2.0,
                Stroke::new(1.5, Color32::from_rgb(180, 180, 180)),
            );
        }

        // North blip on radar perimeter
        let north_pos = Pos2::new(
            center.x + cam_right.y * (radius - 2.0),
            center.y - cam_fwd.y * (radius - 2.0),
        );
        if let Some(north_id) = media.hud_sprite_id(ctx, "radar_north") {
            painter.image(
                north_id,
                Rect::from_center_size(north_pos, Vec2::splat(16.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.text(
                north_pos,
                Align2::CENTER_CENTER,
                "N",
                FontId::monospace(12.0),
                Color32::WHITE,
            );
        }

        // Nearby vehicles on radar
        for v in &vehicles.vehicles {
            let p = world_to_radar(v.position.x, v.position.y);
            if p.distance(center) < radius - 9.0 {
                painter.rect_filled(
                    Rect::from_center_size(p, Vec2::splat(4.5)),
                    0.0,
                    Color32::from_rgb(140, 210, 255),
                );
            }
        }
        // Nearby ENEX doorways matching current interior
        for e in &world.entry_exits {
            if e.target_interior != player.interior {
                continue;
            }
            let p = world_to_radar(e.entrance[0], e.entrance[1]);
            if p.distance(center) < radius - 9.0 {
                painter.rect_filled(
                    Rect::from_center_size(p, Vec2::splat(5.0)),
                    0.0,
                    Color32::from_rgb(255, 210, 40),
                );
            }
        }
        // Mission target blip
        if let Some(target) = missions.current_target_position() {
            let p = world_to_radar(target.x, target.y);
            let delta = p - center;
            let dist = delta.length();
            let clamped = if dist > radius - 6.0 {
                center + delta * ((radius - 6.0) / dist.max(1e-3))
            } else {
                p
            };
            if let Some(cj_id) = media.hud_sprite_id(ctx, "radar_sweet") {
                painter.image(
                    cj_id,
                    Rect::from_center_size(clamped, Vec2::splat(16.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(clamped, 5.0, Color32::from_rgb(245, 205, 45));
            }
        }
        // Player centre blip (rotated triangle pointing up along camera heading)
        if let Some(centre_id) = media.hud_sprite_id(ctx, "radar_centre") {
            painter.image(
                centre_id,
                Rect::from_center_size(center, Vec2::splat(14.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.circle_filled(center, 4.5, Color32::WHITE);
        }
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
    draw_menu_backdrop(ctx, &mut ui_state.media, "SAN ANDREAS MAP");
    egui::Window::new("San Andreas Map")
        .anchor(Align2::CENTER_CENTER, [0.0, 20.0])
        .resizable(false)
        .show(ctx, |ui| {
            let (resp, painter) =
                ui.allocate_painter(Vec2::new(480.0, 480.0), egui::Sense::hover());
            let r = resp.rect;
            painter.rect_filled(r, 4.0, Color32::from_rgb(25, 48, 72));

            let tile_w = r.width() / 12.0;
            let tile_h = r.height() / 12.0;
            for row in 0..12usize {
                for col in 0..12usize {
                    if let Some(tex_id) = ui_state.media.radar_tile_id(ctx, row * 12 + col) {
                        let trect = Rect::from_min_size(
                            Pos2::new(r.min.x + col as f32 * tile_w, r.min.y + row as f32 * tile_h),
                            Vec2::new(tile_w + 0.4, tile_h + 0.4),
                        );
                        painter.image(
                            tex_id,
                            trect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                }
            }

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
                    Color32::from_rgb(235, 240, 225),
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
            let cj_pos = map_pt(player.position.x, player.position.y);
            if let Some(cj_id) = ui_state.media.hud_sprite_id(ctx, "radar_cj") {
                painter.image(
                    cj_id,
                    Rect::from_center_size(cj_pos, Vec2::splat(16.0)),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.circle_filled(cj_pos, 6.0, Color32::from_rgb(255, 60, 60));
            }

            ui.horizontal(|ui| {
                if ui.button("Teleport: Los Santos").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(2495.3, -1688.2, 14.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Teleport: San Fierro").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(-2026.0, 156.0, 30.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Teleport: Las Venturas").clicked() {
                    actions.teleport_to = Some(glam::Vec3::new(2170.0, 1285.0, 12.0));
                    ui_state.screen = ScreenState::Playing;
                }
                if ui.button("Back").clicked() {
                    ui_state.screen = ScreenState::Paused;
                }
            });
        });
}

/// Sutherland-Hodgman clip a rotated map tile quad against an inscribed 16-gon circle.
fn clip_quad_to_circle_mesh(
    quad: &[(Pos2, Pos2); 4],
    center: Pos2,
    radius: f32,
    texture_id: TextureId,
) -> Option<egui::Mesh> {
    let mut poly: Vec<(Pos2, Pos2)> = quad.to_vec();
    const EDGES: usize = 16;
    for i in 0..EDGES {
        if poly.is_empty() {
            return None;
        }
        let a0 = (i as f32 / EDGES as f32) * std::f32::consts::TAU;
        let a1 = ((i + 1) as f32 / EDGES as f32) * std::f32::consts::TAU;
        let p0 = Pos2::new(center.x + radius * a0.cos(), center.y + radius * a0.sin());
        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        // Inward half-plane relative to circle center
        let ex = p1.x - p0.x;
        let ey = p1.y - p0.y;
        let center_side = ex * (center.y - p0.y) - ey * (center.x - p0.x);
        let sign = if center_side >= 0.0 { 1.0 } else { -1.0 };
        let signed_dist = |pt: Pos2| -> f32 { (ex * (pt.y - p0.y) - ey * (pt.x - p0.x)) * sign };

        let mut next = Vec::with_capacity(poly.len() + 2);
        let len = poly.len();
        for j in 0..len {
            let (curr_p, curr_uv) = poly[j];
            let (prev_p, prev_uv) = poly[(j + len - 1) % len];
            let dc = signed_dist(curr_p);
            let dp = signed_dist(prev_p);
            if dc >= 0.0 {
                if dp < 0.0 {
                    let t = (dp / (dp - dc)).clamp(0.0, 1.0);
                    next.push((
                        Pos2::new(
                            prev_p.x + (curr_p.x - prev_p.x) * t,
                            prev_p.y + (curr_p.y - prev_p.y) * t,
                        ),
                        Pos2::new(
                            prev_uv.x + (curr_uv.x - prev_uv.x) * t,
                            prev_uv.y + (curr_uv.y - prev_uv.y) * t,
                        ),
                    ));
                }
                next.push((curr_p, curr_uv));
            } else if dp >= 0.0 {
                let t = (dp / (dp - dc)).clamp(0.0, 1.0);
                next.push((
                    Pos2::new(
                        prev_p.x + (curr_p.x - prev_p.x) * t,
                        prev_p.y + (curr_p.y - prev_p.y) * t,
                    ),
                    Pos2::new(
                        prev_uv.x + (curr_uv.x - prev_uv.x) * t,
                        prev_uv.y + (curr_uv.y - prev_uv.y) * t,
                    ),
                ));
            }
        }
        poly = next;
    }

    if poly.len() < 3 {
        return None;
    }
    let mut mesh = egui::Mesh::with_texture(texture_id);
    for &(pos, uv) in &poly {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv,
            color: Color32::WHITE,
        });
    }
    for i in 1..(poly.len() as u32 - 1) {
        mesh.indices.extend_from_slice(&[0, i, i + 1]);
    }
    Some(mesh)
}

fn draw_settings_screen(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    config: &mut EngineConfig,
) {
    draw_menu_backdrop(ctx, &mut ui_state.media, "SETTINGS");
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
    draw_menu_backdrop(
        ctx,
        &mut ui_state.media,
        if is_save { "SAVE GAME" } else { "LOAD GAME" },
    );
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
                "Pos: ({:.1}, {:.1}, {:.1}) | Int: {} | Mode: {:?} | Anim: {:?}",
                player.position.x,
                player.position.y,
                player.position.z,
                player.interior,
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
