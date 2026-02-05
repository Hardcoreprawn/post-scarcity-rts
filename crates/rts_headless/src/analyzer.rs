//! Balance analysis and auto-suggestion system.
//!
//! Analyzes batch run results to detect balance issues and
//! generate suggestions for tuning.

use crate::batch::BatchResults;
use crate::metrics::GameMetrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity of a balance issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Minor issue, low priority
    Low,
    /// Noticeable imbalance
    Medium,
    /// Significant problem requiring attention
    High,
    /// Critical imbalance breaking gameplay
    Critical,
}

impl Severity {
    /// Get numeric priority (higher = more urgent)
    pub fn priority(&self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

/// A detected balance outlier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceOutlier {
    /// Category of the issue (win_rate, timing, economy, etc.)
    pub category: String,
    /// Specific metric that's out of range
    pub metric: String,
    /// Observed value
    pub value: f64,
    /// Expected acceptable range
    pub expected_range: (f64, f64),
    /// Issue severity
    pub severity: Severity,
    /// Additional context
    pub context: String,
}

impl BalanceOutlier {
    /// Create new outlier
    pub fn new(
        category: &str,
        metric: &str,
        value: f64,
        range: (f64, f64),
        severity: Severity,
    ) -> Self {
        Self {
            category: category.to_string(),
            metric: metric.to_string(),
            value,
            expected_range: range,
            severity,
            context: String::new(),
        }
    }

    /// Add context
    pub fn with_context(mut self, ctx: &str) -> Self {
        self.context = ctx.to_string();
        self
    }
}

/// A balance tuning suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSuggestion {
    /// Target to modify (e.g., "continuity.tank.damage")
    pub target: String,
    /// Current value
    pub current: f64,
    /// Suggested new value
    pub suggested: f64,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Confidence in this suggestion (0.0 - 1.0)
    pub confidence: f64,
    /// Related outliers
    pub related_outliers: Vec<String>,
}

impl BalanceSuggestion {
    /// Create new suggestion
    pub fn new(target: &str, current: f64, suggested: f64, reasoning: &str) -> Self {
        Self {
            target: target.to_string(),
            current,
            suggested,
            reasoning: reasoning.to_string(),
            confidence: 0.5,
            related_outliers: Vec::new(),
        }
    }

    /// Set confidence
    pub fn with_confidence(mut self, conf: f64) -> Self {
        self.confidence = conf.clamp(0.0, 1.0);
        self
    }

    /// Add related outlier
    pub fn with_outlier(mut self, outlier: &str) -> Self {
        self.related_outliers.push(outlier.to_string());
        self
    }

    /// Calculate percent change
    pub fn percent_change(&self) -> f64 {
        if self.current == 0.0 {
            return 0.0;
        }
        (self.suggested - self.current) / self.current * 100.0
    }
}

/// Complete balance analysis report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceAnalysis {
    /// Win rates by faction
    pub win_rates: HashMap<String, f64>,
    /// Matchup matrix (faction_a vs faction_b -> win rate for a)
    pub matchup_matrix: HashMap<(String, String), f64>,
    /// Detected outliers
    pub outliers: Vec<BalanceOutlier>,
    /// Generated suggestions
    pub suggestions: Vec<BalanceSuggestion>,
    /// Games analyzed
    pub games_analyzed: u32,
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Metadata about the analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisMetadata {
    /// Timestamp
    pub timestamp: String,
    /// Source batch ID
    pub source_batch: String,
    /// Analysis version
    pub version: String,
}

impl BalanceAnalysis {
    /// Create new analysis
    pub fn new() -> Self {
        Self {
            metadata: AnalysisMetadata {
                version: "1.0".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Check if any critical issues exist
    pub fn has_critical_issues(&self) -> bool {
        self.outliers
            .iter()
            .any(|o| o.severity == Severity::Critical)
    }

    /// Get outliers sorted by severity
    pub fn outliers_by_severity(&self) -> Vec<&BalanceOutlier> {
        let mut sorted: Vec<_> = self.outliers.iter().collect();
        sorted.sort_by(|a, b| b.severity.priority().cmp(&a.severity.priority()));
        sorted
    }

    /// Get suggestions sorted by confidence
    pub fn suggestions_by_confidence(&self) -> Vec<&BalanceSuggestion> {
        let mut sorted: Vec<_> = self.suggestions.iter().collect();
        sorted.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        sorted
    }

    /// Save to JSON
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load from JSON
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(std::io::Error::other)
    }

    /// Generate markdown summary
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# Balance Analysis Report\n\n");

        md.push_str("## Win Rates\n\n");
        md.push_str("| Faction | Win Rate |\n|---------|----------|\n");
        for (faction, rate) in &self.win_rates {
            md.push_str(&format!("| {} | {:.1}% |\n", faction, rate * 100.0));
        }

        if !self.outliers.is_empty() {
            md.push_str("\n## Issues Detected\n\n");
            for outlier in self.outliers_by_severity() {
                md.push_str(&format!(
                    "- **[{:?}]** {}/{}: {:.2} (expected {:.2}-{:.2})\n",
                    outlier.severity,
                    outlier.category,
                    outlier.metric,
                    outlier.value,
                    outlier.expected_range.0,
                    outlier.expected_range.1
                ));
                if !outlier.context.is_empty() {
                    md.push_str(&format!("  - {}\n", outlier.context));
                }
            }
        }

        if !self.suggestions.is_empty() {
            md.push_str("\n## Suggestions\n\n");
            for (i, suggestion) in self.suggestions_by_confidence().iter().enumerate() {
                md.push_str(&format!(
                    "{}. **{}**: {} → {} ({:+.1}%)\n",
                    i + 1,
                    suggestion.target,
                    suggestion.current,
                    suggestion.suggested,
                    suggestion.percent_change()
                ));
                md.push_str(&format!(
                    "   - Confidence: {:.0}%\n",
                    suggestion.confidence * 100.0
                ));
                md.push_str(&format!("   - Reasoning: {}\n", suggestion.reasoning));
            }
        }

        md.push_str(&format!(
            "\n---\n*Analyzed {} games*\n",
            self.games_analyzed
        ));
        md
    }
}

/// Analyze batch results and generate balance report
pub fn analyze_batch(results: &BatchResults) -> BalanceAnalysis {
    let mut analysis = BalanceAnalysis::new();
    analysis.games_analyzed = results.games.len() as u32;
    analysis.metadata.source_batch = results.config.scenario.clone();

    // Calculate win rates
    let mut wins: HashMap<String, u32> = HashMap::new();
    let mut total_decided = 0u32;

    for metrics in &results.games {
        if let Some(ref winner) = metrics.winner {
            *wins.entry(winner.clone()).or_insert(0) += 1;
            total_decided += 1;
        }
    }

    if total_decided > 0 {
        for (faction, count) in &wins {
            analysis
                .win_rates
                .insert(faction.clone(), *count as f64 / total_decided as f64);
        }
    }

    // Detect win rate imbalances
    for (faction, rate) in &analysis.win_rates {
        if *rate < 0.40 {
            analysis.outliers.push(
                BalanceOutlier::new("win_rate", faction, *rate, (0.45, 0.55), Severity::High)
                    .with_context(&format!("{} is significantly underperforming", faction)),
            );
        } else if *rate < 0.45 {
            analysis.outliers.push(
                BalanceOutlier::new("win_rate", faction, *rate, (0.45, 0.55), Severity::Medium)
                    .with_context(&format!("{} is slightly underperforming", faction)),
            );
        } else if *rate > 0.60 {
            analysis.outliers.push(
                BalanceOutlier::new("win_rate", faction, *rate, (0.45, 0.55), Severity::High)
                    .with_context(&format!("{} is significantly overperforming", faction)),
            );
        } else if *rate > 0.55 {
            analysis.outliers.push(
                BalanceOutlier::new("win_rate", faction, *rate, (0.45, 0.55), Severity::Medium)
                    .with_context(&format!("{} is slightly overperforming", faction)),
            );
        }
    }

    // Analyze game durations
    analyze_timing(&mut analysis, &results.games);

    // Generate suggestions based on outliers
    generate_suggestions(&mut analysis, results);

    analysis
}

/// Analyze timing patterns
fn analyze_timing(analysis: &mut BalanceAnalysis, games: &[GameMetrics]) {
    if games.is_empty() {
        return;
    }

    let avg_duration: f64 =
        games.iter().map(|g| g.duration_ticks as f64).sum::<f64>() / games.len() as f64;

    // Check for games ending too quickly (rushes too strong)
    let early_games = games.iter().filter(|g| g.duration_ticks < 10000).count();
    let early_rate = early_games as f64 / games.len() as f64;

    if early_rate > 0.3 {
        analysis.outliers.push(
            BalanceOutlier::new(
                "timing",
                "early_game_rate",
                early_rate,
                (0.0, 0.2),
                Severity::Medium,
            )
            .with_context("Too many games ending early - rush strategies may be too strong"),
        );
    }

    // Check for games running to time limit
    let late_games = games.iter().filter(|g| g.duration_ticks >= 35000).count();
    let late_rate = late_games as f64 / games.len() as f64;

    if late_rate > 0.2 {
        analysis.outliers.push(
            BalanceOutlier::new(
                "timing",
                "stalemate_rate",
                late_rate,
                (0.0, 0.1),
                Severity::Low,
            )
            .with_context("Higher than expected stalemate rate"),
        );
    }

    // Store average duration for reference
    if avg_duration < 12000.0 {
        analysis.outliers.push(
            BalanceOutlier::new(
                "timing",
                "avg_duration",
                avg_duration,
                (15000.0, 25000.0),
                Severity::Low,
            )
            .with_context("Average game duration is short"),
        );
    }
}

/// Generate balance suggestions from detected issues
fn generate_suggestions(analysis: &mut BalanceAnalysis, _results: &BatchResults) {
    // Suggest fixes for win rate imbalances
    for outlier in &analysis.outliers {
        if outlier.category == "win_rate" && outlier.severity >= Severity::Medium {
            let faction = &outlier.metric;
            let rate = outlier.value;

            if rate < 0.45 {
                // Underpowered faction - buff something
                analysis.suggestions.push(
                    BalanceSuggestion::new(
                        &format!("{}.base_unit.damage", faction),
                        10.0,
                        11.0,
                        &format!(
                            "{} has {:.1}% win rate. Consider buffing base damage.",
                            faction,
                            rate * 100.0
                        ),
                    )
                    .with_confidence(0.6)
                    .with_outlier(&format!("{}/{}", outlier.category, outlier.metric)),
                );
            } else if rate > 0.55 {
                // Overpowered faction - nerf something
                analysis.suggestions.push(
                    BalanceSuggestion::new(
                        &format!("{}.base_unit.damage", faction),
                        10.0,
                        9.0,
                        &format!(
                            "{} has {:.1}% win rate. Consider reducing base damage.",
                            faction,
                            rate * 100.0
                        ),
                    )
                    .with_confidence(0.6)
                    .with_outlier(&format!("{}/{}", outlier.category, outlier.metric)),
                );
            }
        }

        if outlier.category == "timing" && outlier.metric == "early_game_rate" {
            analysis.suggestions.push(
                BalanceSuggestion::new(
                    "all.infantry.build_time",
                    5.0,
                    6.0,
                    "Rush strategies are too effective. Slowing early unit production may help.",
                )
                .with_confidence(0.5)
                .with_outlier("timing/early_game_rate"),
            );
        }
    }
}

/// Compare two batch results to see if changes improved balance
pub fn compare_batches(before: &BatchResults, after: &BatchResults) -> ComparisonReport {
    let before_analysis = analyze_batch(before);
    let after_analysis = analyze_batch(after);

    let mut improvements = Vec::new();
    let mut regressions = Vec::new();

    // Compare win rates
    for (faction, before_rate) in &before_analysis.win_rates {
        if let Some(after_rate) = after_analysis.win_rates.get(faction) {
            let before_dist = (before_rate - 0.5).abs();
            let after_dist = (after_rate - 0.5).abs();

            if after_dist < before_dist - 0.02 {
                improvements.push(format!(
                    "{}: Win rate moved closer to 50% ({:.1}% -> {:.1}%)",
                    faction,
                    before_rate * 100.0,
                    after_rate * 100.0
                ));
            } else if after_dist > before_dist + 0.02 {
                regressions.push(format!(
                    "{}: Win rate moved further from 50% ({:.1}% -> {:.1}%)",
                    faction,
                    before_rate * 100.0,
                    after_rate * 100.0
                ));
            }
        }
    }

    // Compare outlier counts
    let before_issues = before_analysis.outliers.len();
    let after_issues = after_analysis.outliers.len();

    ComparisonReport {
        before_games: before.games.len() as u32,
        after_games: after.games.len() as u32,
        improvements,
        regressions,
        before_issue_count: before_issues as u32,
        after_issue_count: after_issues as u32,
        overall_improved: after_issues < before_issues,
    }
}

/// Report comparing two batch runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub before_games: u32,
    pub after_games: u32,
    pub improvements: Vec<String>,
    pub regressions: Vec<String>,
    pub before_issue_count: u32,
    pub after_issue_count: u32,
    pub overall_improved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_priority() {
        assert!(Severity::Critical.priority() > Severity::High.priority());
        assert!(Severity::High.priority() > Severity::Medium.priority());
        assert!(Severity::Medium.priority() > Severity::Low.priority());
    }

    #[test]
    fn test_balance_outlier() {
        let outlier =
            BalanceOutlier::new("win_rate", "faction_a", 0.65, (0.45, 0.55), Severity::High)
                .with_context("Overpowered");

        assert_eq!(outlier.category, "win_rate");
        assert_eq!(outlier.severity, Severity::High);
        assert!(outlier.context.contains("Overpowered"));
    }

    #[test]
    fn test_suggestion_percent_change() {
        let suggestion = BalanceSuggestion::new("damage", 10.0, 12.0, "Buff needed");
        assert!((suggestion.percent_change() - 20.0).abs() < 0.01);

        let nerf = BalanceSuggestion::new("damage", 10.0, 8.0, "Nerf needed");
        assert!((nerf.percent_change() - (-20.0)).abs() < 0.01);
    }

    #[test]
    fn test_analyze_batch() {
        use crate::batch::{BatchConfig, BatchResults};
        use crate::metrics::BatchSummary;

        let games: Vec<GameMetrics> = (0..100)
            .map(|i| GameMetrics {
                game_id: format!("game_{}", i),
                scenario: "test".to_string(),
                seed: i as u64,
                duration_ticks: 20000,
                winner: Some(if i < 55 { "faction_a" } else { "faction_b" }.to_string()),
                win_condition: "elimination".to_string(),
                factions: HashMap::new(),
                events: Vec::new(),
                final_state_hash: i as u64,
            })
            .collect();

        let results = BatchResults {
            config: BatchConfig::default(),
            games,
            summary: BatchSummary::default(),
            duration_seconds: 1.0,
            errors: Vec::new(),
        };

        let analysis = analyze_batch(&results);

        assert_eq!(analysis.games_analyzed, 100);
        assert!(analysis.win_rates.contains_key("faction_a"));
        assert!(analysis.win_rates.contains_key("faction_b"));

        // Faction A wins 55% - should trigger medium severity outlier
        let a_rate = analysis.win_rates["faction_a"];
        assert!((a_rate - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_markdown_output() {
        let mut analysis = BalanceAnalysis::new();
        analysis.win_rates.insert("continuity".to_string(), 0.58);
        analysis.win_rates.insert("collegium".to_string(), 0.42);
        analysis.games_analyzed = 100;

        analysis.outliers.push(BalanceOutlier::new(
            "win_rate",
            "continuity",
            0.58,
            (0.45, 0.55),
            Severity::Medium,
        ));

        let md = analysis.to_markdown();
        assert!(md.contains("Balance Analysis"));
        assert!(md.contains("continuity"));
        assert!(md.contains("58.0%"));
    }

    #[test]
    fn test_outliers_sorted_by_severity() {
        let mut analysis = BalanceAnalysis::new();
        analysis.outliers.push(BalanceOutlier::new(
            "a",
            "low",
            0.1,
            (0.0, 1.0),
            Severity::Low,
        ));
        analysis.outliers.push(BalanceOutlier::new(
            "b",
            "high",
            0.9,
            (0.0, 1.0),
            Severity::High,
        ));
        analysis.outliers.push(BalanceOutlier::new(
            "c",
            "med",
            0.5,
            (0.0, 1.0),
            Severity::Medium,
        ));

        let sorted = analysis.outliers_by_severity();
        assert_eq!(sorted[0].severity, Severity::High);
        assert_eq!(sorted[1].severity, Severity::Medium);
        assert_eq!(sorted[2].severity, Severity::Low);
    }
}
