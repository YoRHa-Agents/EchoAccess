use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MergeResult {
    Clean(String),
    Conflict {
        merged_with_markers: String,
        conflict_count: usize,
    },
}

pub fn three_way_merge(base: &str, ours: &str, theirs: &str) -> Result<MergeResult> {
    let patch_ours = diffy::create_patch(base, ours);
    let patch_theirs = diffy::create_patch(base, theirs);

    if patch_ours.hunks().is_empty() {
        return Ok(MergeResult::Clean(theirs.to_string()));
    }
    if patch_theirs.hunks().is_empty() {
        return Ok(MergeResult::Clean(ours.to_string()));
    }

    match diffy::apply(base, &patch_ours) {
        Ok(with_ours) => match diffy::apply(&with_ours, &patch_theirs) {
            Ok(merged) => Ok(MergeResult::Clean(merged)),
            Err(_) => {
                let markers = format!("<<<<<<< OURS\n{ours}\n=======\n{theirs}\n>>>>>>> THEIRS");
                Ok(MergeResult::Conflict {
                    merged_with_markers: markers,
                    conflict_count: 1,
                })
            }
        },
        Err(e) => Err(EchoAccessError::Sync(format!("Merge apply failed: {e}"))),
    }
}

pub fn compute_diff(source: &str, target: &str) -> String {
    let patch = diffy::create_patch(source, target);
    format!("{patch}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_changes_clean_merge() {
        let base = "line1\nline2\n";
        let result = three_way_merge(base, base, base).unwrap();
        assert!(matches!(result, MergeResult::Clean(_)));
    }

    #[test]
    fn one_side_changed_clean() {
        let base = "line1\nline2\n";
        let ours = "line1\nline2\nline3\n";
        let result = three_way_merge(base, ours, base).unwrap();
        match result {
            MergeResult::Clean(s) => assert!(s.contains("line3")),
            _ => panic!("expected clean merge"),
        }
    }

    #[test]
    fn compute_diff_shows_changes() {
        let diff = compute_diff("a\n", "b\n");
        assert!(!diff.is_empty());
    }

    #[test]
    fn both_sides_same_change_clean() {
        let base = "line1\n";
        let changed = "line1\nline2\n";
        let result = three_way_merge(base, changed, changed).unwrap();
        assert!(matches!(result, MergeResult::Clean(_)));
    }
}
