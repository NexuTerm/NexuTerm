//! Bounded line differences for manual three-version review, never an implicit merge.
use std::ops::Range;
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub base: Range<usize>,
    pub edited: Range<usize>,
}
#[derive(Debug)]
pub struct Diff {
    pub changes: Vec<Change>,
    pub detailed: bool,
}
const CELLS: usize = 2_000_000;
pub fn compare(base: &str, edited: &str) -> Diff {
    let lines = |s: &str| {
        s.bytes().filter(|b| *b == b'\n').count() + usize::from(!s.is_empty() && !s.ends_with('\n'))
    };
    let (an, bn) = (lines(base), lines(edited));
    if an.max(bn) > 200_000 {
        return Diff {
            changes: if base == edited {
                vec![]
            } else {
                vec![Change {
                    base: 0..an,
                    edited: 0..bn,
                }]
            },
            detailed: false,
        };
    }
    let a: Vec<_> = base.split_inclusive('\n').collect();
    let b: Vec<_> = edited.split_inclusive('\n').collect();
    let mut prefix = 0;
    while prefix < a.len().min(b.len()) && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut ae = a.len();
    let mut be = b.len();
    while ae > prefix && be > prefix && a[ae - 1] == b[be - 1] {
        ae -= 1;
        be -= 1;
    }
    if ae == prefix && be == prefix {
        return Diff {
            changes: vec![],
            detailed: true,
        };
    }
    let n = ae - prefix;
    let m = be - prefix;
    if (n + 1).checked_mul(m + 1).is_none_or(|cells| cells > CELLS) {
        return Diff {
            changes: vec![Change {
                base: prefix..ae,
                edited: prefix..be,
            }],
            detailed: false,
        };
    }
    // Intern lines once: the quadratic part compares integers, not long common strings.
    let mut intern = std::collections::HashMap::new();
    let mut next = 0u32;
    let ids: Vec<u32> = a[prefix..ae]
        .iter()
        .chain(b[prefix..be].iter())
        .map(|line| {
            *intern.entry(*line).or_insert_with(|| {
                let id = next;
                next += 1;
                id
            })
        })
        .collect();
    let (ai, bi) = ids.split_at(n);
    let width = m + 1;
    let mut lengths = vec![0u32; (n + 1) * width];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lengths[i * width + j] = if ai[i] == bi[j] {
                lengths[(i + 1) * width + j + 1] + 1
            } else {
                lengths[(i + 1) * width + j].max(lengths[i * width + j + 1])
            };
        }
    }
    let mut changes = Vec::new();
    let (mut i, mut j) = (0, 0);
    let mut start = None;
    while i < n || j < m {
        if i < n && j < m && ai[i] == bi[j] {
            if let Some((x, y)) = start.take() {
                changes.push(Change {
                    base: prefix + x..prefix + i,
                    edited: prefix + y..prefix + j,
                });
            }
            i += 1;
            j += 1;
        } else {
            start.get_or_insert((i, j));
            if i < n && (j == m || lengths[(i + 1) * width + j] >= lengths[i * width + j + 1]) {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    if let Some((x, y)) = start {
        changes.push(Change {
            base: prefix + x..prefix + i,
            edited: prefix + y..prefix + j,
        });
    }
    Diff {
        changes,
        detailed: true,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separated_changes_preserve_unicode_and_line_endings() {
        let diff = compare("á\r\nb\nc\nd\ne", "á\r\nB\nc\nd\nE\n");
        assert_eq!(
            diff.changes,
            vec![
                Change {
                    base: 1..2,
                    edited: 1..2
                },
                Change {
                    base: 4..5,
                    edited: 4..5
                }
            ]
        );
        assert!(diff.detailed);
        assert!(compare("", "").changes.is_empty());
        assert_eq!(
            compare("a\n", "a\nx").changes[0],
            Change {
                base: 1..1,
                edited: 1..2
            }
        );
        assert_eq!(
            compare("a\nx", "a\n").changes[0],
            Change {
                base: 1..2,
                edited: 1..1
            }
        );
    }
    #[test]
    fn expensive_comparisons_fall_back_without_quadratic_memory() {
        let a = "a\n".repeat(2000);
        let b = "b\n".repeat(2000);
        let diff = compare(&a, &b);
        assert!(!diff.detailed);
        assert_eq!(diff.changes[0].base, 0..2000);
        let a = "same\n".repeat(100_000) + "old\n";
        let b = "same\n".repeat(100_000) + "new\n";
        let diff = compare(&a, &b);
        assert!(diff.detailed);
        assert_eq!(diff.changes[0].base, 100_000..100_001);
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    #[test]
    fn edits_reconstruct_every_variant_exactly() {
        let cases = [
            "",
            "a",
            "a\n",
            "b\n",
            "á\nx\n",
            "a\nb\nc",
            "a\nb\n",
            "a\na\na\n",
            "a\r\nb\r\n",
        ];
        for base in cases {
            for edited in cases {
                let a: Vec<_> = base.split_inclusive('\n').collect();
                let b: Vec<_> = edited.split_inclusive('\n').collect();
                let mut result = String::new();
                let mut offset = 0;
                for change in compare(base, edited).changes {
                    result.push_str(&a[offset..change.base.start].concat());
                    result.push_str(&b[change.edited].concat());
                    offset = change.base.end;
                }
                result.push_str(&a[offset..].concat());
                assert_eq!(result, edited);
            }
        }
        assert!(!compare(&"\n".repeat(200_001), "x").detailed);
    }
}
