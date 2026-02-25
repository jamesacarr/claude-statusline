use crate::types::ContextWindow;

/// Compute context usage from optional remaining and used percentages.
///
/// Prefers `used_percentage` if available. Falls back to deriving from
/// `remaining_percentage`. The scaled value maps raw usage to an 80% ceiling
/// so the bar graph warns early.
pub fn compute_usage(
    remaining_percentage: Option<f64>,
    used_percentage: Option<f64>,
) -> Option<u32> {
    let raw_used_f = if let Some(used) = used_percentage {
        used
    } else if let Some(remaining) = remaining_percentage {
        100.0 - remaining
    } else {
        return None;
    };

    let raw_used = raw_used_f.round().clamp(0.0, 100.0) as u32;
    Some(raw_used)
}

/// Format context window token usage from ContextWindow.
///
/// Computes token count from `current_usage` input tokens (matching how
/// `used_percentage` is calculated: `input_tokens + cache_creation_input_tokens
/// + cache_read_input_tokens`). Falls back to deriving from
/// `used_percentage * context_window_size` if `current_usage` is unavailable.
/// Returns `"0"` if the context window is None or usage is zero.
pub fn format_token_count(ctx: &Option<ContextWindow>) -> String {
    let Some(cw) = ctx else {
        return "0".to_string();
    };

    let total = if let Some(ref usage) = cw.current_usage {
        usage.input_tokens.unwrap_or(0)
            + usage.cache_creation_input_tokens.unwrap_or(0)
            + usage.cache_read_input_tokens.unwrap_or(0)
    } else if let (Some(pct), Some(size)) = (cw.used_percentage, cw.context_window_size) {
        (pct / 100.0 * size as f64).round() as u64
    } else {
        0
    };

    if total == 0 {
        "0".to_string()
    } else if total >= 1000 {
        format!("{:.1}k", total as f64 / 1000.0)
    } else {
        total.to_string()
    }
}

/// Render a 10-segment bar graph with color thresholds.
///
/// `raw_used` drives the bar fill, color threshold selection (50/65/80) and percentage display.
/// `token_display` is appended in parentheses after the percentage.
pub fn render_bar(raw_used: u32, token_display: &str, no_color: bool) -> String {
    let filled = (raw_used / 10) as usize;
    let empty = 10_usize.saturating_sub(filled);
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);

    if no_color {
        return format!(" {} {}% ({})", bar, raw_used, token_display);
    }

    let (color, skull) = if raw_used >= 80 {
        ("\x1b[5;31m", "\u{1F480} ")
    } else if raw_used >= 65 {
        ("\x1b[38;5;208m", "")
    } else if raw_used >= 50 {
        ("\x1b[33m", "")
    } else {
        ("\x1b[32m", "")
    };

    format!(
        " {}{}{} {}% ({})\x1b[0m",
        color, skull, bar, raw_used, token_display
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextWindow, CurrentUsage};

    // --- compute_usage tests ---

    #[test]
    fn compute_usage_prefers_used_percentage_over_remaining() {
        assert_eq!(compute_usage(Some(92.0), Some(8.0)), Some(8));
    }

    #[test]
    fn compute_usage_at_full_returns_hundred() {
        assert_eq!(compute_usage(Some(0.0), Some(100.0)), Some(100));
    }

    #[test]
    fn compute_usage_at_zero_returns_zero() {
        assert_eq!(compute_usage(Some(100.0), Some(0.0)), Some(0));
    }

    #[test]
    fn compute_usage_at_eighty_percent() {
        assert_eq!(compute_usage(Some(20.0), Some(80.0)), Some(80));
    }

    #[test]
    fn compute_usage_returns_none_when_both_are_none() {
        assert_eq!(compute_usage(None, None), None);
    }

    #[test]
    fn compute_usage_falls_back_to_remaining_when_used_is_none() {
        assert_eq!(compute_usage(Some(92.0), None), Some(8));
    }

    #[test]
    fn compute_usage_uses_used_percentage_when_remaining_is_none() {
        assert_eq!(compute_usage(None, Some(8.0)), Some(8));
    }

    #[test]
    fn compute_usage_clamps_negative_remaining_to_max() {
        // remaining = -5 -> raw_used = 100 - (-5) = 105, clamped to 100
        assert_eq!(compute_usage(Some(-5.0), None), Some(100));
    }

    #[test]
    fn compute_usage_clamps_large_remaining_to_zero() {
        // remaining = 150 -> raw_used = 100 - 150 = -50, clamped to 0
        assert_eq!(compute_usage(Some(150.0), None), Some(0));
    }

    // --- format_token_count tests ---

    #[test]
    fn format_token_count_returns_zero_for_none() {
        assert_eq!(format_token_count(&None), "0");
    }

    #[test]
    fn format_token_count_uses_current_usage_input_tokens_ignoring_cumulative_totals() {
        let ctx = Some(ContextWindow {
            total_input_tokens: Some(15234),
            total_output_tokens: Some(4521),
            current_usage: Some(CurrentUsage {
                input_tokens: Some(8500),
                output_tokens: Some(1200),
                cache_creation_input_tokens: Some(5000),
                cache_read_input_tokens: Some(2000),
            }),
            ..Default::default()
        });
        // input_tokens + cache_creation + cache_read = 8500 + 5000 + 2000 = 15500
        // NOT total_input + total_output (19755), NOT including output_tokens (1200)
        assert_eq!(format_token_count(&ctx), "15.5k");
    }

    #[test]
    fn format_token_count_displays_raw_number_below_thousand() {
        let ctx = Some(ContextWindow {
            current_usage: Some(CurrentUsage {
                input_tokens: Some(500),
                cache_creation_input_tokens: Some(200),
                cache_read_input_tokens: Some(142),
                ..Default::default()
            }),
            ..Default::default()
        });
        // 500 + 200 + 142 = 842
        assert_eq!(format_token_count(&ctx), "842");
    }

    #[test]
    fn format_token_count_at_exactly_one_thousand() {
        let ctx = Some(ContextWindow {
            current_usage: Some(CurrentUsage {
                input_tokens: Some(1000),
                ..Default::default()
            }),
            ..Default::default()
        });
        assert_eq!(format_token_count(&ctx), "1.0k");
    }

    #[test]
    fn format_token_count_returns_zero_for_zero_current_usage() {
        let ctx = Some(ContextWindow {
            current_usage: Some(CurrentUsage {
                input_tokens: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        });
        assert_eq!(format_token_count(&ctx), "0");
    }

    #[test]
    fn format_token_count_falls_back_to_percentage_when_no_current_usage() {
        let ctx = Some(ContextWindow {
            total_input_tokens: Some(50000),
            total_output_tokens: Some(10000),
            used_percentage: Some(11.0),
            context_window_size: Some(200000),
            current_usage: None,
            ..Default::default()
        });
        // 11% of 200000 = 22000, NOT 50000 + 10000
        assert_eq!(format_token_count(&ctx), "22.0k");
    }

    #[test]
    fn format_token_count_returns_zero_when_no_current_usage_or_percentage() {
        let ctx = Some(ContextWindow {
            total_input_tokens: Some(50000),
            total_output_tokens: Some(10000),
            current_usage: None,
            used_percentage: None,
            context_window_size: Some(200000),
            ..Default::default()
        });
        assert_eq!(format_token_count(&ctx), "0");
    }

    // --- render_bar tests ---

    #[test]
    fn render_bar_green_below_fifty() {
        let result = render_bar(40, "5.0k", false);
        assert!(result.contains("\x1b[32m"), "expected green ANSI code");
        assert!(result.contains("40%"), "expected percentage 40%");
        assert!(result.contains("(5.0k)"), "expected token display");
        // 40/10 = 4 filled blocks
        assert!(
            result.contains(
                "\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}"
            ),
            "expected 4 filled + 6 empty blocks"
        );
    }

    #[test]
    fn render_bar_yellow_at_fifty() {
        let result = render_bar(56, "8.2k", false);
        assert!(result.contains("\x1b[33m"), "expected yellow ANSI code");
        assert!(result.contains("56%"), "expected percentage 56%");
    }

    #[test]
    fn render_bar_orange_at_sixty_five() {
        let result = render_bar(72, "15.3k", false);
        assert!(
            result.contains("\x1b[38;5;208m"),
            "expected orange 256-color ANSI code"
        );
        assert!(result.contains("72%"), "expected percentage 72%");
    }

    #[test]
    fn render_bar_blinking_red_with_skull_at_eighty() {
        let result = render_bar(80, "20.0k", false);
        assert!(
            result.contains("\x1b[5;31m"),
            "expected blinking red ANSI code"
        );
        assert!(result.contains("80%"), "expected percentage 80%");
        assert!(result.contains("\u{1F480}"), "expected skull emoji");
    }

    #[test]
    fn render_bar_zero_percent_shows_all_empty_blocks() {
        let result = render_bar(0, "0", false);
        assert!(result.contains("\x1b[32m"), "expected green ANSI code");
        assert!(result.contains("0%"), "expected percentage 0%");
        assert!(
            result.contains(
                "\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}"
            ),
            "expected 10 empty blocks"
        );
    }

    #[test]
    fn render_bar_no_color_omits_ansi_sequences() {
        let result = render_bar(40, "5.0k", true);
        assert!(!result.contains("\x1b["), "expected no ANSI sequences");
        assert!(result.contains("40%"), "expected percentage 40%");
        assert!(result.contains("(5.0k)"), "expected token display");
        assert!(
            !result.contains("\u{1F480}"),
            "expected no skull emoji in no_color mode"
        );
    }
}
