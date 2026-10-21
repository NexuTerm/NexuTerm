use std::collections::HashMap;
pub const SCRIPT: &str = "printf '\\n@cpu\\n'; head -n1 /proc/stat; printf '\\n@mem\\n'; cat /proc/meminfo; printf '\\n@net\\n'; cat /proc/net/dev; printf '\\n@up\\n'; cat /proc/uptime; printf '\\n@disk\\n'; LC_ALL=C df -Pk /";
#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub cpu_total: Option<u64>,
    pub cpu_idle: Option<u64>,
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    pub received: Option<u64>,
    pub sent: Option<u64>,
    pub uptime: Option<f64>,
    pub disk_percent: Option<u32>,
}
pub fn parse(text: &str) -> Sample {
    let mut sections = HashMap::<&str, String>::new();
    let mut section = "";
    for line in text.lines() {
        if let Some(s) = line.strip_prefix('@') {
            section = s;
        } else {
            sections
                .entry(section)
                .or_default()
                .push_str(&format!("{line}\n"));
        }
    }
    let mut s = Sample::default();
    if let Some(cpu) = sections.get("cpu") {
        let values: Option<Vec<u64>> = cpu
            .split_whitespace()
            .skip(1)
            .take(8)
            .map(|v| v.parse().ok())
            .collect();
        if let Some(v) = values.filter(|v| v.len() >= 4) {
            s.cpu_total = Some(v.iter().sum());
            s.cpu_idle = Some(v[3] + v.get(4).copied().unwrap_or(0));
        }
    }
    let mem: HashMap<_, _> = sections
        .get("mem")
        .map(|m| {
            m.lines()
                .filter_map(|l| {
                    let (k, v) = l.split_once(':')?;
                    Some((k, v.split_whitespace().next()?.parse::<u64>().ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    s.memory_total = mem.get("MemTotal").copied();
    s.memory_used = s
        .memory_total
        .zip(mem.get("MemAvailable").copied())
        .map(|(t, a)| t.saturating_sub(a));
    if let Some(net) = sections.get("net") {
        let mut rx = 0;
        let mut tx = 0;
        let mut found = false;
        for line in net.lines() {
            if let Some((iface, data)) = line.split_once(':') {
                if iface.trim() == "lo" {
                    continue;
                }
                let fields: Vec<_> = data.split_whitespace().collect();
                if let (Some(r), Some(t)) = (
                    fields.first().and_then(|s| s.parse::<u64>().ok()),
                    fields.get(8).and_then(|s| s.parse::<u64>().ok()),
                ) {
                    rx += r;
                    tx += t;
                    found = true;
                }
            }
        }
        if found {
            s.received = Some(rx);
            s.sent = Some(tx);
        }
    }
    s.uptime = sections
        .get("up")
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok());
    s.disk_percent = sections
        .get("disk")
        .and_then(|s| s.lines().nth(1))
        .and_then(|s| s.split_whitespace().nth(4))
        .and_then(|s| s.trim_end_matches('%').parse().ok());
    s
}
pub fn cpu_percent(before: &Sample, after: &Sample) -> Option<f64> {
    let total = after.cpu_total?.checked_sub(before.cpu_total?)?;
    let idle = after.cpu_idle?.checked_sub(before.cpu_idle?)?;
    (total > 0).then(|| 100.0 * (total.saturating_sub(idle)) as f64 / total as f64)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_is_not_zero() {
        let s = parse("@mem\nMemTotal: 100 kB\n@up\n12.5 3\n");
        assert_eq!(s.memory_used, None);
        assert_eq!(s.uptime, Some(12.5));
        assert!(s.cpu_total.is_none());
    }
}
