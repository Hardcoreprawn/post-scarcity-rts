//! Visual quality review system.
//!
//! Provides structures for automated visual quality testing and
//! review report generation.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Result of a silhouette distinction test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilhouetteResult {
    /// Number of distinct unit shapes detected
    pub unit_count_detected: u32,
    /// Expected number based on game state
    pub unit_count_expected: u32,
    /// Percentage of units with overlapping silhouettes
    pub overlap_percentage: f64,
    /// Test passed
    pub pass: bool,
    /// Detailed notes
    pub notes: String,
}

impl SilhouetteResult {
    /// Check if test passes
    pub fn evaluate(detected: u32, expected: u32, overlap: f64) -> Self {
        let count_ok = (detected as f64 - expected as f64).abs() / expected.max(1) as f64 <= 0.1;
        let overlap_ok = overlap < 0.1;
        Self {
            unit_count_detected: detected,
            unit_count_expected: expected,
            overlap_percentage: overlap,
            pass: count_ok && overlap_ok,
            notes: if !count_ok {
                format!(
                    "Unit count mismatch: {} detected vs {} expected",
                    detected, expected
                )
            } else if !overlap_ok {
                format!("High overlap: {:.1}%", overlap * 100.0)
            } else {
                "All checks passed".to_string()
            },
        }
    }
}

/// Result of a color distinction test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDistinctionResult {
    /// Faction colors (LAB color space)
    pub faction_colors: Vec<(String, [f64; 3])>,
    /// Minimum color distance between factions
    pub min_distance: f64,
    /// Test passed
    pub pass: bool,
    /// Detailed notes
    pub notes: String,
}

impl ColorDistinctionResult {
    /// Threshold for distinguishable colors in LAB space
    pub const DISTANCE_THRESHOLD: f64 = 50.0;

    /// Evaluate color distinction
    pub fn evaluate(colors: Vec<(String, [f64; 3])>) -> Self {
        let mut min_dist = f64::MAX;

        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                let dist = lab_distance(&colors[i].1, &colors[j].1);
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }

        Self {
            faction_colors: colors,
            min_distance: min_dist,
            pass: min_dist > Self::DISTANCE_THRESHOLD,
            notes: if min_dist > Self::DISTANCE_THRESHOLD {
                format!("Colors well distinguished (distance: {:.1})", min_dist)
            } else {
                format!(
                    "Colors too similar (distance: {:.1}, need > {:.1})",
                    min_dist,
                    Self::DISTANCE_THRESHOLD
                )
            },
        }
    }
}

/// Calculate LAB color space distance
fn lab_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dl = a[0] - b[0];
    let da = a[1] - b[1];
    let db = a[2] - b[2];
    (dl * dl + da * da + db * db).sqrt()
}

/// Result of a brightness test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessResult {
    /// Average brightness per faction (0.0 - 1.0)
    pub faction_brightness: Vec<(String, f64)>,
    /// Test passed
    pub pass: bool,
    /// Detailed notes
    pub notes: String,
}

impl BrightnessResult {
    /// Minimum acceptable brightness
    pub const MIN_BRIGHTNESS: f64 = 0.2;
    /// Maximum acceptable brightness
    pub const MAX_BRIGHTNESS: f64 = 0.8;

    /// Evaluate brightness levels
    pub fn evaluate(brightness: Vec<(String, f64)>) -> Self {
        let all_ok = brightness
            .iter()
            .all(|(_, b)| *b >= Self::MIN_BRIGHTNESS && *b <= Self::MAX_BRIGHTNESS);

        let notes = brightness
            .iter()
            .filter(|(_, b)| *b < Self::MIN_BRIGHTNESS || *b > Self::MAX_BRIGHTNESS)
            .map(|(name, b)| {
                if *b < Self::MIN_BRIGHTNESS {
                    format!("{} is too dark ({:.2})", name, b)
                } else {
                    format!("{} is too bright ({:.2})", name, b)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");

        Self {
            faction_brightness: brightness,
            pass: all_ok,
            notes: if all_ok {
                "Brightness levels acceptable".to_string()
            } else {
                notes
            },
        }
    }
}

/// Combined visual quality report for a screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualQualityReport {
    /// Screenshot filename
    pub screenshot: String,
    /// Game tick when captured
    pub tick: u64,
    /// Trigger that caused capture
    pub trigger: String,
    /// Silhouette test result
    pub silhouette: Option<SilhouetteResult>,
    /// Color distinction result
    pub color_distinction: Option<ColorDistinctionResult>,
    /// Brightness result
    pub brightness: Option<BrightnessResult>,
    /// Overall pass/fail
    pub overall_pass: bool,
    /// Manual review notes
    pub manual_notes: Vec<String>,
    /// Quality score (0-100)
    pub quality_score: u32,
}

impl VisualQualityReport {
    /// Create new report
    pub fn new(screenshot: &str, tick: u64, trigger: &str) -> Self {
        Self {
            screenshot: screenshot.to_string(),
            tick,
            trigger: trigger.to_string(),
            silhouette: None,
            color_distinction: None,
            brightness: None,
            overall_pass: true,
            manual_notes: Vec::new(),
            quality_score: 100,
        }
    }

    /// Calculate overall pass based on individual tests
    pub fn calculate_overall(&mut self) {
        let mut score = 100u32;
        let mut pass = true;

        if let Some(ref s) = self.silhouette {
            if !s.pass {
                score = score.saturating_sub(30);
                pass = false;
            }
        }

        if let Some(ref c) = self.color_distinction {
            if !c.pass {
                score = score.saturating_sub(40);
                pass = false;
            }
        }

        if let Some(ref b) = self.brightness {
            if !b.pass {
                score = score.saturating_sub(20);
                pass = false;
            }
        }

        self.overall_pass = pass;
        self.quality_score = score;
    }

    /// Add manual review note
    pub fn add_note(&mut self, note: &str) {
        self.manual_notes.push(note.to_string());
    }
}

/// Complete visual review for a batch of games
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchVisualReview {
    /// Batch ID
    pub batch_id: String,
    /// Individual screenshot reports
    pub reports: Vec<VisualQualityReport>,
    /// Average quality score
    pub average_score: f64,
    /// Pass rate
    pub pass_rate: f64,
    /// Common issues found
    pub common_issues: Vec<String>,
}

impl BatchVisualReview {
    /// Create new batch review
    pub fn new(batch_id: &str) -> Self {
        Self {
            batch_id: batch_id.to_string(),
            reports: Vec::new(),
            average_score: 0.0,
            pass_rate: 0.0,
            common_issues: Vec::new(),
        }
    }

    /// Add report to batch
    pub fn add_report(&mut self, report: VisualQualityReport) {
        self.reports.push(report);
        self.recalculate_stats();
    }

    /// Recalculate aggregate statistics
    pub fn recalculate_stats(&mut self) {
        if self.reports.is_empty() {
            return;
        }

        let total_score: u32 = self.reports.iter().map(|r| r.quality_score).sum();
        self.average_score = total_score as f64 / self.reports.len() as f64;

        let pass_count = self.reports.iter().filter(|r| r.overall_pass).count();
        self.pass_rate = pass_count as f64 / self.reports.len() as f64;
    }

    /// Save review to JSON
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load review from JSON
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(std::io::Error::other)
    }

    /// Generate HTML report
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Visual Quality Review</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: sans-serif; margin: 20px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #4CAF50; color: white; }\n");
        html.push_str(".pass { color: green; }\n");
        html.push_str(".fail { color: red; }\n");
        html.push_str(".score-high { background-color: #c8e6c9; }\n");
        html.push_str(".score-mid { background-color: #fff9c4; }\n");
        html.push_str(".score-low { background-color: #ffcdd2; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str(&format!("<h1>Visual Review: {}</h1>\n", self.batch_id));
        html.push_str(&format!(
            "<p>Average Score: <strong>{:.1}</strong> | Pass Rate: <strong>{:.1}%</strong></p>\n",
            self.average_score,
            self.pass_rate * 100.0
        ));

        html.push_str("<table>\n<tr><th>Screenshot</th><th>Tick</th><th>Trigger</th>");
        html.push_str("<th>Score</th><th>Pass</th><th>Notes</th></tr>\n");

        for report in &self.reports {
            let score_class = if report.quality_score >= 80 {
                "score-high"
            } else if report.quality_score >= 50 {
                "score-mid"
            } else {
                "score-low"
            };
            let pass_class = if report.overall_pass { "pass" } else { "fail" };
            let pass_text = if report.overall_pass { "✓" } else { "✗" };

            html.push_str(&format!(
                "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td>",
                score_class, report.screenshot, report.tick, report.trigger
            ));
            html.push_str(&format!(
                "<td>{}</td><td class=\"{}\">{}</td>",
                report.quality_score, pass_class, pass_text
            ));
            html.push_str(&format!(
                "<td>{}</td></tr>\n",
                report.manual_notes.join("; ")
            ));
        }

        html.push_str("</table>\n");

        if !self.common_issues.is_empty() {
            html.push_str("<h2>Common Issues</h2>\n<ul>\n");
            for issue in &self.common_issues {
                html.push_str(&format!("<li>{}</li>\n", issue));
            }
            html.push_str("</ul>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silhouette_result_pass() {
        let result = SilhouetteResult::evaluate(10, 10, 0.05);
        assert!(result.pass);
    }

    #[test]
    fn test_silhouette_result_fail_count() {
        let result = SilhouetteResult::evaluate(5, 10, 0.05);
        assert!(!result.pass);
        assert!(result.notes.contains("mismatch"));
    }

    #[test]
    fn test_silhouette_result_fail_overlap() {
        let result = SilhouetteResult::evaluate(10, 10, 0.25);
        assert!(!result.pass);
        assert!(result.notes.contains("overlap"));
    }

    #[test]
    fn test_color_distinction_pass() {
        let colors = vec![
            ("continuity".to_string(), [50.0, -20.0, -40.0]), // Blue
            ("collegium".to_string(), [70.0, 10.0, 60.0]),    // Gold
        ];
        let result = ColorDistinctionResult::evaluate(colors);
        assert!(result.pass);
        assert!(result.min_distance > 50.0);
    }

    #[test]
    fn test_color_distinction_fail() {
        let colors = vec![
            ("faction_a".to_string(), [50.0, 0.0, 0.0]),
            ("faction_b".to_string(), [55.0, 5.0, 5.0]),
        ];
        let result = ColorDistinctionResult::evaluate(colors);
        assert!(!result.pass);
    }

    #[test]
    fn test_brightness_pass() {
        let brightness = vec![
            ("continuity".to_string(), 0.5),
            ("collegium".to_string(), 0.6),
        ];
        let result = BrightnessResult::evaluate(brightness);
        assert!(result.pass);
    }

    #[test]
    fn test_brightness_fail_dark() {
        let brightness = vec![("dark_faction".to_string(), 0.1)];
        let result = BrightnessResult::evaluate(brightness);
        assert!(!result.pass);
        assert!(result.notes.contains("too dark"));
    }

    #[test]
    fn test_visual_quality_report() {
        let mut report = VisualQualityReport::new("test.png", 1000, "first_contact");
        report.silhouette = Some(SilhouetteResult::evaluate(10, 10, 0.05));
        report.color_distinction = Some(ColorDistinctionResult::evaluate(vec![
            ("a".to_string(), [50.0, -20.0, -40.0]),
            ("b".to_string(), [70.0, 10.0, 60.0]),
        ]));
        report.calculate_overall();

        assert!(report.overall_pass);
        assert_eq!(report.quality_score, 100);
    }

    #[test]
    fn test_batch_review_stats() {
        let mut review = BatchVisualReview::new("test_batch");

        let mut r1 = VisualQualityReport::new("a.png", 100, "test");
        r1.quality_score = 80;
        r1.overall_pass = true;
        review.add_report(r1);

        let mut r2 = VisualQualityReport::new("b.png", 200, "test");
        r2.quality_score = 60;
        r2.overall_pass = false;
        review.add_report(r2);

        assert_eq!(review.average_score, 70.0);
        assert_eq!(review.pass_rate, 0.5);
    }

    #[test]
    fn test_html_generation() {
        let mut review = BatchVisualReview::new("html_test");
        let report = VisualQualityReport::new("test.png", 500, "major_battle");
        review.add_report(report);

        let html = review.to_html();
        assert!(html.contains("html_test"));
        assert!(html.contains("test.png"));
        assert!(html.contains("major_battle"));
    }
}
