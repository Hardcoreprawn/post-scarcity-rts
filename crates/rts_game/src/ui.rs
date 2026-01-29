//! UI plugin for game interface using egui.
//!
//! Provides resource HUD, minimap, selection panel, and command panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSet};
use rts_core::components::Command as CoreCommand;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::camera::MainCamera;
use crate::components::{
    AttackTarget, Building, BuildingType, CoreEntityId, GameCommandQueue, GameDepot, GameFaction,
    GameHealth, GamePosition, GameProductionQueue, PlayerFaction, Selected, UnitType,
};
use crate::construction::BuildingPlacement;
use crate::economy::PlayerResources;
use crate::input::{calculate_formation_offset, InputMode};
use crate::render::CommandFeedbackEvent;
use crate::simulation::{ClientCommandSet, CoreCommandBuffer};

/// Plugin for game UI using egui.
///
/// Provides:
/// - Resource HUD (top bar)
/// - Minimap (bottom-left)
/// - Selection panel (bottom-center)
/// - Command panel (bottom-right)
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<PlayerResources>()
            .init_resource::<PlayerFaction>()
            .init_resource::<UiSettings>()
            .init_resource::<InputMode>()
            .add_systems(Update, apply_ui_accessibility.after(EguiSet::InitContexts))
            .add_systems(
                Update,
                (
                    ui_resource_bar,
                    ui_minimap.in_set(ClientCommandSet::Gather),
                    ui_selection_panel,
                    ui_command_panel.in_set(ClientCommandSet::Gather),
                    ui_build_menu,
                )
                    .after(apply_ui_accessibility),
            );
    }
}

/// UI accessibility settings.
#[derive(Resource, Debug, Clone, Copy)]
pub struct UiSettings {
    /// UI scale factor applied to egui.
    pub ui_scale: f32,
    /// High-contrast mode toggle.
    pub high_contrast: bool,
    /// Minimum UI scale.
    pub min_scale: f32,
    /// Maximum UI scale.
    pub max_scale: f32,
    /// Increment for UI scale adjustments.
    pub scale_step: f32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            high_contrast: false,
            min_scale: 0.75,
            max_scale: 1.5,
            scale_step: 0.1,
        }
    }
}

fn apply_ui_accessibility(
    mut contexts: EguiContexts,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<UiSettings>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Equal) {
        settings.ui_scale = (settings.ui_scale + settings.scale_step).min(settings.max_scale);
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        settings.ui_scale = (settings.ui_scale - settings.scale_step).max(settings.min_scale);
    }
    if keyboard.just_pressed(KeyCode::F9) {
        settings.high_contrast = !settings.high_contrast;
    }

    apply_ui_settings(ctx, *settings);
}

fn apply_ui_settings(ctx: &egui::Context, settings: UiSettings) {
    ctx.set_pixels_per_point(settings.ui_scale);

    if settings.high_contrast {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::WHITE);
        visuals.widgets.inactive.bg_fill = egui::Color32::BLACK;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(30);
        visuals.widgets.active.bg_fill = egui::Color32::from_gray(60);
        visuals.window_fill = egui::Color32::BLACK;
        visuals.panel_fill = egui::Color32::BLACK;
        visuals.window_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
        visuals.faint_bg_color = egui::Color32::BLACK;
        ctx.set_visuals(visuals);
    } else {
        ctx.set_visuals(egui::Visuals::dark());
    }
}

/// Converts FactionId to egui Color32.
pub fn faction_to_egui_color(faction: FactionId) -> egui::Color32 {
    match faction {
        FactionId::Continuity => egui::Color32::from_rgb(51, 102, 204), // Blue
        FactionId::Collegium => egui::Color32::from_rgb(204, 153, 51),  // Gold
        FactionId::Tinkers => egui::Color32::from_rgb(153, 102, 51),    // Brown
        FactionId::BioSovereigns => egui::Color32::from_rgb(51, 178, 76), // Green
        FactionId::Zephyr => egui::Color32::from_rgb(153, 204, 229),    // Sky blue
    }
}

/// Returns the display name for a faction.
pub fn faction_name(faction: FactionId) -> &'static str {
    match faction {
        FactionId::Continuity => "Continuity Authority",
        FactionId::Collegium => "The Collegium",
        FactionId::Tinkers => "Tinkers' Union",
        FactionId::BioSovereigns => "The Sculptors",
        FactionId::Zephyr => "Zephyr Guild",
    }
}

/// Renders the top resource bar showing feedstock and supply.
fn ui_resource_bar(mut contexts: EguiContexts, resources: Res<PlayerResources>) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    egui::TopBottomPanel::top("resource_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 20.0;

            // Feedstock
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⛏")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(100, 200, 255)),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {}",
                        resources.feedstock, resources.feedstock_cap
                    ))
                    .size(16.0)
                    .strong(),
                );
            });

            ui.separator();

            // Supply
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("👥")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(100, 255, 100)),
                );
                let supply_color = if resources.supply_used >= resources.supply_cap {
                    egui::Color32::RED
                } else if resources.supply_used as f32 >= resources.supply_cap as f32 * 0.8 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::WHITE
                };
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {}",
                        resources.supply_used, resources.supply_cap
                    ))
                    .size(16.0)
                    .strong()
                    .color(supply_color),
                );
            });

            // Spacer to push game time to right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("0:00").size(14.0).weak());
                ui.label("Game Time:");
            });
        });
    });
}

/// Renders the minimap in the bottom-left corner.
fn ui_minimap(
    mut contexts: EguiContexts,
    units: Query<(&GamePosition, &GameFaction)>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    selected_units: Query<(Entity, &CoreEntityId), (With<Selected>, With<GameCommandQueue>)>,
    mut commands: Commands,
    input_mode: Res<InputMode>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut feedback_events: EventWriter<CommandFeedbackEvent>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    const MINIMAP_SIZE: f32 = 180.0;
    const WORLD_SIZE: f32 = 2000.0; // Assumed world bounds

    egui::Window::new("Minimap")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .fixed_size([MINIMAP_SIZE, MINIMAP_SIZE])
        .show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::Vec2::splat(MINIMAP_SIZE), egui::Sense::click());

            let rect = response.rect;

            // Background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 30, 20));

            // Border
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::GRAY));

            // Draw units as dots
            for (pos, faction) in units.iter() {
                let world_pos = pos.as_vec2();

                let minimap_pos = world_to_minimap(world_pos, rect, WORLD_SIZE);
                let color = faction_to_egui_color(faction.faction);
                painter.circle_filled(minimap_pos, 3.0, color);
            }

            // Draw camera viewport
            if let Ok(camera_transform) = camera_query.get_single() {
                let cam_pos = camera_transform.translation.truncate();
                let viewport_size = Vec2::new(400.0, 300.0); // Approximate viewport size

                let min_pos = world_to_minimap(cam_pos - viewport_size / 2.0, rect, WORLD_SIZE);
                let max_pos = world_to_minimap(cam_pos + viewport_size / 2.0, rect, WORLD_SIZE);

                painter.rect_stroke(
                    egui::Rect::from_min_max(
                        egui::Pos2::new(min_pos.x, min_pos.y),
                        egui::Pos2::new(max_pos.x, max_pos.y),
                    ),
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );
            }

            if response.clicked() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let world_pos = minimap_to_world(pointer, rect, WORLD_SIZE);
                    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                        camera_transform.translation.x = world_pos.x;
                        camera_transform.translation.y = world_pos.y;
                    }

                    let unit_count = selected_units.iter().count();
                    if unit_count > 0 {
                        let shift_held = keyboard.pressed(KeyCode::ShiftLeft)
                            || keyboard.pressed(KeyCode::ShiftRight);

                        for (index, (entity, core_id)) in selected_units.iter().enumerate() {
                            let offset = if unit_count > 1 {
                                calculate_formation_offset(index, unit_count)
                            } else {
                                Vec2::ZERO
                            };

                            let target = Vec2Fixed::new(
                                Fixed::from_num(world_pos.x + offset.x),
                                Fixed::from_num(world_pos.y + offset.y),
                            );

                            let command = match *input_mode {
                                InputMode::Normal => CoreCommand::MoveTo(target),
                                InputMode::AttackMove => CoreCommand::AttackMove(target),
                                InputMode::Patrol => CoreCommand::Patrol(target),
                            };

                            if shift_held {
                                core_commands.queue(core_id.0, command);
                            } else {
                                core_commands.set(core_id.0, command);
                                commands.entity(entity).remove::<AttackTarget>();
                            }
                        }

                        feedback_events.send(CommandFeedbackEvent {
                            position: world_pos,
                        });
                    }
                }
            }
        });
}

fn world_to_minimap(world_pos: Vec2, rect: egui::Rect, world_size: f32) -> egui::Pos2 {
    let minimap_x = rect.min.x + ((world_pos.x + world_size / 2.0) / world_size) * rect.width();
    let minimap_y = rect.max.y - ((world_pos.y + world_size / 2.0) / world_size) * rect.height();
    egui::Pos2::new(minimap_x, minimap_y)
}

fn minimap_to_world(minimap_pos: egui::Pos2, rect: egui::Rect, world_size: f32) -> Vec2 {
    let normalized_x = (minimap_pos.x - rect.min.x) / rect.width();
    let normalized_y = (rect.max.y - minimap_pos.y) / rect.height();
    Vec2::new(
        normalized_x * world_size - world_size / 2.0,
        normalized_y * world_size - world_size / 2.0,
    )
}

/// Renders the selection panel showing selected unit info.
fn ui_selection_panel(
    mut contexts: EguiContexts,
    selected: Query<(&GameHealth, &GameFaction), With<Selected>>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let selected_units: Vec<_> = selected.iter().collect();

    if selected_units.is_empty() {
        return;
    }

    egui::Window::new("Selection")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -10.0])
        .fixed_size([300.0, 100.0])
        .show(ctx, |ui| {
            if selected_units.len() == 1 {
                // Single unit selected - show details
                let (health, faction) = selected_units[0];

                ui.horizontal(|ui| {
                    // Unit portrait placeholder
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(64.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, 4.0, faction_to_egui_color(faction.faction));

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(faction_name(faction.faction))
                                .size(16.0)
                                .strong(),
                        );
                        ui.label("Unit");

                        // Health bar
                        let health_ratio = health.ratio();
                        let bar_width = 150.0;
                        let (bar_rect, _) = ui.allocate_exact_size(
                            egui::Vec2::new(bar_width, 12.0),
                            egui::Sense::hover(),
                        );

                        // Background
                        ui.painter().rect_filled(
                            bar_rect,
                            2.0,
                            egui::Color32::from_rgb(60, 20, 20),
                        );

                        // Health portion
                        let health_color = if health_ratio > 0.5 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else if health_ratio > 0.25 {
                            egui::Color32::from_rgb(200, 200, 50)
                        } else {
                            egui::Color32::from_rgb(200, 50, 50)
                        };

                        let health_rect = egui::Rect::from_min_size(
                            bar_rect.min,
                            egui::Vec2::new(bar_width * health_ratio, 12.0),
                        );
                        ui.painter().rect_filled(health_rect, 2.0, health_color);

                        ui.label(format!("{} / {}", health.current, health.max));
                    });
                });
            } else {
                // Multiple units selected - show count per faction
                ui.label(
                    egui::RichText::new(format!("{} units selected", selected_units.len()))
                        .size(16.0)
                        .strong(),
                );

                ui.horizontal(|ui| {
                    // Group by faction
                    let mut faction_counts: std::collections::HashMap<FactionId, usize> =
                        std::collections::HashMap::new();
                    for (_, faction) in &selected_units {
                        *faction_counts.entry(faction.faction).or_insert(0) += 1;
                    }

                    for (faction, count) in faction_counts {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(32.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(rect, 4.0, faction_to_egui_color(faction));
                        ui.label(format!("×{}", count));
                    }
                });
            }
        });
}

/// Renders the command panel with action buttons.
fn ui_command_panel(
    mut contexts: EguiContexts,
    selected: Query<Entity, With<Selected>>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    core_ids: Query<&CoreEntityId>,
    mut depot_production: Query<
        (&GameFaction, &mut GameProductionQueue),
        (With<GameDepot>, Without<Building>),
    >,
    mut building_production: Query<
        (&GameFaction, &Building, &mut GameProductionQueue),
        Without<GameDepot>,
    >,
    mut resources: ResMut<PlayerResources>,
    player_faction: Res<PlayerFaction>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };
    let selected_count = selected.iter().count();

    if selected_count == 0 {
        return;
    }

    // Check if we have a selected depot or barracks (for production UI)
    let mut selected_depot: Option<Entity> = None;
    let mut selected_barracks: Option<Entity> = None;
    for entity in selected.iter() {
        if depot_production.get(entity).is_ok() {
            selected_depot = Some(entity);
        } else if let Ok((_, building, _)) = building_production.get(entity) {
            if building.building_type == BuildingType::Barracks {
                selected_barracks = Some(entity);
            }
        }
    }

    egui::Window::new("Commands")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .fixed_size([280.0, 220.0])
        .show(ctx, |ui| {
            // If depot selected, show harvester production only
            if let Some(depot_entity) = selected_depot {
                if let Ok((faction, mut production)) = depot_production.get_mut(depot_entity) {
                    if faction.faction == player_faction.faction {
                        ui.label(egui::RichText::new("Command Center").strong());
                        ui.separator();

                        ui.horizontal(|ui| {
                            // Harvester button (only unit depot produces)
                            let harv_cost = UnitType::Harvester.cost();
                            let harv_supply = UnitType::Harvester.supply();
                            let can_afford_harv = resources.feedstock >= harv_cost
                                && resources.supply_used + harv_supply <= resources.supply_cap
                                && production.can_queue();
                            ui.add_enabled_ui(can_afford_harv, |ui| {
                                if ui
                                    .button(format!(
                                        "🔧 Harvester\n{} ⚡{}",
                                        harv_cost, harv_supply
                                    ))
                                    .clicked()
                                {
                                    resources.feedstock -= harv_cost;
                                    resources.supply_used += harv_supply;
                                    production.enqueue(UnitType::Harvester);
                                }
                            });
                        });

                        render_production_queue(ui, &mut production, &mut resources);
                        ui.separator();
                    }
                }
            }

            // If barracks selected, show infantry/ranger production
            if let Some(barracks_entity) = selected_barracks {
                if let Ok((faction, _, mut production)) =
                    building_production.get_mut(barracks_entity)
                {
                    if faction.faction == player_faction.faction {
                        ui.label(egui::RichText::new("Barracks").strong());
                        ui.separator();

                        ui.horizontal(|ui| {
                            // Infantry button
                            let inf_cost = UnitType::Infantry.cost();
                            let inf_supply = UnitType::Infantry.supply();
                            let can_afford_inf = resources.feedstock >= inf_cost
                                && resources.supply_used + inf_supply <= resources.supply_cap
                                && production.can_queue();
                            ui.add_enabled_ui(can_afford_inf, |ui| {
                                if ui
                                    .button(format!("🗡 Infantry\n{} ⚡{}", inf_cost, inf_supply))
                                    .clicked()
                                {
                                    resources.feedstock -= inf_cost;
                                    resources.supply_used += inf_supply;
                                    production.enqueue(UnitType::Infantry);
                                }
                            });

                            // Ranger button
                            let rang_cost = UnitType::Ranger.cost();
                            let rang_supply = UnitType::Ranger.supply();
                            let can_afford_rang = resources.feedstock >= rang_cost
                                && resources.supply_used + rang_supply <= resources.supply_cap
                                && production.can_queue();
                            ui.add_enabled_ui(can_afford_rang, |ui| {
                                if ui
                                    .button(format!("🏹 Ranger\n{} ⚡{}", rang_cost, rang_supply))
                                    .clicked()
                                {
                                    resources.feedstock -= rang_cost;
                                    resources.supply_used += rang_supply;
                                    production.enqueue(UnitType::Ranger);
                                }
                            });
                        });

                        render_production_queue(ui, &mut production, &mut resources);
                        ui.separator();
                    }
                }
            }

            // Standard unit commands
            ui.horizontal(|ui| {
                // Stop button
                if ui
                    .button(egui::RichText::new("⏹ Stop").size(14.0))
                    .clicked()
                {
                    for entity in selected.iter() {
                        if let Ok(core_id) = core_ids.get(entity) {
                            core_commands.set(core_id.0, CoreCommand::Stop);
                        }
                    }
                }

                // Hold Position button
                if ui
                    .button(egui::RichText::new("🛡 Hold").size(14.0))
                    .clicked()
                {
                    for entity in selected.iter() {
                        if let Ok(core_id) = core_ids.get(entity) {
                            core_commands.set(core_id.0, CoreCommand::HoldPosition);
                        }
                    }
                }
            });

            ui.label(egui::RichText::new("Right-click to move").weak().size(11.0));
        });
}

/// Helper to render production queue UI.
fn render_production_queue(
    ui: &mut egui::Ui,
    production: &mut GameProductionQueue,
    resources: &mut PlayerResources,
) {
    if !production.queue.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Production Queue").size(12.0));

        ui.horizontal(|ui| {
            for (i, queued) in production.queue.iter().enumerate() {
                let icon = match queued.unit_type {
                    UnitType::Infantry => "🗡",
                    UnitType::Harvester => "🔧",
                    UnitType::Ranger => "🏹",
                };

                if i == 0 {
                    let progress_pct = (queued.progress * 100.0).round() as i32;
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(icon).size(20.0));
                            ui.add(egui::ProgressBar::new(queued.progress).desired_width(30.0));
                            ui.label(
                                egui::RichText::new(format!("{}%", progress_pct))
                                    .size(10.0)
                                    .weak(),
                            );
                        });
                    });
                } else {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(icon).size(20.0));
                            ui.label(egui::RichText::new(format!("#{}", i)).size(10.0).weak());
                        });
                    });
                }
            }
        });

        if ui.small_button("❌ Cancel Last").clicked() {
            if let Some((cancelled, refund_rate)) = production.cancel_last() {
                let refund = (cancelled.cost() as f32 * refund_rate * 0.75) as i32;
                resources.feedstock += refund;
                // Also refund supply
                resources.supply_used -= cancelled.supply();
            }
        }
    }
}

/// Renders the build menu for placing new buildings.
fn ui_build_menu(
    mut contexts: EguiContexts,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    resources: Res<PlayerResources>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    // Toggle build menu with B key
    if keyboard.just_pressed(KeyCode::KeyB) && placement.placing.is_none() {
        // Show build menu
    }

    // Cancel placement with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        placement.placing = None;
    }

    // Show build menu window
    egui::Window::new("Build")
        .anchor(egui::Align2::LEFT_CENTER, [10.0, 0.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Buildings [B]").strong());
            ui.separator();

            // Building buttons
            for building_type in [
                BuildingType::SupplyDepot,
                BuildingType::Barracks,
                BuildingType::TechLab,
                BuildingType::Turret,
            ] {
                let cost = building_type.cost();
                let can_afford = resources.feedstock >= cost;
                let is_placing = placement.placing == Some(building_type);

                let icon = match building_type {
                    BuildingType::Depot => "🏠",
                    BuildingType::SupplyDepot => "📦",
                    BuildingType::Barracks => "⚔️",
                    BuildingType::TechLab => "🔬",
                    BuildingType::Turret => "🗼",
                };

                let label = format!("{} {} ({})", icon, building_type.name(), cost);

                ui.add_enabled_ui(can_afford, |ui| {
                    let button = ui.selectable_label(is_placing, label);
                    if button.clicked() {
                        if is_placing {
                            placement.placing = None;
                        } else {
                            placement.placing = Some(building_type);
                        }
                    }
                });
            }

            // Show placement status
            if let Some(bt) = placement.placing {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!(
                        "Placing: {} - Click to place, ESC to cancel",
                        bt.name()
                    ))
                    .weak()
                    .size(11.0),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_world_round_trip() {
        let rect =
            egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(200.0, 200.0));
        let world_size = 2000.0;
        let world_pos = Vec2::new(150.0, -250.0);

        let minimap_pos = world_to_minimap(world_pos, rect, world_size);
        let back = minimap_to_world(minimap_pos, rect, world_size);

        let delta = (world_pos - back).length();
        assert!(delta < 0.01);
    }

    #[test]
    fn apply_ui_settings_updates_scale_and_contrast() {
        let ctx = egui::Context::default();
        let settings = UiSettings {
            ui_scale: 1.25,
            high_contrast: true,
            ..UiSettings::default()
        };

        apply_ui_settings(&ctx, settings);

        assert!(ctx.style().visuals.override_text_color.is_some());
    }
}
