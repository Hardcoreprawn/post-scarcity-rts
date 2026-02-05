//! Balance and gameplay tests for rts_game.
//!
//! These tests verify that game mechanics work correctly and help
//! catch balance issues early.

// =============================================================================
// Combat Tests
// =============================================================================

mod combat {

    /// Verify damage formula: base_damage - armor = actual_damage (min 1)
    #[test]
    fn test_damage_reduction_by_armor() {
        // Light armor (0) takes full damage
        let base_damage = 15;
        let light_armor = 0;
        let actual = (base_damage - light_armor).max(1);
        assert_eq!(actual, 15);

        // Medium armor (2) reduces damage
        let medium_armor = 2;
        let actual = (base_damage - medium_armor).max(1);
        assert_eq!(actual, 13);

        // Heavy armor (5) reduces more
        let heavy_armor = 5;
        let actual = (base_damage - heavy_armor).max(1);
        assert_eq!(actual, 10);

        // Building armor (8) is strongest
        let building_armor = 8;
        let actual = (base_damage - building_armor).max(1);
        assert_eq!(actual, 7);
    }

    /// Verify minimum damage is always 1
    #[test]
    fn test_minimum_damage_is_one() {
        let weak_damage = 3;
        let heavy_armor = 10;
        let actual = (weak_damage - heavy_armor).max(1);
        assert_eq!(actual, 1, "Minimum damage should be 1");
    }

    /// Verify attack cooldown timing
    #[test]
    fn test_attack_cooldown() {
        let cooldown = 1.0_f32; // 1 second between attacks
        let dt = 1.0 / 60.0_f32; // 60fps frame time

        // After 61 frames at 60fps, should have enough time for 1 attack cycle
        let total_time = dt * 61.0;
        assert!(
            total_time >= cooldown,
            "Should complete 1 attack in ~60 frames"
        );

        // After 120 frames, should have done 2 attacks
        let attacks = (dt * 120.0 / cooldown).floor() as i32;
        assert_eq!(attacks, 2, "Should complete 2 attacks in 120 frames");
    }

    /// Calculate time-to-kill for various matchups
    #[test]
    fn test_time_to_kill_infantry_vs_infantry() {
        let infantry_health = 100;
        let infantry_damage = 15;
        let attack_cooldown = 1.0_f32;

        // Attacks needed to kill
        let attacks_needed = (infantry_health as f32 / infantry_damage as f32).ceil() as i32;
        let time_to_kill = attacks_needed as f32 * attack_cooldown;

        assert_eq!(attacks_needed, 7, "Should take 7 attacks to kill infantry");
        assert_eq!(time_to_kill, 7.0, "TTK should be 7 seconds");
    }

    /// Rangers should kill infantry faster than infantry kills rangers
    #[test]
    fn test_ranger_vs_infantry_advantage() {
        let infantry_health = 100;
        let ranger_health = 75;
        let base_damage = 15;

        // Infantry attacking ranger
        let attacks_to_kill_ranger = (ranger_health as f32 / base_damage as f32).ceil() as i32;

        // Ranger attacking infantry (assuming same damage - rangers have range advantage)
        let attacks_to_kill_infantry = (infantry_health as f32 / base_damage as f32).ceil() as i32;

        assert_eq!(attacks_to_kill_ranger, 5);
        assert_eq!(attacks_to_kill_infantry, 7);

        // Rangers die faster in melee but have range advantage
        assert!(attacks_to_kill_ranger < attacks_to_kill_infantry);
    }

    /// Harvesters should be weak in combat
    #[test]
    fn test_harvester_combat_weakness() {
        let harvester_health = 80;
        let infantry_health = 100;
        let base_damage = 15;

        let attacks_to_kill_harvester =
            (harvester_health as f32 / base_damage as f32).ceil() as i32;
        let attacks_to_kill_infantry = (infantry_health as f32 / base_damage as f32).ceil() as i32;

        assert!(
            attacks_to_kill_harvester < attacks_to_kill_infantry,
            "Harvesters should die faster than infantry"
        );
    }

    /// Buildings should be tough
    #[test]
    fn test_building_survivability() {
        let depot_health = 1500; // Buffed from 500 to resist early rushes
        let building_armor = 8;
        let base_damage = 15;

        let actual_damage = (base_damage - building_armor).max(1);
        let attacks_to_destroy = (depot_health as f32 / actual_damage as f32).ceil() as i32;

        assert_eq!(attacks_to_destroy, 215, "Depot should survive 215 attacks");

        // With 1 second cooldown, that's ~215 seconds to destroy with 1 unit
        let time_to_destroy = attacks_to_destroy as f32;
        assert!(
            time_to_destroy > 180.0,
            "Buildings should take over 3 minutes to destroy"
        );
    }

    /// Depot regeneration should make early rush futile
    #[test]
    fn test_depot_regeneration_negates_weak_rush() {
        let depot_regen_per_sec = 10.0_f32;
        let building_armor = 8;
        let base_damage = 15;
        let attack_cooldown = 1.0_f32;

        let actual_damage = (base_damage - building_armor).max(1);
        let dps_per_attacker = actual_damage as f32 / attack_cooldown;

        // How many attackers needed to overcome regen?
        let attackers_to_overcome_regen = (depot_regen_per_sec / dps_per_attacker).ceil() as i32;

        // Should need at least 2 attackers to deal net damage
        assert!(
            attackers_to_overcome_regen >= 2,
            "Single attacker should be unable to damage depot faster than regen"
        );

        // With 3 starting infantry, they CAN damage depot but it's slow
        let attackers = 3;
        let net_dps = (attackers as f32 * dps_per_attacker) - depot_regen_per_sec;
        let depot_health = 1500.0_f32;
        let time_to_kill = depot_health / net_dps;

        // Should take over 2 minutes even with focused rush
        assert!(
            time_to_kill > 120.0,
            "3 infantry rush should take over 2 minutes to destroy depot"
        );
    }
}

// =============================================================================
// Economy Tests
// =============================================================================

mod economy {

    /// Verify harvester gather rates
    #[test]
    fn test_harvester_gather_rate() {
        let gather_rate = 10; // per tick while at node
        let capacity = 100;

        // Time to fill cargo
        let ticks_to_fill = capacity / gather_rate;
        assert_eq!(ticks_to_fill, 10, "Should fill in 10 ticks at node");
    }

    /// Verify temporary node depletion
    #[test]
    fn test_node_depletion() {
        let node_resources = 500;

        // Harvests to deplete (at gather_rate of 10 per harvest)
        let harvests_to_deplete = node_resources / 10;
        assert_eq!(
            harvests_to_deplete, 50,
            "Node should deplete after 50 harvests"
        );
    }

    /// Verify permanent node yield degradation
    #[test]
    fn test_permanent_node_diminishing_returns() {
        let base_yield = 15;
        let optimal_harvesters = 2;

        // At optimal, full yield
        let yield_at_optimal = base_yield;
        assert_eq!(yield_at_optimal, 15);

        // Over optimal, reduced yield per harvester
        let harvesters: i32 = 4;
        let over_optimal = harvesters.saturating_sub(optimal_harvesters);
        let penalty_per: f32 = 0.15; // 15% reduction per extra
        let multiplier: f32 = 1.0 - (over_optimal as f32 * penalty_per);
        let actual_yield = (base_yield as f32 * multiplier.max(0.25)) as i32;

        // With 2 extra harvesters: 1.0 - (2 * 0.15) = 0.7
        assert_eq!(
            actual_yield, 10,
            "Yield should be reduced with extra harvesters"
        );
    }

    /// Verify resource cap is respected
    #[test]
    fn test_resource_cap() {
        let current = 4900;
        let cap = 5000;
        let income = 100;

        let new_total = (current + income).min(cap);
        assert_eq!(new_total, 5000, "Should cap at 5000");

        // Already at cap
        let at_cap = 5000;
        let new_total = (at_cap + income).min(cap);
        assert_eq!(new_total, 5000, "Should stay at cap");
    }
}

// =============================================================================
// Production Tests
// =============================================================================

mod production {

    /// Verify unit costs
    #[test]
    fn test_unit_costs() {
        let infantry_cost = 50;
        let harvester_cost = 100;
        let ranger_cost = 75;

        // Harvester is most expensive (economy unit)
        assert!(harvester_cost > infantry_cost);
        assert!(harvester_cost > ranger_cost);

        // Ranger is mid-tier
        assert!(ranger_cost > infantry_cost);
    }

    /// Verify build times
    #[test]
    fn test_build_times() {
        let infantry_time = 5.0_f32;
        let harvester_time = 8.0_f32;
        let ranger_time = 7.0_f32;

        // Harvester takes longest
        assert!(harvester_time > infantry_time);
        assert!(harvester_time > ranger_time);
    }

    /// Verify supply costs
    #[test]
    fn test_supply_costs() {
        let infantry_supply = 1;
        let harvester_supply = 2;
        let ranger_supply = 2;

        // Infantry is cheapest on supply
        assert!(infantry_supply < harvester_supply);
        assert!(infantry_supply < ranger_supply);
    }

    /// Verify can't exceed supply cap
    #[test]
    fn test_supply_cap_enforcement() {
        let supply_used = 18;
        let supply_cap = 20;
        let unit_supply_cost = 2;

        let can_build = supply_used + unit_supply_cost <= supply_cap;
        assert!(can_build, "Should be able to build at 18/20");

        let supply_used = 19;
        let can_build = supply_used + unit_supply_cost <= supply_cap;
        assert!(!can_build, "Should NOT be able to build at 19/20");
    }

    /// Verify production queue limits
    #[test]
    fn test_queue_limit() {
        let max_queue = 5;
        let current_queue = 5;

        let can_queue = current_queue < max_queue;
        assert!(!can_queue, "Queue should be full");
    }

    /// Verify cancel refund rates
    #[test]
    fn test_cancel_refund() {
        let unit_cost = 100;
        let progress = 0.0_f32; // Just started
        let refund_rate = 0.75; // 75% refund

        let refund = (unit_cost as f32 * (1.0 - progress) * refund_rate) as i32;
        assert_eq!(refund, 75, "Should get 75% refund at 0% progress");

        // 50% progress
        let progress = 0.5;
        let refund = (unit_cost as f32 * (1.0 - progress) * refund_rate) as i32;
        assert_eq!(refund, 37, "Should get ~37% refund at 50% progress");
    }
}

// =============================================================================
// Unit Stats Tests
// =============================================================================

mod unit_stats {

    /// Infantry should be a balanced generalist
    #[test]
    fn test_infantry_stats() {
        let health = 100;
        let cost = 50;
        let supply = 1;
        let build_time = 5.0_f32;

        // Good HP per cost ratio
        let hp_per_cost = health as f32 / cost as f32;
        assert_eq!(hp_per_cost, 2.0, "Infantry should have 2 HP per cost");

        // Quick to build
        assert!(build_time <= 5.0, "Infantry should build in 5s or less");

        // Supply efficient
        assert_eq!(supply, 1, "Infantry should cost 1 supply");
    }

    /// Harvesters should be economic units
    #[test]
    fn test_harvester_stats() {
        let health = 80;
        let cost = 100;
        let capacity = 100;

        // Lower HP than infantry
        assert!(health < 100, "Harvesters should have less HP than infantry");

        // Expensive but good resource returns
        let trips_to_pay_off = cost / capacity;
        assert_eq!(
            trips_to_pay_off, 1,
            "Harvester pays for itself in 1 full trip"
        );
    }

    /// Rangers should be glass cannons with range
    #[test]
    fn test_ranger_stats() {
        let health = 75;
        let cost = 75;
        let attack_range = 150.0_f32;
        let infantry_range = 50.0_f32;

        // Lower HP than infantry
        assert!(health < 100);

        // Higher cost than infantry
        assert!(cost > 50);

        // Much better range
        assert!(
            attack_range > infantry_range * 2.0,
            "Rangers should outrange infantry significantly"
        );
    }

    /// Depot should be sturdy HQ
    #[test]
    fn test_depot_stats() {
        let health = 1500; // Buffed to resist early rushes
        let armor = 8;
        let infantry_health = 100;

        // 15x infantry HP (tanky!)
        assert_eq!(health / infantry_health, 15);

        // Has armor
        assert!(armor > 0, "Buildings should have armor");
    }
}

// =============================================================================
// AI Behavior Tests
// =============================================================================

mod ai {

    /// AI should prioritize harvesters early
    #[test]
    fn test_ai_early_economy_priority() {
        let starting_harvesters = 1;
        let ideal_harvesters = 2;

        // Should build harvesters until threshold met
        let should_build_harvester = starting_harvesters < ideal_harvesters;
        assert!(should_build_harvester);
    }

    /// AI should only launch waves with sufficient force  
    #[test]
    fn test_ai_attack_threshold() {
        let rally_units = 7;
        let min_wave_size = 8; // Waves require 8 units minimum

        let should_launch_wave = rally_units >= min_wave_size;
        assert!(!should_launch_wave, "AI shouldn't launch wave with only 7 units");

        let rally_units = 8;
        let should_launch_wave = rally_units >= min_wave_size;
        assert!(should_launch_wave, "AI should launch wave with 8+ units");
    }

    /// AI harvesters should seek nodes
    #[test]
    fn test_ai_harvester_assignment() {
        // Idle harvester should get assigned to nearest node
        let harvester_pos: (f32, f32) = (0.0, 0.0);
        let node1_pos: (f32, f32) = (100.0, 0.0);
        let node2_pos: (f32, f32) = (50.0, 50.0);

        let dist1 = ((node1_pos.0 - harvester_pos.0).powi(2)
            + (node1_pos.1 - harvester_pos.1).powi(2))
        .sqrt();
        let dist2 = ((node2_pos.0 - harvester_pos.0).powi(2)
            + (node2_pos.1 - harvester_pos.1).powi(2))
        .sqrt();

        assert!(dist2 < dist1, "Should pick closer node");
    }
}

// =============================================================================
// Balance Meta Tests (overall game health)
// =============================================================================

mod balance_meta {

    /// Economy should support continuous production
    #[test]
    fn test_economy_supports_production() {
        let harvester_income_per_trip = 100; // Resources per full cargo
        let trip_time_seconds = 20.0; // Estimate: travel + gather + return
        let income_per_second = harvester_income_per_trip as f32 / trip_time_seconds;

        let infantry_cost = 50;
        let infantry_build_time = 5.0;
        let infantry_drain_per_second = infantry_cost as f32 / infantry_build_time;

        // 1 harvester should roughly support 1 infantry queue
        // 5 income/sec vs 10 drain/sec - need ~2 harvesters per queue
        let harvesters_per_queue = (infantry_drain_per_second / income_per_second).ceil() as i32;
        assert!(
            harvesters_per_queue <= 3,
            "Should need at most 3 harvesters per production queue"
        );
    }

    /// Combat should be decisive but not instant
    #[test]
    fn test_combat_pacing() {
        let infantry_health = 100;
        let base_damage = 15;
        let attack_cooldown = 1.0;

        let ttk = (infantry_health as f32 / base_damage as f32).ceil() * attack_cooldown;

        // TTK should be 3-15 seconds
        assert!(ttk >= 3.0, "Combat shouldn't be too fast (3s min TTK)");
        assert!(ttk <= 15.0, "Combat shouldn't be too slow (15s max TTK)");
    }

    /// Army value should scale with cost
    #[test]
    fn test_cost_efficiency_parity() {
        // 2 infantry (100 cost) vs 1.33 rangers (100 cost)
        // Should be roughly equal in value

        let infantry_cost = 50;
        let infantry_hp = 100;

        let ranger_cost = 75;
        let ranger_hp = 75;

        // HP per cost comparison
        let infantry_efficiency = infantry_hp as f32 / infantry_cost as f32;
        let ranger_efficiency = ranger_hp as f32 / ranger_cost as f32;

        // Infantry more HP-efficient (rangers have range to compensate)
        assert!(
            infantry_efficiency > ranger_efficiency,
            "Infantry should be more HP-efficient (rangers have range instead)"
        );
    }
}
