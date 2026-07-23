use crate::types::CostInfo;

/// Format the session's total cost as a currency string (e.g. `"$0.05"`).
///
/// Reads `total_cost_usd` from the cost info. Returns `None` when no cost data
/// is present so the segment can be omitted. A present cost of `0.0` still
/// renders as `"$0.00"`.
pub fn format_cost(cost: &Option<CostInfo>) -> Option<String> {
    let total = cost.as_ref()?.total_cost_usd?;
    Some(format!("${:.2}", total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_cost_returns_none_for_none() {
        assert_eq!(format_cost(&None), None);
    }

    #[test]
    fn format_cost_returns_none_when_total_cost_usd_missing() {
        let cost = Some(CostInfo {
            total_cost_usd: None,
            ..Default::default()
        });
        assert_eq!(format_cost(&cost), None);
    }

    #[test]
    fn format_cost_formats_with_two_decimals() {
        let cost = Some(CostInfo {
            total_cost_usd: Some(0.05),
            ..Default::default()
        });
        assert_eq!(format_cost(&cost), Some("$0.05".to_string()));
    }

    #[test]
    fn format_cost_renders_zero_cost() {
        let cost = Some(CostInfo {
            total_cost_usd: Some(0.0),
            ..Default::default()
        });
        assert_eq!(format_cost(&cost), Some("$0.00".to_string()));
    }

    #[test]
    fn format_cost_rounds_to_two_decimals() {
        let cost = Some(CostInfo {
            total_cost_usd: Some(1.2345),
            ..Default::default()
        });
        assert_eq!(format_cost(&cost), Some("$1.23".to_string()));
    }

    #[test]
    fn format_cost_formats_larger_amount() {
        let cost = Some(CostInfo {
            total_cost_usd: Some(12.5),
            ..Default::default()
        });
        assert_eq!(format_cost(&cost), Some("$12.50".to_string()));
    }
}
