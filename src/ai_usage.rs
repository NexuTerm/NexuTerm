//! Provider-reported usage only: quota windows and context are distinct quantities.
use serde_json::Value;
#[derive(Debug, PartialEq)]
pub struct Window {
    pub label: String,
    pub remaining: f64,
    pub resets_at: Option<i64>,
}
pub fn windows(value: &Value, model: &str) -> Vec<Window> {
    let buckets = &value["rateLimitsByLimitId"];
    let snapshot = buckets
        .get(model)
        .or_else(|| buckets.get("codex"))
        .or_else(|| value.get("rateLimits"))
        .unwrap_or(value);
    ["primary", "secondary"]
        .into_iter()
        .filter_map(|name| {
            let window = &snapshot[name];
            let used = window["usedPercent"]
                .as_f64()
                .filter(|v| v.is_finite() && *v >= 0.0)?;
            let label = match window["windowDurationMins"].as_u64() {
                Some(300) => "5 h".into(),
                Some(10080) => "Semanal".into(),
                Some(n) if n % 1440 == 0 && n > 0 => format!("{} d", n / 1440),
                Some(n) if n % 60 == 0 && n > 0 => format!("{} h", n / 60),
                Some(n) if n > 0 => format!("{n} min"),
                _ => {
                    if name == "primary" {
                        "Principal".into()
                    } else {
                        "Secundaria".into()
                    }
                }
            };
            Some(Window {
                label,
                remaining: (100.0 - used).clamp(0.0, 100.0),
                resets_at: window["resetsAt"].as_i64(),
            })
        })
        .collect()
}
pub fn context_remaining(used: u64, size: u64) -> Option<f64> {
    (size > 0).then(|| 100.0 * (1.0 - (used as f64 / size as f64).min(1.0)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn quotas_use_real_windows_and_never_invent_missing_percentages() {
        let value = json!({"rateLimits":{"primary":{"usedPercent":23.5,"windowDurationMins":300},"secondary":{"usedPercent":100,"windowDurationMins":10080}}});
        let w = windows(&value, "model");
        assert_eq!((&w[0].label, w[0].remaining), (&"5 h".to_owned(), 76.5));
        assert_eq!((&w[1].label, w[1].remaining), (&"Semanal".to_owned(), 0.0));
        assert!(windows(&json!({}), "model").is_empty());
        assert!(windows(&json!({"primary":{"usedPercent":-1}}), "model").is_empty());
        assert_eq!(
            windows(
                &json!({"primary":{"usedPercent":110,"windowDurationMins":15}}),
                "model"
            )[0]
            .remaining,
            0.0
        );
        assert_eq!(
            windows(
                &json!({"primary":{"usedPercent":1,"windowDurationMins":15}}),
                "model"
            )[0]
            .label,
            "15 min"
        );
    }
    #[test]
    fn model_bucket_precedes_shared_bucket_and_context_is_separate() {
        let v = json!({"rateLimitsByLimitId":{"special":{"primary":{"usedPercent":90}},"codex":{"primary":{"usedPercent":10}}}});
        assert_eq!(windows(&v, "special")[0].remaining, 10.0);
        assert_eq!(windows(&v, "other")[0].remaining, 90.0);
        assert_eq!(context_remaining(25, 100), Some(75.0));
        assert_eq!(context_remaining(120, 100), Some(0.0));
        assert_eq!(context_remaining(1, 0), None);
    }
}
