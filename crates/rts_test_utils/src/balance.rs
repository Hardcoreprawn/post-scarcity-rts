//! Balance testing utilities for headless simulation.
//!
//! This module provides tools for running thousands of simulated battles
//! to verify game balance across units, factions, and matchups.

use rts_core::factions::FactionId;

/// Result of a simulated battle.
#[derive(Debug, Clone)]
pub struct BattleResult {
    /// The winning faction (None if draw/timeout).
    pub winner: Option<FactionId>,
    /// Simulation ticks elapsed.
    pub ticks: u64,
    /// Starting army value for faction A.
    pub army_value_a: i32,
    /// Starting army value for faction B.
    pub army_value_b: i32,
    /// Remaining army value for faction A.
    pub remaining_value_a: i32,
    /// Remaining army value for faction B.
    pub remaining_value_b: i32,
}

/// Statistics for a set of battles.
#[derive(Debug, Clone, Default)]
pub struct BattleStats {
    /// Total battles run.
    pub total_battles: u32,
    /// Wins for faction A.
    pub wins_a: u32,
    /// Wins for faction B.
    pub wins_b: u32,
    /// Draws (timeouts or simultaneous elimination).
    pub draws: u32,
    /// Average ticks to resolution.
    pub avg_ticks: f64,
    /// Average remaining value ratio (winner's remaining / loser's starting).
    pub avg_remaining_ratio: f64,
}

impl BattleStats {
    /// Calculate win rate for faction A (0.0 to 1.0).
    pub fn win_rate_a(&self) -> f64 {
        if self.total_battles == 0 {
            return 0.5;
        }
        self.wins_a as f64 / self.total_battles as f64
    }

    /// Calculate win rate for faction B (0.0 to 1.0).
    pub fn win_rate_b(&self) -> f64 {
        if self.total_battles == 0 {
            return 0.5;
        }
        self.wins_b as f64 / self.total_battles as f64
    }

    /// Check if matchup is balanced (within acceptable range).
    pub fn is_balanced(&self, min_rate: f64, max_rate: f64) -> bool {
        let rate = self.win_rate_a();
        rate >= min_rate && rate <= max_rate
    }
}

/// Unit composition for a battle.
#[derive(Debug, Clone, Default)]
pub struct ArmyComposition {
    /// Number of infantry units.
    pub infantry: u32,
    /// Number of ranger units.
    pub rangers: u32,
    /// Number of harvesters.
    pub harvesters: u32,
}

impl ArmyComposition {
    /// Create a new army composition.
    pub fn new(infantry: u32, rangers: u32, harvesters: u32) -> Self {
        Self {
            infantry,
            rangers,
            harvesters,
        }
    }

    /// Calculate total resource cost of army.
    pub fn total_cost(&self) -> i32 {
        (self.infantry as i32 * 50) + (self.rangers as i32 * 75) + (self.harvesters as i32 * 100)
    }

    /// Calculate total supply used.
    pub fn total_supply(&self) -> i32 {
        self.infantry as i32 + (self.rangers as i32 * 2) + (self.harvesters as i32 * 2)
    }

    /// Calculate total HP of army.
    pub fn total_hp(&self) -> i32 {
        (self.infantry as i32 * 100) + (self.rangers as i32 * 75) + (self.harvesters as i32 * 80)
    }

    /// Calculate theoretical DPS (damage per second).
    pub fn total_dps(&self) -> f32 {
        let base_damage = 15.0;
        let cooldown = 1.0;
        let dps_per_unit = base_damage / cooldown;

        // Harvesters don't fight effectively
        ((self.infantry + self.rangers) as f32) * dps_per_unit
    }
}

/// Unit type for matchup testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitType {
    /// Basic melee combat unit.
    Infantry,
    /// Ranged combat unit with lower HP.
    Ranger,
    /// Economic unit, weak in combat.
    Harvester,
}

impl UnitType {
    /// Get unit stats: (health, damage, armor, attack_range, cost).
    pub fn stats(&self) -> (i32, i32, i32, f32, i32) {
        match self {
            UnitType::Infantry => (100, 15, 0, 50.0, 50),
            UnitType::Ranger => (75, 15, 0, 150.0, 75),
            UnitType::Harvester => (80, 5, 0, 30.0, 100),
        }
    }

    /// Calculate time to kill another unit type (1v1).
    pub fn time_to_kill(&self, target: UnitType) -> f32 {
        let (_, my_damage, _, _, _) = self.stats();
        let (target_hp, _, target_armor, _, _) = target.stats();

        let actual_damage = (my_damage - target_armor).max(1);
        let attacks_needed = (target_hp as f32 / actual_damage as f32).ceil();
        let attack_cooldown = 1.0; // seconds

        attacks_needed * attack_cooldown
    }

    /// Calculate cost efficiency vs another unit.
    /// Returns how much value you get per resource spent.
    pub fn cost_efficiency_vs(&self, target: UnitType) -> f32 {
        let (_, _, _, _, my_cost) = self.stats();
        let (_, _, _, _, target_cost) = target.stats();

        let my_ttk = self.time_to_kill(target);
        let their_ttk = target.time_to_kill(*self);

        // If I kill them faster, I'm more efficient
        // Adjust for cost difference
        (their_ttk / my_ttk) * (target_cost as f32 / my_cost as f32)
    }
}

/// Generate a TTK (time-to-kill) matrix for all unit matchups.
pub fn generate_ttk_matrix() -> Vec<(UnitType, UnitType, f32)> {
    let units = [UnitType::Infantry, UnitType::Ranger, UnitType::Harvester];
    let mut results = Vec::new();

    for attacker in &units {
        for defender in &units {
            let ttk = attacker.time_to_kill(*defender);
            results.push((*attacker, *defender, ttk));
        }
    }

    results
}

/// Generate a cost efficiency matrix.
pub fn generate_efficiency_matrix() -> Vec<(UnitType, UnitType, f32)> {
    let units = [UnitType::Infantry, UnitType::Ranger, UnitType::Harvester];
    let mut results = Vec::new();

    for attacker in &units {
        for defender in &units {
            let efficiency = attacker.cost_efficiency_vs(*defender);
            results.push((*attacker, *defender, efficiency));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infantry_ttk_matrix() {
        let matrix = generate_ttk_matrix();

        // Infantry vs Infantry: 100 HP / 15 damage = 7 attacks = 7 seconds
        let inf_v_inf = matrix
            .iter()
            .find(|(a, d, _)| *a == UnitType::Infantry && *d == UnitType::Infantry)
            .map(|(_, _, ttk)| *ttk)
            .unwrap();
        assert_eq!(inf_v_inf, 7.0);

        // Infantry vs Ranger: 75 HP / 15 damage = 5 attacks = 5 seconds
        let inf_v_rang = matrix
            .iter()
            .find(|(a, d, _)| *a == UnitType::Infantry && *d == UnitType::Ranger)
            .map(|(_, _, ttk)| *ttk)
            .unwrap();
        assert_eq!(inf_v_rang, 5.0);
    }

    #[test]
    fn test_army_composition_cost() {
        let army = ArmyComposition::new(5, 3, 2);
        // 5*50 + 3*75 + 2*100 = 250 + 225 + 200 = 675
        assert_eq!(army.total_cost(), 675);
    }

    #[test]
    fn test_army_composition_supply() {
        let army = ArmyComposition::new(5, 3, 2);
        // 5*1 + 3*2 + 2*2 = 5 + 6 + 4 = 15
        assert_eq!(army.total_supply(), 15);
    }

    #[test]
    fn test_battle_stats_win_rate() {
        let stats = BattleStats {
            total_battles: 100,
            wins_a: 55,
            wins_b: 40,
            draws: 5,
            avg_ticks: 1000.0,
            avg_remaining_ratio: 0.3,
        };

        assert!((stats.win_rate_a() - 0.55).abs() < 0.001);
        assert!((stats.win_rate_b() - 0.40).abs() < 0.001);
        assert!(stats.is_balanced(0.45, 0.55));
    }

    #[test]
    fn test_ranger_has_range_advantage() {
        let (_, _, _, inf_range, _) = UnitType::Infantry.stats();
        let (_, _, _, rang_range, _) = UnitType::Ranger.stats();

        assert!(
            rang_range > inf_range * 2.0,
            "Rangers should outrange infantry significantly"
        );
    }

    #[test]
    fn test_infantry_is_cost_efficient() {
        // Infantry should be reasonably cost efficient vs rangers
        let efficiency = UnitType::Infantry.cost_efficiency_vs(UnitType::Ranger);

        // Infantry costs 50, kills ranger in 5s
        // Ranger costs 75, kills infantry in 7s
        // Efficiency = (7/5) * (75/50) = 1.4 * 1.5 = 2.1
        assert!(
            efficiency > 1.0,
            "Infantry should be cost efficient vs rangers in straight fight"
        );
    }
}
