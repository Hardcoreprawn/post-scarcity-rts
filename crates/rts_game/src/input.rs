//! Input plugin for game input handling.
//!
//! Provides move commands, attack-move, stop command, and harvester targeting.

use bevy::prelude::*;
use rts_core::components::Command as CoreCommand;
use rts_core::math::{Fixed, Vec2Fixed};

use crate::camera::MainCamera;
use crate::components::{
    AttackTarget, CombatStats, CoreEntityId, GameCommandQueue, GameFaction, GameHarvester,
    GameHarvesterState, GamePosition, GameResourceNode, MovementTarget, Selected,
};
use crate::render::CommandFeedbackEvent;
use crate::simulation::{ClientCommandSet, CoreCommandBuffer, UNIT_RADIUS};

/// Plugin for game input handling.
///
/// Provides:
/// - Right-click to issue move commands
/// - Attack-move with A + right-click
/// - Stop command with S key
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMode>()
            .init_resource::<KeyBindings>()
            .add_systems(Update, update_input_mode.before(ClientCommandSet::Gather))
            .add_systems(Update, handle_move_command.in_set(ClientCommandSet::Gather))
            .add_systems(Update, handle_stop_command.in_set(ClientCommandSet::Gather))
            .add_systems(Update, handle_hold_command.in_set(ClientCommandSet::Gather));
    }
}

/// Key bindings for core RTS commands.
#[derive(Resource, Debug, Clone, Copy)]
pub struct KeyBindings {
    /// Hold for attack-move mode.
    pub attack_move: KeyCode,
    /// Hold for patrol mode.
    pub patrol: KeyCode,
    /// Stop command.
    pub stop: KeyCode,
    /// Hold position command.
    pub hold_position: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            attack_move: KeyCode::KeyA,
            patrol: KeyCode::KeyP,
            stop: KeyCode::KeyS,
            hold_position: KeyCode::KeyH,
        }
    }
}

/// Current input mode for command issuing.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    /// Normal mode - right-click moves.
    #[default]
    Normal,
    /// Attack mode - right-click attack-moves.
    AttackMove,
    /// Patrol mode - right-click patrols.
    Patrol,
}

/// Updates the input mode based on key presses.
fn update_input_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut input_mode: ResMut<InputMode>,
) {
    // Hold patrol or attack-move binding
    if keyboard.pressed(bindings.patrol) {
        *input_mode = InputMode::Patrol;
    } else if keyboard.pressed(bindings.attack_move) {
        *input_mode = InputMode::AttackMove;
    } else {
        *input_mode = InputMode::Normal;
    }
}

/// Handles right-click to issue move/attack-move commands.
/// Also handles right-clicking on resource nodes to direct harvesters,
/// and right-clicking on enemies to attack them.
fn handle_move_command(
    commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    input_mode: Res<InputMode>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    selected_units: Query<
        (
            Entity,
            &CoreEntityId,
            &GameFaction,
            Option<&mut GameHarvester>,
            Option<&CombatStats>,
        ),
        (With<Selected>, With<GameCommandQueue>),
    >,
    nodes: Query<(Entity, &GamePosition, Option<&Sprite>), With<GameResourceNode>>,
    potential_targets: Query<
        (
            Entity,
            &CoreEntityId,
            &GamePosition,
            &GameFaction,
            Option<&Sprite>,
        ),
        (With<CombatStats>, Without<Selected>),
    >,
    feedback_events: EventWriter<CommandFeedbackEvent>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    issue_move_commands_at(
        commands,
        *input_mode,
        shift_held,
        world_position,
        &mut core_commands,
        selected_units,
        nodes,
        potential_targets,
        feedback_events,
    );
}

fn issue_move_commands_at(
    mut commands: Commands,
    input_mode: InputMode,
    shift_held: bool,
    world_position: Vec2,
    core_commands: &mut CoreCommandBuffer,
    mut selected_units: Query<
        (
            Entity,
            &CoreEntityId,
            &GameFaction,
            Option<&mut GameHarvester>,
            Option<&CombatStats>,
        ),
        (With<Selected>, With<GameCommandQueue>),
    >,
    nodes: Query<(Entity, &GamePosition, Option<&Sprite>), With<GameResourceNode>>,
    potential_targets: Query<
        (
            Entity,
            &CoreEntityId,
            &GamePosition,
            &GameFaction,
            Option<&Sprite>,
        ),
        (With<CombatStats>, Without<Selected>),
    >,
    mut feedback_events: EventWriter<CommandFeedbackEvent>,
) {
    // Check if we clicked on a resource node
    const NODE_CLICK_RADIUS: f32 = 30.0;
    let clicked_node: Option<(Entity, Vec2Fixed)> = nodes
        .iter()
        .find(|(_, node_pos, sprite)| {
            let node_world = node_pos.as_vec2();
            node_world.distance(world_position) < click_radius(*sprite, NODE_CLICK_RADIUS)
        })
        .map(|(entity, pos, _)| (entity, pos.value));

    // Check if we clicked on an enemy unit
    const UNIT_CLICK_RADIUS: f32 = 25.0;
    let clicked_enemy = |my_faction: &GameFaction| -> Option<(Entity, CoreEntityId)> {
        potential_targets
            .iter()
            .filter(|(_, _, _, faction, _)| faction.faction != my_faction.faction)
            .find(|(_, _, pos, _, sprite)| {
                let unit_world = pos.as_vec2();
                unit_world.distance(world_position) < click_radius(*sprite, UNIT_CLICK_RADIUS)
            })
            .map(|(entity, core_id, _, _, _)| (entity, *core_id))
    };

    // Count selected units for formation spreading
    let unit_count = selected_units.iter().count();

    // Calculate formation offsets for units
    // Uses a spiral pattern to spread units around the clicked point
    let mut issued_command = false;
    for (index, (entity, core_id, my_faction, harvester_opt, combat_opt)) in
        selected_units.iter_mut().enumerate()
    {
        // If we clicked on a node and this is a harvester, send it to harvest
        if let (Some((node_entity, node_pos)), Some(mut harvester)) = (clicked_node, harvester_opt)
        {
            // Clear current commands and set harvester to target this node
            harvester.state = GameHarvesterState::MovingToNode(node_entity);
            harvester.assigned_node = Some(node_entity); // Remember this node
            commands
                .entity(entity)
                .insert(MovementTarget { target: node_pos });
            issued_command = true;
            continue;
        }

        // If we clicked on an enemy and this unit can attack, attack it
        if combat_opt.is_some() {
            if let Some((target_entity, target_core)) = clicked_enemy(my_faction) {
                if shift_held {
                    core_commands.queue(core_id.0, CoreCommand::Attack(target_core.0));
                } else {
                    commands
                        .entity(entity)
                        .insert(AttackTarget {
                            target: target_entity,
                        })
                        .remove::<MovementTarget>();
                    core_commands.set(core_id.0, CoreCommand::Attack(target_core.0));
                }
                issued_command = true;
                continue;
            }
        }

        let offset = if unit_count > 1 {
            calculate_formation_offset(index, unit_count)
        } else {
            Vec2::ZERO
        };

        let target = Vec2Fixed::new(
            Fixed::from_num(world_position.x + offset.x),
            Fixed::from_num(world_position.y + offset.y),
        );

        let command = match input_mode {
            InputMode::Normal => CoreCommand::MoveTo(target),
            InputMode::AttackMove => CoreCommand::AttackMove(target),
            InputMode::Patrol => CoreCommand::Patrol(target),
        };

        if shift_held {
            // Queue the command
            core_commands.queue(core_id.0, command);
        } else {
            // Replace existing commands
            core_commands.set(core_id.0, command);
            // Clear attack target when issuing move command
            commands.entity(entity).remove::<AttackTarget>();
        }
        issued_command = true;
    }

    if issued_command {
        feedback_events.send(CommandFeedbackEvent {
            position: world_position,
        });
    }
}

fn click_radius(sprite: Option<&Sprite>, fallback: f32) -> f32 {
    let size = sprite
        .and_then(|sprite| sprite.custom_size)
        .unwrap_or(Vec2::new(fallback * 2.0, fallback * 2.0));
    let radius = size.x.max(size.y) / 2.0;
    radius.max(fallback)
}

/// Calculates a formation offset for unit placement.
///
/// Uses a spiral/circular pattern to distribute units around a center point.
pub fn calculate_formation_offset(index: usize, total: usize) -> Vec2 {
    if total <= 1 {
        return Vec2::ZERO;
    }

    // Spacing between units in formation
    const FORMATION_SPACING: f32 = UNIT_RADIUS * 2.5;

    // For small groups, use a simple circle
    if total <= 7 {
        let angle = (index as f32 / total as f32) * std::f32::consts::TAU;
        let radius = FORMATION_SPACING;
        return Vec2::new(angle.cos() * radius, angle.sin() * radius);
    }

    // For larger groups, use concentric rings
    // Ring 0: center (1 unit)
    // Ring 1: 6 units
    // Ring 2: 12 units
    // Ring 3: 18 units, etc.

    let mut ring = 0;
    let mut ring_start = 0;
    let mut ring_capacity = 1;

    while ring_start + ring_capacity <= index {
        ring_start += ring_capacity;
        ring += 1;
        ring_capacity = if ring == 0 { 1 } else { ring * 6 };
    }

    if ring == 0 {
        return Vec2::ZERO;
    }

    let index_in_ring = index - ring_start;
    let angle = (index_in_ring as f32 / ring_capacity as f32) * std::f32::consts::TAU;
    let radius = ring as f32 * FORMATION_SPACING;

    Vec2::new(angle.cos() * radius, angle.sin() * radius)
}

/// Handles S key to issue stop commands.
fn handle_stop_command(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    selected_units: Query<&CoreEntityId, (With<Selected>, With<GameCommandQueue>)>,
) {
    if keyboard.just_pressed(bindings.stop) {
        for core_id in selected_units.iter() {
            core_commands.set(core_id.0, CoreCommand::Stop);
        }
    }
}

/// Handles H key to issue hold position commands.
fn handle_hold_command(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut core_commands: ResMut<CoreCommandBuffer>,
    selected_units: Query<&CoreEntityId, (With<Selected>, With<GameCommandQueue>)>,
) {
    if keyboard.just_pressed(bindings.hold_position) {
        for core_id in selected_units.iter() {
            core_commands.set(core_id.0, CoreCommand::HoldPosition);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::DamageType;
    use crate::simulation::{ClientCommandSet, CoreSimulation, SimulationPlugin};

    #[derive(Resource, Debug, Clone, Copy)]
    struct PendingCommand {
        position: Vec2,
        shift_held: bool,
    }

    fn issue_pending_command(
        commands: Commands,
        input_mode: Res<InputMode>,
        pending: Res<PendingCommand>,
        mut core_commands: ResMut<CoreCommandBuffer>,
        selected_units: Query<
            (
                Entity,
                &CoreEntityId,
                &GameFaction,
                Option<&mut GameHarvester>,
                Option<&CombatStats>,
            ),
            (With<Selected>, With<GameCommandQueue>),
        >,
        nodes: Query<(Entity, &GamePosition, Option<&Sprite>), With<GameResourceNode>>,
        potential_targets: Query<
            (
                Entity,
                &CoreEntityId,
                &GamePosition,
                &GameFaction,
                Option<&Sprite>,
            ),
            (With<CombatStats>, Without<Selected>),
        >,
        feedback_events: EventWriter<CommandFeedbackEvent>,
    ) {
        issue_move_commands_at(
            commands,
            *input_mode,
            pending.shift_held,
            pending.position,
            &mut core_commands,
            selected_units,
            nodes,
            potential_targets,
            feedback_events,
        );
    }

    fn setup_basic_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(SimulationPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(KeyBindings::default());
        app.add_event::<CommandFeedbackEvent>();
        app
    }

    #[test]
    fn stop_command_sets_queue() {
        let mut app = setup_basic_app();
        app.add_systems(Update, handle_stop_command.in_set(ClientCommandSet::Gather));
        let entity = app
            .world_mut()
            .spawn((Selected, GameCommandQueue::new(), GamePosition::ORIGIN))
            .id();

        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyS);

        app.update();

        let core_id = app.world().get::<CoreEntityId>(entity).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();
        assert_eq!(queue.current(), Some(&CoreCommand::Stop));
    }

    #[test]
    fn hold_command_sets_queue() {
        let mut app = setup_basic_app();
        app.add_systems(Update, handle_hold_command.in_set(ClientCommandSet::Gather));
        let entity = app
            .world_mut()
            .spawn((Selected, GameCommandQueue::new(), GamePosition::ORIGIN))
            .id();

        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyH);

        app.update();

        let core_id = app.world().get::<CoreEntityId>(entity).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();
        assert_eq!(queue.current(), Some(&CoreCommand::HoldPosition));
    }

    #[test]
    fn patrol_mode_issues_patrol_command() {
        let mut app = setup_basic_app();
        app.insert_resource(InputMode::Patrol);
        app.add_systems(
            Update,
            issue_pending_command.in_set(ClientCommandSet::Gather),
        );
        app.insert_resource(PendingCommand {
            position: Vec2::new(100.0, 80.0),
            shift_held: false,
        });

        let unit = app
            .world_mut()
            .spawn((
                Selected,
                GameCommandQueue::new(),
                GamePosition::ORIGIN,
                GameFaction {
                    faction: rts_core::factions::FactionId::Continuity,
                },
            ))
            .id();

        let expected = Vec2Fixed::new(Fixed::from_num(100.0), Fixed::from_num(80.0));

        app.update();

        let core_id = app.world().get::<CoreEntityId>(unit).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();
        match queue.current().cloned() {
            Some(CoreCommand::Patrol(target)) => {
                assert_eq!(target, expected);
            }
            other => panic!("Expected patrol command, got {other:?}"),
        }
    }

    #[test]
    fn attack_move_mode_issues_attack_move_command() {
        let mut app = setup_basic_app();
        app.insert_resource(InputMode::AttackMove);
        app.add_systems(
            Update,
            issue_pending_command.in_set(ClientCommandSet::Gather),
        );
        app.insert_resource(PendingCommand {
            position: Vec2::new(120.0, 60.0),
            shift_held: false,
        });

        let unit = app
            .world_mut()
            .spawn((
                Selected,
                GameCommandQueue::new(),
                GamePosition::ORIGIN,
                GameFaction {
                    faction: rts_core::factions::FactionId::Continuity,
                },
            ))
            .id();

        let expected = Vec2Fixed::new(Fixed::from_num(120.0), Fixed::from_num(60.0));

        app.update();

        let core_id = app.world().get::<CoreEntityId>(unit).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();
        match queue.current().cloned() {
            Some(CoreCommand::AttackMove(target)) => {
                assert_eq!(target, expected);
            }
            other => panic!("Expected attack-move command, got {other:?}"),
        }
    }

    #[test]
    fn shift_queue_appends_commands() {
        let mut app = setup_basic_app();
        app.insert_resource(InputMode::Normal);
        app.add_systems(
            Update,
            issue_pending_command.in_set(ClientCommandSet::Gather),
        );

        let unit = app
            .world_mut()
            .spawn((
                Selected,
                GameCommandQueue::new(),
                GamePosition::ORIGIN,
                GameFaction {
                    faction: rts_core::factions::FactionId::Continuity,
                },
            ))
            .id();

        app.insert_resource(PendingCommand {
            position: Vec2::new(50.0, 20.0),
            shift_held: false,
        });
        app.update();

        app.insert_resource(PendingCommand {
            position: Vec2::new(80.0, 40.0),
            shift_held: true,
        });
        app.update();

        let core_id = app.world().get::<CoreEntityId>(unit).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();

        let first = Vec2Fixed::new(Fixed::from_num(50.0), Fixed::from_num(20.0));
        let second = Vec2Fixed::new(Fixed::from_num(80.0), Fixed::from_num(40.0));
        let commands: Vec<_> = queue.commands.iter().cloned().collect();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], CoreCommand::MoveTo(first));
        assert_eq!(commands[1], CoreCommand::MoveTo(second));
    }

    #[test]
    fn custom_keybindings_drive_stop_command() {
        let mut app = setup_basic_app();
        app.insert_resource(KeyBindings {
            stop: KeyCode::KeyZ,
            ..KeyBindings::default()
        });
        app.add_systems(Update, handle_stop_command.in_set(ClientCommandSet::Gather));

        let entity = app
            .world_mut()
            .spawn((Selected, GameCommandQueue::new(), GamePosition::ORIGIN))
            .id();

        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyZ);

        app.update();

        let core_id = app.world().get::<CoreEntityId>(entity).unwrap().0;
        let sim = &app.world().resource::<CoreSimulation>().sim;
        let queue = sim
            .get_entity(core_id)
            .unwrap()
            .command_queue
            .as_ref()
            .unwrap();
        assert_eq!(queue.current(), Some(&CoreCommand::Stop));
    }

    #[test]
    fn click_radius_uses_sprite_size() {
        let small = Sprite {
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..Default::default()
        };
        let large = Sprite {
            custom_size: Some(Vec2::new(80.0, 40.0)),
            ..Default::default()
        };

        let small_radius = click_radius(Some(&small), 25.0);
        let large_radius = click_radius(Some(&large), 25.0);
        let fallback_radius = click_radius(None, 25.0);

        assert!(large_radius > small_radius);
        assert!(small_radius >= fallback_radius);
    }

    #[test]
    fn shift_attack_does_not_set_attack_target() {
        let mut app = setup_basic_app();
        app.insert_resource(InputMode::default());
        app.add_systems(
            Update,
            issue_pending_command.in_set(ClientCommandSet::Gather),
        );

        let enemy_position = Vec2Fixed::new(Fixed::from_num(50.0), Fixed::from_num(10.0));
        let _enemy = app
            .world_mut()
            .spawn((
                GamePosition::new(enemy_position),
                GameFaction {
                    faction: rts_core::factions::FactionId::Collegium,
                },
                CombatStats::new(10, DamageType::Kinetic, 60.0, 0.5),
            ))
            .id();

        let selected = app
            .world_mut()
            .spawn((
                Selected,
                GameCommandQueue::new(),
                GamePosition::ORIGIN,
                GameFaction {
                    faction: rts_core::factions::FactionId::Continuity,
                },
                CombatStats::new(8, DamageType::Kinetic, 60.0, 0.5),
            ))
            .id();

        app.insert_resource(PendingCommand {
            position: Vec2::new(50.0, 10.0),
            shift_held: true,
        });

        app.update();

        let selected_has_target = app.world().get::<AttackTarget>(selected).is_some();
        assert!(
            !selected_has_target,
            "Shift-queued attacks should not set AttackTarget."
        );
    }
}
