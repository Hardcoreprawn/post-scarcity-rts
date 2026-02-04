//! Game metrics collection for balance analysis.
//!
//! This module provides comprehensive metrics collection for analyzing
//! game balance across multiple matches.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Complete metrics for a single game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetrics {
    /// Unique game identifier.
    pub game_id: String,
    /// Scenario name.
    pub scenario: String,
    /// Random seed used.
    pub seed: u64,
    /// Total game duration in ticks.
    pub duration_ticks: u64,
    /// Winning faction (None = draw).
    pub winner: Option<String>,
    /// How the game ended.
    pub win_condition: String,
    /// Per-faction metrics.
    pub factions: HashMap<String, FactionMetrics>,
    /// Timed events log.
    pub events: Vec<TimedEvent>,
    /// Final simulation state hash (for determinism validation).
    pub final_state_hash: u64,
}

impl GameMetrics {
    /// Create a new game metrics instance.
    #[must_use]
    pub fn new(game_id: impl Into<String>, scenario: impl Into<String>, seed: u64) -> Self {
        Self {
            game_id: game_id.into(),
            scenario: scenario.into(),
            seed,
            ..Default::default()
        }
    }

    /// Get or create faction metrics.
    pub fn faction_mut(&mut self, faction_id: &str) -> &mut FactionMetrics {
        self.factions
            .entry(faction_id.to_string())
            .or_insert_with(|| FactionMetrics::new(faction_id))
    }

    /// Record a timed event.
    pub fn record_event(&mut self, tick: u64, event_type: EventType, faction: &str, details: &str) {
        self.events.push(TimedEvent {
            tick,
            event_type,
            faction: faction.to_string(),
            details: details.to_string(),
        });
    }

    /// Finalize the game with outcome.
    pub fn finalize(&mut self, duration: u64, winner: Option<String>, condition: &str) {
        self.duration_ticks = duration;
        self.winner = winner;
        self.win_condition = condition.to_string();
    }
}

/// Metrics for a single faction in a game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionMetrics {
    /// Faction identifier.
    pub faction_id: String,
    /// Final score.
    pub final_score: i64,

    // === Economy ===
    /// Total resources gathered.
    pub total_resources_gathered: i64,
    /// Total resources spent.
    pub total_resources_spent: i64,
    /// Peak income rate (per minute).
    pub peak_income_rate: f64,
    /// Resource efficiency (gathered / potential).
    pub resource_efficiency: f64,

    // === Military ===
    /// Units produced by type.
    pub units_produced: HashMap<String, u32>,
    /// Units lost by type.
    pub units_lost: HashMap<String, u32>,
    /// Enemy units killed by type.
    pub units_killed: HashMap<String, u32>,
    /// Buildings constructed by type.
    pub buildings_constructed: HashMap<String, u32>,
    /// Buildings destroyed (enemy).
    pub buildings_destroyed: HashMap<String, u32>,
    /// Buildings lost.
    pub buildings_lost: HashMap<String, u32>,

    // === Combat ===
    /// Total damage dealt.
    pub total_damage_dealt: i64,
    /// Total damage taken.
    pub total_damage_taken: i64,
    /// Battles won.
    pub battles_won: u32,
    /// Battles lost.
    pub battles_lost: u32,
    /// Kill/death ratio.
    pub kd_ratio: f64,

    // === Timing ===
    /// Tick of first attack on enemy.
    pub first_attack_tick: Option<u64>,
    /// Tick of first expansion.
    pub first_expansion_tick: Option<u64>,
    /// Tech unlock times (tech_name -> tick).
    pub tech_unlock_times: HashMap<String, u64>,
    /// Time to first military unit.
    pub first_combat_unit_tick: Option<u64>,

    // === Positioning ===
    /// Map control over time (tick, percentage).
    pub map_control_over_time: Vec<(u64, f64)>,
    /// Average army position over time (tick, x, y).
    pub average_army_position: Vec<(u64, f64, f64)>,
    /// Maximum units at once.
    pub peak_army_size: u32,
}

impl FactionMetrics {
    /// Create new faction metrics.
    #[must_use]
    pub fn new(faction_id: impl Into<String>) -> Self {
        Self {
            faction_id: faction_id.into(),
            ..Default::default()
        }
    }

    /// Record a unit production.
    pub fn record_unit_produced(&mut self, unit_type: &str) {
        *self
            .units_produced
            .entry(unit_type.to_string())
            .or_default() += 1;
    }

    /// Record a unit death.
    pub fn record_unit_lost(&mut self, unit_type: &str) {
        *self.units_lost.entry(unit_type.to_string()).or_default() += 1;
    }

    /// Record an enemy unit kill.
    pub fn record_unit_killed(&mut self, unit_type: &str) {
        *self.units_killed.entry(unit_type.to_string()).or_default() += 1;
    }

    /// Record a building construction.
    pub fn record_building_constructed(&mut self, building_type: &str) {
        *self
            .buildings_constructed
            .entry(building_type.to_string())
            .or_default() += 1;
    }

    /// Record resources gathered.
    pub fn record_resources_gathered(&mut self, amount: i64) {
        self.total_resources_gathered += amount;
    }

    /// Record resources spent.
    pub fn record_resources_spent(&mut self, amount: i64) {
        self.total_resources_spent += amount;
    }

    /// Record damage dealt.
    pub fn record_damage_dealt(&mut self, amount: i64) {
        self.total_damage_dealt += amount;
    }

    /// Record damage taken.
    pub fn record_damage_taken(&mut self, amount: i64) {
        self.total_damage_taken += amount;
    }

    /// Calculate final stats.
    pub fn calculate_derived_stats(&mut self) {
        let total_killed: u32 = self.units_killed.values().sum();
        let total_lost: u32 = self.units_lost.values().sum();
        self.kd_ratio = if total_lost > 0 {
            total_killed as f64 / total_lost as f64
        } else if total_killed > 0 {
            f64::INFINITY
        } else {
            1.0
        };
    }

    /// Record map control snapshot.
    pub fn record_map_control(&mut self, tick: u64, control_percentage: f64) {
        self.map_control_over_time.push((tick, control_percentage));
    }

    /// Record army position snapshot.
    pub fn record_army_position(&mut self, tick: u64, x: f64, y: f64) {
        self.average_army_position.push((tick, x, y));
    }
}

/// A timed event during the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    /// Tick when the event occurred.
    pub tick: u64,
    /// Type of event.
    pub event_type: EventType,
    /// Faction involved.
    pub faction: String,
    /// Event details.
    pub details: String,
}

/// Types of events that can be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Unit was produced.
    UnitProduced,
    /// Unit was killed.
    UnitKilled,
    /// Building completed construction.
    BuildingCompleted,
    /// Building was destroyed.
    BuildingDestroyed,
    /// Battle started (multiple units engaged).
    BattleStarted,
    /// Battle ended.
    BattleEnded,
    /// Expansion (new base) started.
    ExpansionStarted,
    /// Tech was unlocked.
    TechUnlocked,
    /// Major engagement (>5 units involved).
    MajorEngagement,
    /// Resources depleted (node exhausted).
    ResourcesDepleted,
    /// First attack on enemy.
    FirstAttack,
}

/// Summary statistics across multiple games.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total games played.
    pub total_games: u32,
    /// Games won by each faction.
    pub wins_by_faction: HashMap<String, u32>,
    /// Win rates by faction.
    pub win_rates: HashMap<String, f64>,
    /// Average game duration in ticks.
    pub avg_duration_ticks: f64,
    /// Shortest game.
    pub min_duration_ticks: u64,
    /// Longest game.
    pub max_duration_ticks: u64,
    /// Draws count.
    pub draws: u32,

    // === Aggregated Stats ===
    /// Average units produced per game by faction.
    pub avg_units_produced: HashMap<String, f64>,
    /// Average resources gathered per game by faction.
    pub avg_resources_gathered: HashMap<String, f64>,
    /// Average K/D ratio by faction.
    pub avg_kd_ratio: HashMap<String, f64>,
    /// First attack timing distribution (faction -> avg tick).
    pub avg_first_attack_tick: HashMap<String, f64>,
}

impl BatchSummary {
    /// Calculate summary from a list of game metrics.
    #[must_use]
    pub fn from_games(games: &[GameMetrics]) -> Self {
        if games.is_empty() {
            return Self::default();
        }

        let mut summary = Self {
            total_games: games.len() as u32,
            ..Default::default()
        };

        let mut duration_sum = 0u64;
        let mut min_duration = u64::MAX;
        let mut max_duration = 0u64;

        // Aggregate stats
        let mut faction_units_produced: HashMap<String, Vec<u32>> = HashMap::new();
        let mut faction_resources: HashMap<String, Vec<i64>> = HashMap::new();
        let mut faction_kd: HashMap<String, Vec<f64>> = HashMap::new();
        let mut faction_first_attack: HashMap<String, Vec<u64>> = HashMap::new();

        for game in games {
            // Duration stats
            duration_sum += game.duration_ticks;
            min_duration = min_duration.min(game.duration_ticks);
            max_duration = max_duration.max(game.duration_ticks);

            // Win tracking
            if let Some(winner) = &game.winner {
                *summary.wins_by_faction.entry(winner.clone()).or_default() += 1;
            } else {
                summary.draws += 1;
            }

            // Per-faction aggregation
            for (faction_id, faction) in &game.factions {
                let total_units: u32 = faction.units_produced.values().sum();
                faction_units_produced
                    .entry(faction_id.clone())
                    .or_default()
                    .push(total_units);

                faction_resources
                    .entry(faction_id.clone())
                    .or_default()
                    .push(faction.total_resources_gathered);

                faction_kd
                    .entry(faction_id.clone())
                    .or_default()
                    .push(faction.kd_ratio);

                if let Some(tick) = faction.first_attack_tick {
                    faction_first_attack
                        .entry(faction_id.clone())
                        .or_default()
                        .push(tick);
                }
            }
        }

        // Calculate averages
        summary.avg_duration_ticks = duration_sum as f64 / games.len() as f64;
        summary.min_duration_ticks = min_duration;
        summary.max_duration_ticks = max_duration;

        // Win rates
        for (faction, wins) in &summary.wins_by_faction {
            summary
                .win_rates
                .insert(faction.clone(), *wins as f64 / summary.total_games as f64);
        }

        // Average stats
        for (faction, values) in faction_units_produced {
            let avg = values.iter().sum::<u32>() as f64 / values.len() as f64;
            summary.avg_units_produced.insert(faction, avg);
        }

        for (faction, values) in faction_resources {
            let avg = values.iter().sum::<i64>() as f64 / values.len() as f64;
            summary.avg_resources_gathered.insert(faction, avg);
        }

        for (faction, values) in faction_kd {
            let avg = values.iter().filter(|v| v.is_finite()).sum::<f64>()
                / values.iter().filter(|v| v.is_finite()).count().max(1) as f64;
            summary.avg_kd_ratio.insert(faction, avg);
        }

        for (faction, values) in faction_first_attack {
            let avg = values.iter().sum::<u64>() as f64 / values.len() as f64;
            summary.avg_first_attack_tick.insert(faction, avg);
        }

        summary
    }

    /// Check if faction balance is within acceptable range.
    #[must_use]
    pub fn is_balanced(&self, threshold: f64) -> bool {
        // Win rates should be within threshold of 0.5
        for rate in self.win_rates.values() {
            if (rate - 0.5).abs() > threshold {
                return false;
            }
        }
        true
    }

    /// Get the dominant faction (if any).
    #[must_use]
    pub fn dominant_faction(&self, threshold: f64) -> Option<&String> {
        for (faction, rate) in &self.win_rates {
            if *rate > 0.5 + threshold {
                return Some(faction);
            }
        }
        None
    }
}

/// Metrics collector that tracks events during a game.
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// Current game metrics.
    metrics: GameMetrics,
    /// Current tick.
    current_tick: u64,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    #[must_use]
    pub fn new(game_id: &str, scenario: &str, seed: u64) -> Self {
        Self {
            metrics: GameMetrics::new(game_id, scenario, seed),
            current_tick: 0,
        }
    }

    /// Update the current tick.
    pub fn set_tick(&mut self, tick: u64) {
        self.current_tick = tick;
    }

    /// Record a unit production.
    pub fn on_unit_produced(&mut self, faction: &str, unit_type: &str) {
        self.metrics
            .faction_mut(faction)
            .record_unit_produced(unit_type);
        self.metrics.record_event(
            self.current_tick,
            EventType::UnitProduced,
            faction,
            unit_type,
        );
    }

    /// Record a unit death.
    pub fn on_unit_killed(&mut self, victim_faction: &str, killer_faction: &str, unit_type: &str) {
        self.metrics
            .faction_mut(victim_faction)
            .record_unit_lost(unit_type);
        self.metrics
            .faction_mut(killer_faction)
            .record_unit_killed(unit_type);
        self.metrics.record_event(
            self.current_tick,
            EventType::UnitKilled,
            victim_faction,
            unit_type,
        );
    }

    /// Record damage.
    pub fn on_damage(&mut self, attacker_faction: &str, target_faction: &str, amount: i64) {
        self.metrics
            .faction_mut(attacker_faction)
            .record_damage_dealt(amount);
        self.metrics
            .faction_mut(target_faction)
            .record_damage_taken(amount);
    }

    /// Record resources gathered.
    pub fn on_resources_gathered(&mut self, faction: &str, amount: i64) {
        self.metrics
            .faction_mut(faction)
            .record_resources_gathered(amount);
    }

    /// Record building construction.
    pub fn on_building_completed(&mut self, faction: &str, building_type: &str) {
        self.metrics
            .faction_mut(faction)
            .record_building_constructed(building_type);
        self.metrics.record_event(
            self.current_tick,
            EventType::BuildingCompleted,
            faction,
            building_type,
        );
    }

    /// Record first attack.
    pub fn on_first_attack(&mut self, faction: &str) {
        let faction_metrics = self.metrics.faction_mut(faction);
        if faction_metrics.first_attack_tick.is_none() {
            faction_metrics.first_attack_tick = Some(self.current_tick);
            self.metrics.record_event(
                self.current_tick,
                EventType::FirstAttack,
                faction,
                "First attack",
            );
        }
    }

    /// Finalize and return the metrics.
    #[must_use]
    pub fn finalize(mut self, winner: Option<String>, condition: &str) -> GameMetrics {
        self.metrics.finalize(self.current_tick, winner, condition);

        // Calculate derived stats for each faction
        for faction in self.metrics.factions.values_mut() {
            faction.calculate_derived_stats();
        }

        self.metrics
    }

    /// Get current metrics (immutable).
    #[must_use]
    pub fn current(&self) -> &GameMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_metrics_new() {
        let metrics = GameMetrics::new("game_001", "skirmish", 12345);
        assert_eq!(metrics.game_id, "game_001");
        assert_eq!(metrics.seed, 12345);
    }

    #[test]
    fn test_faction_metrics_recording() {
        let mut faction = FactionMetrics::new("continuity");
        faction.record_unit_produced("infantry");
        faction.record_unit_produced("infantry");
        faction.record_unit_killed("infantry");

        assert_eq!(faction.units_produced.get("infantry"), Some(&2));
        assert_eq!(faction.units_killed.get("infantry"), Some(&1));
    }

    #[test]
    fn test_kd_ratio_calculation() {
        let mut faction = FactionMetrics::new("test");
        faction.record_unit_killed("infantry");
        faction.record_unit_killed("infantry");
        faction.record_unit_lost("infantry");
        faction.calculate_derived_stats();

        assert!((faction.kd_ratio - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_batch_summary() {
        let mut game1 = GameMetrics::new("g1", "test", 1);
        game1.winner = Some("continuity".to_string());
        game1.duration_ticks = 1000;

        let mut game2 = GameMetrics::new("g2", "test", 2);
        game2.winner = Some("collegium".to_string());
        game2.duration_ticks = 2000;

        let summary = BatchSummary::from_games(&[game1, game2]);

        assert_eq!(summary.total_games, 2);
        assert_eq!(summary.wins_by_faction.get("continuity"), Some(&1));
        assert_eq!(summary.wins_by_faction.get("collegium"), Some(&1));
        assert!((summary.avg_duration_ticks - 1500.0).abs() < 0.001);
    }

    #[test]
    fn test_balance_check() {
        let mut summary = BatchSummary::default();
        summary.win_rates.insert("faction_a".to_string(), 0.52);
        summary.win_rates.insert("faction_b".to_string(), 0.48);

        assert!(summary.is_balanced(0.1));
        assert!(!summary.is_balanced(0.01));
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new("test", "scenario", 42);
        collector.set_tick(100);
        collector.on_unit_produced("continuity", "infantry");
        collector.on_resources_gathered("continuity", 500);

        let metrics = collector.finalize(Some("continuity".to_string()), "elimination");

        assert_eq!(metrics.winner, Some("continuity".to_string()));
        assert_eq!(
            metrics
                .factions
                .get("continuity")
                .unwrap()
                .units_produced
                .get("infantry"),
            Some(&1)
        );
    }
}
