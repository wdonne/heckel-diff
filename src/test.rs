use crate::Change::{Delete, Insert, Unchanged};
use crate::{diff, new_version, old_version};

#[test]
fn make_ends_meet() {
    let old = vec!["a", "b", "c", "d", "a"];
    let new = vec!["a", "f", "c", "g", "a"];
    let answer = vec![
        Unchanged(&"a", 0, 0),
        Insert(&"f", 1),
        Delete(&"b", 1),
        Unchanged(&"c", 2, 2),
        Insert(&"g", 3),
        Delete(&"d", 3),
        Unchanged(&"a", 4, 4),
    ];
    let calculated = diff(&old, &new);
    let new_version: Vec<&str> = new_version(calculated.as_slice())
        .into_iter()
        .copied()
        .collect();
    let old_version: Vec<&str> = old_version(calculated.as_slice())
        .into_iter()
        .copied()
        .collect();

    assert_eq!(answer, calculated);
    assert_eq!(new, new_version);
    assert_eq!(old, old_version);
}
