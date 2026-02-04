//! Visual quality rating system.
//!
//! Analyzes game screenshots and rates visual quality on multiple dimensions:
//! - Unit readability and clarity
//! - Faction color distinction
//! - Battle formation quality
//! - Map coverage and positioning
//! - Visual balance (even distribution)

use crate::ascii_visualizer::ScreenshotState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Comprehensive visual quality score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualScore {
    /// Overall score (0-100).
    pub overall: u32,
    /// Individual dimension scores.
    pub dimensions: VisualDimensions,
    /// Issues detected.
    pub issues: Vec<VisualIssue>,
    /// Positive aspects found.
    pub positives: Vec<String>,
    /// Improvement suggestions.
    pub suggestions: Vec<String>,
}

/// Individual scoring dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDimensions {
    /// Unit clarity - can individual units be distinguished? (0-100)
    pub unit_clarity: u32,
    /// Faction distinction - are factions visually different? (0-100)
    pub faction_distinction: u32,
    /// Battle readability - can you understand the battle state? (0-100)
    pub battle_readability: u32,
    /// Map usage - are units well distributed? (0-100)
    pub map_usage: u32,
    /// Formation quality - do units form sensible groups? (0-100)
    pub formation_quality: u32,
    /// Visual balance - symmetric/fair feeling? (0-100)
    pub visual_balance: u32,
}

impl Default for VisualDimensions {
    fn default() -> Self {
        Self {
            unit_clarity: 50,
            faction_distinction: 50,
            battle_readability: 50,
            map_usage: 50,
            formation_quality: 50,
            visual_balance: 50,
        }
    }
}

/// A visual issue that affects score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualIssue {
    /// Issue severity (1-5, 5 being worst).
    pub severity: u32,
    /// Issue category.
    pub category: String,
    /// Description of the issue.
    pub description: String,
    /// Score penalty.
    pub penalty: u32,
}

/// Visual quality analyzer.
#[derive(Debug, Clone, Default)]
pub struct VisualAnalyzer {
    /// Minimum units for meaningful analysis.
    min_units: u32,
    /// Expected map size for coverage calculations (reserved for future use).
    #[allow(dead_code)]
    expected_map_size: (u32, u32),
    /// Overlap threshold for unit clustering issues.
    overlap_threshold: f32,
}

impl VisualAnalyzer {
    /// Create new analyzer.
    pub fn new() -> Self {
        Self {
            min_units: 5,
            expected_map_size: (512, 512),
            overlap_threshold: 10.0, // Units closer than this are "overlapping"
        }
    }

    /// Analyze a screenshot and produce a visual score.
    pub fn analyze(&self, state: &ScreenshotState) -> VisualScore {
        let mut dimensions = VisualDimensions::default();
        let mut issues: Vec<VisualIssue> = Vec::new();
        let mut positives: Vec<String> = Vec::new();
        let mut suggestions: Vec<String> = Vec::new();

        // 1. Analyze unit clarity (overlap detection)
        let (clarity_score, clarity_issues) = self.analyze_unit_clarity(state);
        dimensions.unit_clarity = clarity_score;
        issues.extend(clarity_issues);

        // 2. Analyze faction distinction
        let (faction_score, faction_issues) = self.analyze_faction_distinction(state);
        dimensions.faction_distinction = faction_score;
        issues.extend(faction_issues);

        // 3. Analyze battle readability
        let (battle_score, battle_issues) = self.analyze_battle_readability(state);
        dimensions.battle_readability = battle_score;
        issues.extend(battle_issues);

        // 4. Analyze map usage
        let (map_score, map_issues) = self.analyze_map_usage(state);
        dimensions.map_usage = map_score;
        issues.extend(map_issues);

        // 5. Analyze formation quality
        let (formation_score, formation_issues) = self.analyze_formation_quality(state);
        dimensions.formation_quality = formation_score;
        issues.extend(formation_issues);

        // 6. Analyze visual balance
        let (balance_score, balance_issues) = self.analyze_visual_balance(state);
        dimensions.visual_balance = balance_score;
        issues.extend(balance_issues);

        // Collect positives
        if clarity_score >= 80 {
            positives.push("Good unit separation - units clearly distinguishable".to_string());
        }
        if faction_score >= 80 {
            positives.push("Strong faction visual distinction".to_string());
        }
        if map_score >= 70 {
            positives.push("Good map coverage and usage".to_string());
        }
        if formation_score >= 70 {
            positives.push("Units form meaningful tactical groups".to_string());
        }

        // Generate suggestions based on issues
        for issue in &issues {
            match issue.category.as_str() {
                "unit_overlap" => {
                    suggestions
                        .push("Increase unit spacing or add formation spreading".to_string());
                }
                "faction_clustering" => {
                    suggestions
                        .push("Improve faction color contrast or add unit markers".to_string());
                }
                "map_unused" => {
                    suggestions.push("Encourage more map exploration and expansion".to_string());
                }
                "deathball" => {
                    suggestions
                        .push("Add formation controls to spread units during combat".to_string());
                }
                _ => {}
            }
        }
        suggestions.sort();
        suggestions.dedup();

        // Calculate overall score (weighted average)
        let overall = (dimensions.unit_clarity as f32 * 0.2
            + dimensions.faction_distinction as f32 * 0.2
            + dimensions.battle_readability as f32 * 0.25
            + dimensions.map_usage as f32 * 0.1
            + dimensions.formation_quality as f32 * 0.15
            + dimensions.visual_balance as f32 * 0.1) as u32;

        VisualScore {
            overall,
            dimensions,
            issues,
            positives,
            suggestions,
        }
    }

    fn analyze_unit_clarity(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();

        if state.units.len() < self.min_units as usize {
            return (100, issues); // Not enough units to analyze
        }

        // Count overlapping units
        let mut overlap_count = 0;
        let total_pairs = state.units.len() * (state.units.len() - 1) / 2;

        for i in 0..state.units.len() {
            for j in (i + 1)..state.units.len() {
                let u1 = &state.units[i];
                let u2 = &state.units[j];
                let dx = u1.position[0] - u2.position[0];
                let dy = u1.position[1] - u2.position[1];
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < self.overlap_threshold {
                    overlap_count += 1;
                }
            }
        }

        let overlap_ratio = if total_pairs > 0 {
            overlap_count as f32 / total_pairs as f32
        } else {
            0.0
        };

        // Score based on overlap ratio
        let mut score = ((1.0 - overlap_ratio) * 100.0) as u32;

        if overlap_ratio > 0.3 {
            issues.push(VisualIssue {
                severity: 4,
                category: "unit_overlap".to_string(),
                description: format!(
                    "{:.0}% of unit pairs overlap (too many units stacked)",
                    overlap_ratio * 100.0
                ),
                penalty: 30,
            });
            score = score.saturating_sub(30);
        } else if overlap_ratio > 0.1 {
            issues.push(VisualIssue {
                severity: 2,
                category: "unit_overlap".to_string(),
                description: format!("{:.0}% of unit pairs overlap", overlap_ratio * 100.0),
                penalty: 10,
            });
            score = score.saturating_sub(10);
        }

        (score.min(100), issues)
    }

    fn analyze_faction_distinction(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();

        // Count units per faction
        let mut faction_counts: HashMap<&str, u32> = HashMap::new();
        for unit in &state.units {
            *faction_counts.entry(&unit.faction).or_insert(0) += 1;
        }

        if faction_counts.len() < 2 {
            return (100, issues); // Only one faction visible
        }

        // Check for faction mingling in same area
        let _map_center = (
            state.map_bounds[0] as f32 / 2.0,
            state.map_bounds[1] as f32 / 2.0,
        );

        // Find center of mass for each faction
        let mut faction_centers: HashMap<&str, (f32, f32, u32)> = HashMap::new();
        for unit in &state.units {
            let entry = faction_centers
                .entry(&unit.faction)
                .or_insert((0.0, 0.0, 0));
            entry.0 += unit.position[0];
            entry.1 += unit.position[1];
            entry.2 += 1;
        }

        let centers: Vec<(&str, (f32, f32))> = faction_centers
            .iter()
            .map(|(k, v)| (*k, (v.0 / v.2 as f32, v.1 / v.2 as f32)))
            .collect();

        // Check if faction centers are far enough apart
        let mut score = 100u32;

        if centers.len() >= 2 {
            let dx = centers[0].1 .0 - centers[1].1 .0;
            let dy = centers[0].1 .1 - centers[1].1 .1;
            let center_distance = (dx * dx + dy * dy).sqrt();

            if center_distance < 50.0 {
                issues.push(VisualIssue {
                    severity: 3,
                    category: "faction_clustering".to_string(),
                    description: format!(
                        "Faction centers very close ({:.0} units apart)",
                        center_distance
                    ),
                    penalty: 25,
                });
                score = score.saturating_sub(25);
            }
        }

        // Check for intermingled units
        let mut intermingled_count = 0;
        for i in 0..state.units.len() {
            for j in (i + 1)..state.units.len() {
                if state.units[i].faction != state.units[j].faction {
                    let dx = state.units[i].position[0] - state.units[j].position[0];
                    let dy = state.units[i].position[1] - state.units[j].position[1];
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < 15.0 {
                        intermingled_count += 1;
                    }
                }
            }
        }

        if intermingled_count > state.units.len() / 4 {
            issues.push(VisualIssue {
                severity: 2,
                category: "faction_intermingled".to_string(),
                description: format!("{} enemy unit pairs in close proximity", intermingled_count),
                penalty: 15,
            });
            score = score.saturating_sub(15);
        }

        (score.min(100), issues)
    }

    fn analyze_battle_readability(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();
        let mut score = 100u32;

        // Check if we can understand what's happening

        // 1. Check for damaged units (indicates active combat)
        let damaged_count = state
            .units
            .iter()
            .filter(|u| u.health_percent < 1.0)
            .count();

        let total_units = state.units.len();

        if total_units > 10 && damaged_count == 0 {
            // Large battle but no damage - might be setup phase
            // Not an issue per se, but less interesting visually
        }

        // 2. Check unit density - too many units in small area?
        if total_units > 0 {
            let mut bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN); // minx, miny, maxx, maxy
            for unit in &state.units {
                bounds.0 = bounds.0.min(unit.position[0]);
                bounds.1 = bounds.1.min(unit.position[1]);
                bounds.2 = bounds.2.max(unit.position[0]);
                bounds.3 = bounds.3.max(unit.position[1]);
            }

            let battle_width = (bounds.2 - bounds.0).max(1.0);
            let battle_height = (bounds.3 - bounds.1).max(1.0);
            let battle_area = battle_width * battle_height;
            let density = total_units as f32 / battle_area;

            if density > 0.1 && total_units > 20 {
                issues.push(VisualIssue {
                    severity: 3,
                    category: "deathball".to_string(),
                    description: format!(
                        "{} units in tight {:.0}x{:.0} area (deathball formation)",
                        total_units, battle_width, battle_height
                    ),
                    penalty: 20,
                });
                score = score.saturating_sub(20);
            }
        }

        // 3. Check for variety in unit positions (not all in a line)
        if total_units >= 5 {
            // Simple check: is variance in both X and Y reasonable?
            let avg_x: f32 =
                state.units.iter().map(|u| u.position[0]).sum::<f32>() / total_units as f32;
            let avg_y: f32 =
                state.units.iter().map(|u| u.position[1]).sum::<f32>() / total_units as f32;

            let var_x: f32 = state
                .units
                .iter()
                .map(|u| (u.position[0] - avg_x).powi(2))
                .sum::<f32>()
                / total_units as f32;
            let var_y: f32 = state
                .units
                .iter()
                .map(|u| (u.position[1] - avg_y).powi(2))
                .sum::<f32>()
                / total_units as f32;

            // If one dimension has very low variance, units are in a line
            if (var_x < 100.0 || var_y < 100.0) && (var_x > 100.0 || var_y > 100.0) {
                // One axis has much more variance - could be a line formation
                // This is actually OK tactically
            }
        }

        (score.min(100), issues)
    }

    fn analyze_map_usage(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();
        let mut score = 100u32;

        if state.units.is_empty() {
            return (50, issues);
        }

        let map_w = state.map_bounds[0] as f32;
        let map_h = state.map_bounds[1] as f32;

        // Divide map into quadrants and check unit distribution
        let mut quadrant_counts = [0u32; 4]; // TL, TR, BL, BR

        for unit in &state.units {
            let qx = if unit.position[0] < map_w / 2.0 { 0 } else { 1 };
            let qy = if unit.position[1] < map_h / 2.0 { 0 } else { 1 };
            quadrant_counts[qy * 2 + qx] += 1;
        }

        let total = state.units.len() as u32;
        let max_in_quadrant = *quadrant_counts.iter().max().unwrap_or(&0);
        let empty_quadrants = quadrant_counts.iter().filter(|&&c| c == 0).count();

        // Penalize if all units in one quadrant
        if max_in_quadrant == total && total > 10 {
            issues.push(VisualIssue {
                severity: 2,
                category: "map_unused".to_string(),
                description: "All units concentrated in one quadrant".to_string(),
                penalty: 20,
            });
            score = score.saturating_sub(20);
        } else if empty_quadrants >= 3 && total > 15 {
            issues.push(VisualIssue {
                severity: 1,
                category: "map_unused".to_string(),
                description: format!("{} quadrants empty", empty_quadrants),
                penalty: 10,
            });
            score = score.saturating_sub(10);
        }

        (score.min(100), issues)
    }

    fn analyze_formation_quality(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();
        let mut score = 80u32; // Start at 80, adjust based on findings

        if state.units.len() < 5 {
            return (90, issues); // Too few units to judge formation
        }

        // Group units by faction
        let mut faction_units: HashMap<&str, Vec<&crate::ascii_visualizer::UnitSnapshot>> =
            HashMap::new();
        for unit in &state.units {
            faction_units.entry(&unit.faction).or_default().push(unit);
        }

        for (faction, units) in &faction_units {
            if units.len() < 3 {
                continue;
            }

            // Check for reasonable spread within faction
            let mut centroid = (0.0f32, 0.0f32);
            for unit in units {
                centroid.0 += unit.position[0];
                centroid.1 += unit.position[1];
            }
            centroid.0 /= units.len() as f32;
            centroid.1 /= units.len() as f32;

            // Calculate average distance from centroid
            let avg_dist: f32 = units
                .iter()
                .map(|u| {
                    let dx = u.position[0] - centroid.0;
                    let dy = u.position[1] - centroid.1;
                    (dx * dx + dy * dy).sqrt()
                })
                .sum::<f32>()
                / units.len() as f32;

            // Too tight formation?
            if avg_dist < 5.0 && units.len() > 5 {
                issues.push(VisualIssue {
                    severity: 2,
                    category: "formation_tight".to_string(),
                    description: format!(
                        "{} units in very tight formation (avg spread: {:.1})",
                        faction, avg_dist
                    ),
                    penalty: 15,
                });
                score = score.saturating_sub(15);
            }

            // Good formation gets bonus
            if avg_dist >= 10.0 && avg_dist <= 50.0 {
                score = score.saturating_add(10);
            }
        }

        (score.min(100), issues)
    }

    fn analyze_visual_balance(&self, state: &ScreenshotState) -> (u32, Vec<VisualIssue>) {
        let mut issues = Vec::new();
        let mut score = 100u32;

        // Count units per faction
        let mut faction_counts: HashMap<&str, u32> = HashMap::new();
        for unit in &state.units {
            *faction_counts.entry(&unit.faction).or_insert(0) += 1;
        }

        if faction_counts.len() < 2 {
            return (100, issues);
        }

        let counts: Vec<u32> = faction_counts.values().copied().collect();
        if counts.len() >= 2 {
            let max = *counts.iter().max().unwrap_or(&1);
            let min = *counts.iter().min().unwrap_or(&1);

            let ratio = if min > 0 {
                max as f32 / min as f32
            } else {
                10.0
            };

            if ratio > 3.0 {
                issues.push(VisualIssue {
                    severity: 2,
                    category: "army_imbalance".to_string(),
                    description: format!("Large unit count imbalance ({:.1}:1)", ratio),
                    penalty: 20,
                });
                score = score.saturating_sub(20);
            } else if ratio > 2.0 {
                issues.push(VisualIssue {
                    severity: 1,
                    category: "army_imbalance".to_string(),
                    description: format!("Moderate unit count imbalance ({:.1}:1)", ratio),
                    penalty: 10,
                });
                score = score.saturating_sub(10);
            }
        }

        (score.min(100), issues)
    }

    /// Analyze a batch of screenshots and produce aggregate scores.
    pub fn analyze_batch(&self, screenshots: &[ScreenshotState]) -> BatchVisualScore {
        let scores: Vec<VisualScore> = screenshots.iter().map(|s| self.analyze(s)).collect();

        if scores.is_empty() {
            return BatchVisualScore::default();
        }

        let avg_overall =
            scores.iter().map(|s| s.overall).sum::<u32>() as f32 / scores.len() as f32;
        let min_overall = scores.iter().map(|s| s.overall).min().unwrap_or(0);
        let max_overall = scores.iter().map(|s| s.overall).max().unwrap_or(100);

        // Collect common issues
        let mut issue_counts: HashMap<String, u32> = HashMap::new();
        for score in &scores {
            for issue in &score.issues {
                *issue_counts.entry(issue.category.clone()).or_insert(0) += 1;
            }
        }

        let mut common_issues: Vec<(String, u32)> = issue_counts.into_iter().collect();
        common_issues.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        // Aggregate dimension scores
        let avg_dimensions = VisualDimensions {
            unit_clarity: (scores
                .iter()
                .map(|s| s.dimensions.unit_clarity)
                .sum::<u32>() as f32
                / scores.len() as f32) as u32,
            faction_distinction: (scores
                .iter()
                .map(|s| s.dimensions.faction_distinction)
                .sum::<u32>() as f32
                / scores.len() as f32) as u32,
            battle_readability: (scores
                .iter()
                .map(|s| s.dimensions.battle_readability)
                .sum::<u32>() as f32
                / scores.len() as f32) as u32,
            map_usage: (scores.iter().map(|s| s.dimensions.map_usage).sum::<u32>() as f32
                / scores.len() as f32) as u32,
            formation_quality: (scores
                .iter()
                .map(|s| s.dimensions.formation_quality)
                .sum::<u32>() as f32
                / scores.len() as f32) as u32,
            visual_balance: (scores
                .iter()
                .map(|s| s.dimensions.visual_balance)
                .sum::<u32>() as f32
                / scores.len() as f32) as u32,
        };

        BatchVisualScore {
            average_overall: avg_overall,
            min_overall,
            max_overall,
            sample_count: scores.len() as u32,
            common_issues,
            average_dimensions: avg_dimensions,
            individual_scores: scores,
        }
    }
}

/// Aggregate visual score for a batch of screenshots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchVisualScore {
    /// Average overall score.
    pub average_overall: f32,
    /// Minimum score in batch.
    pub min_overall: u32,
    /// Maximum score in batch.
    pub max_overall: u32,
    /// Number of screenshots analyzed.
    pub sample_count: u32,
    /// Most common issues (category, count).
    pub common_issues: Vec<(String, u32)>,
    /// Average dimension scores.
    pub average_dimensions: VisualDimensions,
    /// Individual scores.
    pub individual_scores: Vec<VisualScore>,
}

impl BatchVisualScore {
    /// Generate a markdown report.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Visual Quality Report\n\n");
        md.push_str(&format!("**Samples Analyzed:** {}\n\n", self.sample_count));
        md.push_str(&format!(
            "## Overall Score: {:.1}/100\n\n",
            self.average_overall
        ));
        md.push_str(&format!("- Min: {}\n", self.min_overall));
        md.push_str(&format!("- Max: {}\n\n", self.max_overall));

        md.push_str("## Dimension Scores\n\n");
        md.push_str("| Dimension | Score |\n");
        md.push_str("|-----------|-------|\n");
        md.push_str(&format!(
            "| Unit Clarity | {} |\n",
            self.average_dimensions.unit_clarity
        ));
        md.push_str(&format!(
            "| Faction Distinction | {} |\n",
            self.average_dimensions.faction_distinction
        ));
        md.push_str(&format!(
            "| Battle Readability | {} |\n",
            self.average_dimensions.battle_readability
        ));
        md.push_str(&format!(
            "| Map Usage | {} |\n",
            self.average_dimensions.map_usage
        ));
        md.push_str(&format!(
            "| Formation Quality | {} |\n",
            self.average_dimensions.formation_quality
        ));
        md.push_str(&format!(
            "| Visual Balance | {} |\n\n",
            self.average_dimensions.visual_balance
        ));

        if !self.common_issues.is_empty() {
            md.push_str("## Common Issues\n\n");
            for (issue, count) in &self.common_issues {
                let pct = (*count as f32 / self.sample_count as f32) * 100.0;
                md.push_str(&format!(
                    "- **{}**: {} occurrences ({:.0}%)\n",
                    issue, count, pct
                ));
            }
        }

        md
    }

    /// Check if visuals pass quality bar.
    pub fn passes_quality_bar(&self, min_score: u32) -> bool {
        self.average_overall >= min_score as f32
    }

    /// Get improvement priority list.
    pub fn improvement_priorities(&self) -> Vec<String> {
        let mut priorities = Vec::new();

        let dims = &self.average_dimensions;
        let mut dim_scores = vec![
            ("Unit Clarity", dims.unit_clarity),
            ("Faction Distinction", dims.faction_distinction),
            ("Battle Readability", dims.battle_readability),
            ("Map Usage", dims.map_usage),
            ("Formation Quality", dims.formation_quality),
            ("Visual Balance", dims.visual_balance),
        ];

        dim_scores.sort_by_key(|(_, score)| *score);

        for (name, score) in dim_scores.iter().take(3) {
            if *score < 70 {
                priorities.push(format!("Improve {} (currently {})", name, score));
            }
        }

        priorities
    }
}

/// Load and analyze all screenshots in a directory.
pub fn analyze_screenshots_in_dir(path: &Path) -> std::io::Result<BatchVisualScore> {
    let mut screenshots = Vec::new();

    fn collect_screenshots(
        path: &Path,
        screenshots: &mut Vec<ScreenshotState>,
    ) -> std::io::Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                collect_screenshots(&entry.path(), screenshots)?;
            }
        } else if path.extension().map_or(false, |e| e == "json") {
            if let Ok(state) = ScreenshotState::load(path) {
                screenshots.push(state);
            }
        }
        Ok(())
    }

    collect_screenshots(path, &mut screenshots)?;

    let analyzer = VisualAnalyzer::new();
    Ok(analyzer.analyze_batch(&screenshots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ascii_visualizer::UnitSnapshot;

    fn make_unit(faction: &str, x: f32, y: f32) -> UnitSnapshot {
        UnitSnapshot {
            entity_id: 1,
            kind: "infantry".to_string(),
            faction: faction.to_string(),
            position: [x, y],
            rotation: 0.0,
            health_percent: 1.0,
            animation_state: "idle".to_string(),
            animation_frame: 0,
            is_selected: false,
            current_action: None,
        }
    }

    fn make_state(units: Vec<UnitSnapshot>) -> ScreenshotState {
        ScreenshotState {
            tick: 100,
            game_id: "test".to_string(),
            trigger: serde_json::json!({}),
            camera: serde_json::json!({}),
            units,
            buildings: vec![],
            projectiles: vec![],
            effects: vec![],
            map_bounds: [512, 512],
            fog_of_war: None,
        }
    }

    #[test]
    fn test_overlapping_units_detected() {
        let units = vec![
            make_unit("a", 100.0, 100.0),
            make_unit("a", 101.0, 101.0),
            make_unit("a", 102.0, 102.0),
            make_unit("a", 103.0, 103.0),
            make_unit("a", 104.0, 104.0),
            make_unit("a", 105.0, 105.0),
        ];
        let state = make_state(units);

        let analyzer = VisualAnalyzer::new();
        let score = analyzer.analyze(&state);

        assert!(score.dimensions.unit_clarity < 80, "Should detect overlap");
        assert!(score.issues.iter().any(|i| i.category == "unit_overlap"));
    }

    #[test]
    fn test_well_spread_units() {
        let units = vec![
            make_unit("a", 100.0, 100.0),
            make_unit("a", 200.0, 100.0),
            make_unit("b", 300.0, 100.0),
            make_unit("b", 400.0, 100.0),
            make_unit("a", 100.0, 200.0),
        ];
        let state = make_state(units);

        let analyzer = VisualAnalyzer::new();
        let score = analyzer.analyze(&state);

        assert!(
            score.dimensions.unit_clarity >= 80,
            "Should have good clarity"
        );
    }

    #[test]
    fn test_faction_distinction() {
        // Factions well separated
        let units = vec![
            make_unit("continuity", 100.0, 256.0),
            make_unit("continuity", 120.0, 256.0),
            make_unit("collegium", 400.0, 256.0),
            make_unit("collegium", 420.0, 256.0),
        ];
        let state = make_state(units);

        let analyzer = VisualAnalyzer::new();
        let score = analyzer.analyze(&state);

        assert!(score.dimensions.faction_distinction >= 80);
    }

    #[test]
    fn test_batch_analysis() {
        let states: Vec<ScreenshotState> = (0..10)
            .map(|i| {
                make_state(vec![
                    make_unit("a", 100.0 + i as f32 * 10.0, 100.0),
                    make_unit("b", 400.0, 100.0 + i as f32 * 10.0),
                ])
            })
            .collect();

        let analyzer = VisualAnalyzer::new();
        let batch_score = analyzer.analyze_batch(&states);

        assert_eq!(batch_score.sample_count, 10);
        assert!(batch_score.average_overall > 0.0);
    }

    #[test]
    fn test_quality_bar() {
        let batch = BatchVisualScore {
            average_overall: 75.0,
            min_overall: 60,
            max_overall: 90,
            sample_count: 10,
            common_issues: vec![],
            average_dimensions: VisualDimensions::default(),
            individual_scores: vec![],
        };

        assert!(batch.passes_quality_bar(70));
        assert!(!batch.passes_quality_bar(80));
    }
}
