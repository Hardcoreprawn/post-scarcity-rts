//! Headless game runner implementation.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use bevy::prelude::*;
use rts_core::components::Command as CoreCommand;
use rts_core::factions::FactionId;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::protocol::{
    Command, EntityState, EntityType, GameResult, GameStatus, HealthState, MatchStatsOutput,
    ResourceState, Response,
};
use crate::scenario::Scenario;

/// Entity ID mapping from internal to external IDs.
#[derive(Resource, Default)]
struct EntityIdMap {
    next_id: u32,
    bevy_to_external: HashMap<Entity, u32>,
    external_to_bevy: HashMap<u32, Entity>,
}

impl EntityIdMap {
    fn register(&mut self, entity: Entity) -> u32 {
        if let Some(&id) = self.bevy_to_external.get(&entity) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.bevy_to_external.insert(entity, id);
        self.external_to_bevy.insert(id, entity);
        id
    }

    fn lookup(&self, external_id: u32) -> Option<Entity> {
        self.external_to_bevy.get(&external_id).copied()
    }

    fn lookup_external(&self, entity: Entity) -> Option<u32> {
        self.bevy_to_external.get(&entity).copied()
    }

    fn remove(&mut self, entity: Entity) {
        if let Some(id) = self.bevy_to_external.remove(&entity) {
            self.external_to_bevy.remove(&id);
        }
    }
}

/// Command queue for processing input.
#[derive(Resource, Default)]
struct CommandQueue {
    commands: Vec<Command>,
    should_quit: bool,
}

/// Response queue for output.
#[derive(Resource, Default)]
struct ResponseQueue {
    responses: Vec<Response>,
}

impl ResponseQueue {
    fn send(&mut self, response: Response) {
        self.responses.push(response);
    }
}

/// Headless runner configuration.
#[derive(Resource, Clone)]
pub struct HeadlessConfig {
    /// Output state after every tick (vs only on query).
    pub auto_state_output: bool,
    /// Scenario file to load on startup.
    pub scenario_path: Option<String>,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            auto_state_output: false,
            scenario_path: None,
        }
    }
}

/// Headless runner for AI-controlled gameplay.
pub struct HeadlessRunner {
    config: HeadlessConfig,
}

impl HeadlessRunner {
    /// Create a new headless runner with default config.
    pub fn new() -> Self {
        Self {
            config: HeadlessConfig::default(),
        }
    }

    /// Create a runner with custom configuration.
    pub fn with_config(config: HeadlessConfig) -> Self {
        Self { config }
    }

    /// Run the headless game loop.
    ///
    /// Reads JSON commands from stdin, outputs responses to stdout.
    pub fn run(self) {
        // Build app with headless plugins
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_plugins(rts_game::plugins::HeadlessGamePlugins)
            .insert_resource(self.config.clone())
            .init_resource::<EntityIdMap>()
            .init_resource::<CommandQueue>()
            .init_resource::<ResponseQueue>()
            .add_systems(First, read_stdin_commands)
            .add_systems(Last, (process_commands, flush_responses).chain());

        // Output ready message
        let ready = Response::ready(0);
        print!("{}", ready.to_json_line());
        io::stdout().flush().ok();

        // Run the app (blocks until quit)
        app.run();
    }
}

impl Default for HeadlessRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// System to read commands from stdin (non-blocking).
fn read_stdin_commands(mut queue: ResMut<CommandQueue>) {
    // Try to read a line from stdin (non-blocking)
    // In a real implementation we'd use async/threading,
    // but for now we'll do a simple non-blocking check
    let _stdin = io::stdin();

    // Check if there's input available (platform-specific)
    // For simplicity, we'll do a blocking read with a small timeout
    // TODO: Make this properly async

    if let Some(line) = try_read_line() {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        match Command::from_json(line) {
            Ok(cmd) => {
                if matches!(cmd, Command::Quit) {
                    queue.should_quit = true;
                }
                queue.commands.push(cmd);
            }
            Err(e) => {
                let error = Response::error(format!("Parse error: {}", e), None);
                print!("{}", error.to_json_line());
                io::stdout().flush().ok();
            }
        }
    }
}

/// Try to read a line without blocking forever.
fn try_read_line() -> Option<String> {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    // Use a channel with timeout for non-blocking read
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_ok() {
            let _ = sender.send(line);
        }
    });

    // Wait up to 10ms for input
    receiver.recv_timeout(Duration::from_millis(10)).ok()
}

/// System to process queued commands.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn process_commands(
    mut queue: ResMut<CommandQueue>,
    mut responses: ResMut<ResponseQueue>,
    mut entity_map: ResMut<EntityIdMap>,
    mut bevy_commands: Commands,
    mut app_exit: EventWriter<AppExit>,
    core_sim: Option<ResMut<rts_game::simulation::CoreSimulation>>,
    mut core_commands: Option<ResMut<rts_game::simulation::CoreCommandBuffer>>,
    mut player_resources: Option<ResMut<rts_game::economy::PlayerResources>>,
    units: Query<(
        Entity,
        &rts_game::components::GamePosition,
        &rts_game::components::GameFaction,
        Option<&rts_game::components::GameHealth>,
        Option<&rts_game::components::GameHarvester>,
        Option<&rts_game::components::AttackTarget>,
        Option<&rts_game::components::GameUnitKind>,
        Option<&rts_game::components::Building>,
        Option<&rts_game::components::CoreEntityId>,
    )>,
    resources: Option<Res<rts_game::economy::PlayerResources>>,
    game_state: Option<Res<rts_game::victory::GameState>>,
) {
    // Process all queued commands
    for cmd in queue.commands.drain(..) {
        let cmd_name = cmd.name();

        match cmd {
            Command::Tick { count: _ } => {
                // Advance simulation - for headless, we just acknowledge
                // The actual ticking happens in SimulationPlugin automatically
                // But we need to report state after
                responses.send(Response::ack(cmd_name));

                // Output state after ticks
                if let (Some(core), Some(res)) = (core_sim.as_ref(), resources.as_ref()) {
                    let state = build_state_response(
                        core.sim.get_tick(),
                        &units,
                        &entity_map,
                        res.feedstock as u32,
                        game_state.as_ref(),
                        core.sim.state_hash(),
                    );
                    responses.send(state);
                }
            }

            Command::Query => {
                if let (Some(core), Some(res)) = (core_sim.as_ref(), resources.as_ref()) {
                    let state = build_state_response(
                        core.sim.get_tick(),
                        &units,
                        &entity_map,
                        res.feedstock as u32,
                        game_state.as_ref(),
                        core.sim.state_hash(),
                    );
                    responses.send(state);
                } else {
                    responses.send(Response::error(
                        "Simulation not initialized",
                        Some(cmd_name),
                    ));
                }
            }

            Command::Hash => {
                if let Some(core) = core_sim.as_ref() {
                    responses.send(Response::StateHash {
                        tick: core.sim.get_tick(),
                        hash: core.sim.state_hash(),
                    });
                } else {
                    responses.send(Response::error(
                        "Simulation not initialized",
                        Some(cmd_name),
                    ));
                }
            }

            Command::Spawn {
                unit_type,
                x,
                y,
                faction,
            } => {
                let faction_id = faction.map_or(FactionId::Continuity, |f| match f {
                    0 => FactionId::Continuity,
                    1 => FactionId::Collegium,
                    _ => FactionId::Tinkers,
                });

                let pos = Vec2::new(x as f32, y as f32);
                let entity = bevy_commands
                    .spawn(rts_game::bundles::UnitBundle::new(pos, faction_id, 100))
                    .id();

                let external_id = entity_map.register(entity);
                responses.send(Response::Spawned {
                    entity_id: external_id,
                    unit_type,
                });
            }

            Command::Move {
                entity_id,
                target_x,
                target_y,
            } => {
                if let Some(entity) = entity_map.lookup(entity_id) {
                    // Find the CoreEntityId for this entity
                    if let Ok((_, _, _, _, _, _, _, _, Some(core_id))) = units.get(entity) {
                        if let Some(ref mut cmds) = core_commands {
                            let target = Vec2Fixed::new(
                                Fixed::from_num(target_x),
                                Fixed::from_num(target_y),
                            );
                            cmds.set(core_id.0, CoreCommand::MoveTo(target));
                            responses.send(Response::ack(cmd_name));
                        } else {
                            responses.send(Response::error(
                                "Core simulation not available",
                                Some(cmd_name),
                            ));
                        }
                    } else {
                        responses.send(Response::error(
                            format!("Entity {} not registered with core", entity_id),
                            Some(cmd_name),
                        ));
                    }
                } else {
                    responses.send(Response::error(
                        format!("Entity {} not found", entity_id),
                        Some(cmd_name),
                    ));
                }
            }

            Command::Attack {
                entity_id,
                target_id,
            } => {
                if let (Some(entity), Some(target_entity)) =
                    (entity_map.lookup(entity_id), entity_map.lookup(target_id))
                {
                    // Get core IDs for both entities
                    let attacker_core = units.get(entity).ok().and_then(|q| q.8.map(|c| c.0));
                    let target_core = units.get(target_entity).ok().and_then(|q| q.8.map(|c| c.0));

                    if let (Some(attacker_id), Some(target_id)) = (attacker_core, target_core) {
                        if let Some(ref mut cmds) = core_commands {
                            cmds.set(attacker_id, CoreCommand::Attack(target_id));
                            responses.send(Response::ack(cmd_name));
                        } else {
                            responses.send(Response::error(
                                "Core simulation not available",
                                Some(cmd_name),
                            ));
                        }
                    } else {
                        responses.send(Response::error(
                            "Entities not registered with core",
                            Some(cmd_name),
                        ));
                    }
                } else if entity_map.lookup(entity_id).is_none() {
                    responses.send(Response::error(
                        format!("Entity {} not found", entity_id),
                        Some(cmd_name),
                    ));
                } else {
                    responses.send(Response::error(
                        format!("Target {} not found", target_id),
                        Some(cmd_name),
                    ));
                }
            }

            Command::Stop { entity_id } => {
                if let Some(entity) = entity_map.lookup(entity_id) {
                    if let Ok((_, _, _, _, _, _, _, _, Some(core_id))) = units.get(entity) {
                        if let Some(ref mut cmds) = core_commands {
                            cmds.set(core_id.0, CoreCommand::Stop);
                            responses.send(Response::ack(cmd_name));
                        } else {
                            responses.send(Response::error(
                                "Core simulation not available",
                                Some(cmd_name),
                            ));
                        }
                    } else {
                        responses.send(Response::error(
                            format!("Entity {} not registered with core", entity_id),
                            Some(cmd_name),
                        ));
                    }
                } else {
                    responses.send(Response::error(
                        format!("Entity {} not found", entity_id),
                        Some(cmd_name),
                    ));
                }
            }

            Command::Teleport { entity_id, x, y } => {
                if let Some(entity) = entity_map.lookup(entity_id) {
                    // Update the GamePosition component directly
                    let target = Vec2Fixed::new(Fixed::from_num(x), Fixed::from_num(y));
                    bevy_commands
                        .entity(entity)
                        .insert(rts_game::components::GamePosition::new(target));
                    responses.send(Response::ack(cmd_name));
                } else {
                    responses.send(Response::error(
                        format!("Entity {} not found", entity_id),
                        Some(cmd_name),
                    ));
                }
            }

            Command::Kill { entity_id } => {
                if let Some(entity) = entity_map.lookup(entity_id) {
                    bevy_commands.entity(entity).despawn_recursive();
                    entity_map.remove(entity);
                    responses.send(Response::ack(cmd_name));
                } else {
                    responses.send(Response::error(
                        format!("Entity {} not found", entity_id),
                        Some(cmd_name),
                    ));
                }
            }

            Command::SetResources { amount } => {
                if let Some(ref mut res) = player_resources {
                    res.feedstock = amount as i32;
                    responses.send(Response::ack(cmd_name));
                } else {
                    responses.send(Response::error(
                        "Player resources not available",
                        Some(cmd_name),
                    ));
                }
            }

            Command::Speed { multiplier: _ } => {
                // Speed control would need Time resource modification
                // For now, just acknowledge
                responses.send(Response::ack(cmd_name));
            }

            Command::SpawnBuilding {
                building_type,
                x,
                y,
                faction,
            } => {
                let faction_id = faction.map_or(FactionId::Continuity, |f| match f {
                    0 => FactionId::Continuity,
                    1 => FactionId::Collegium,
                    _ => FactionId::Tinkers,
                });

                let pos = Vec2::new(x as f32, y as f32);

                // Spawn appropriate building bundle based on type
                let entity = match building_type.as_str() {
                    "command_center" | "depot" => bevy_commands
                        .spawn(rts_game::bundles::DepotBundle::new(pos, faction_id))
                        .id(),
                    "barracks" => bevy_commands
                        .spawn(rts_game::bundles::BarracksBundle::new(pos, faction_id))
                        .id(),
                    "supply_depot" | "supply" => bevy_commands
                        .spawn(rts_game::bundles::SupplyDepotBundle::new(pos, faction_id))
                        .id(),
                    "tech_lab" | "techlab" => bevy_commands
                        .spawn(rts_game::bundles::TechLabBundle::new(pos, faction_id))
                        .id(),
                    "turret" | "defense" => bevy_commands
                        .spawn(rts_game::bundles::TurretBundle::new(pos, faction_id))
                        .id(),
                    _ => {
                        // Default to depot for unknown building types
                        bevy_commands
                            .spawn(rts_game::bundles::DepotBundle::new(pos, faction_id))
                            .id()
                    }
                };

                let external_id = entity_map.register(entity);
                responses.send(Response::Spawned {
                    entity_id: external_id,
                    unit_type: building_type,
                });
            }

            Command::Win | Command::Lose => {
                responses.send(Response::ack(cmd_name));
                // Trigger game over
                let result = if matches!(cmd, Command::Win) {
                    GameResult::Victory
                } else {
                    GameResult::Defeat
                };
                let tick = core_sim.as_ref().map_or(0, |c| c.sim.get_tick());
                responses.send(Response::GameOver {
                    result,
                    ticks: tick,
                    stats: MatchStatsOutput::default(),
                });
            }

            Command::Quit => {
                responses.send(Response::Bye);
                app_exit.send(AppExit::Success);
            }

            Command::LoadScenario { path } => {
                // Try to load the scenario
                match Scenario::load(&path) {
                    Ok(scenario) => {
                        tracing::info!("Loaded scenario: {}", scenario.name);
                        responses.send(Response::ack(cmd_name));
                        // Note: Actually applying the scenario would require more complex
                        // world manipulation. For now, we just acknowledge the load.
                    }
                    Err(e) => {
                        responses.send(Response::error(
                            format!("Failed to load scenario: {}", e),
                            Some(cmd_name),
                        ));
                    }
                }
            }

            Command::Screenshot { path: _ } => {
                responses.send(Response::error(
                    "Screenshots require graphical mode",
                    Some(cmd_name),
                ));
            }
        }
    }

    // Handle quit flag
    if queue.should_quit {
        app_exit.send(AppExit::Success);
    }
}

/// Build state response from current game state.
#[allow(clippy::type_complexity)]
fn build_state_response(
    tick: u64,
    units: &Query<(
        Entity,
        &rts_game::components::GamePosition,
        &rts_game::components::GameFaction,
        Option<&rts_game::components::GameHealth>,
        Option<&rts_game::components::GameHarvester>,
        Option<&rts_game::components::AttackTarget>,
        Option<&rts_game::components::GameUnitKind>,
        Option<&rts_game::components::Building>,
        Option<&rts_game::components::CoreEntityId>,
    )>,
    entity_map: &EntityIdMap,
    feedstock: u32,
    game_state: Option<&Res<rts_game::victory::GameState>>,
    hash: u64,
) -> Response {
    let mut entities = Vec::new();

    for (entity, pos, faction, health, harvester, attack_target, unit_kind, building, _core_id) in
        units.iter()
    {
        let external_id = entity_map.lookup_external(entity).unwrap_or(0);

        let entity_type = if building.is_some() {
            EntityType::Building {
                kind: "unknown".to_string(),
            }
        } else {
            EntityType::Unit {
                kind: unit_kind.map_or("unknown".to_string(), |_| "unit".to_string()),
            }
        };

        let health_state = health.map(|h| HealthState {
            current: h.current,
            max: h.max,
        });

        let cargo = harvester.map(|h| h.current_load as u32);
        let target = attack_target.and_then(|at| entity_map.lookup_external(at.target));

        entities.push(EntityState {
            id: external_id,
            entity_type,
            x: pos.value.x.to_num::<f64>(),
            y: pos.value.y.to_num::<f64>(),
            faction: faction.faction as u8,
            health: health_state,
            cargo,
            target,
            state: None,
        });
    }

    let game_status = match game_state {
        Some(gs) => match **gs {
            rts_game::victory::GameState::Playing => GameStatus::InProgress,
            rts_game::victory::GameState::Victory => GameStatus::Victory,
            rts_game::victory::GameState::Defeat => GameStatus::Defeat,
        },
        None => GameStatus::InProgress,
    };

    Response::State {
        tick,
        entities,
        resources: ResourceState { feedstock },
        game_status,
        hash,
    }
}

/// System to flush response queue to stdout.
fn flush_responses(mut responses: ResMut<ResponseQueue>) {
    for response in responses.responses.drain(..) {
        print!("{}", response.to_json_line());
    }
    io::stdout().flush().ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_map() {
        let map = EntityIdMap::default();

        // Can't easily create Entity in unit test, but we can test the structure
        assert_eq!(map.next_id, 0);
    }
}
