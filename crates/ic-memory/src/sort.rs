//! In-place canonical ordering for the crate's bounded allocation collections.

use std::cmp::Ordering;

// Heap sorting keeps O(n log n) worst-case work and constant auxiliary space
// without linking a general-purpose quicksort for each allocation record type.
pub fn sort_unstable_by<T>(values: &mut [T], compare: impl Fn(&T, &T) -> Ordering) {
    let Some((first, last)) = values.first().zip(values.last()) else {
        return;
    };
    let descending = compare(first, last).is_gt();
    // Canonical snapshots and reversed input need only a linear scan.
    if values.windows(2).all(|pair| {
        let order = compare(&pair[0], &pair[1]);
        if descending {
            !order.is_lt()
        } else {
            !order.is_gt()
        }
    }) {
        if descending {
            for start in 0..values.len() / 2 {
                values.swap(start, values.len() - 1 - start);
            }
        }
        return;
    }
    for root in (0..values.len() / 2).rev() {
        sift_down(values, root, values.len(), &compare);
    }
    for end in (1..values.len()).rev() {
        values.swap(0, end);
        sift_down(values, 0, end, &compare);
    }
}

fn sift_down<T>(
    values: &mut [T],
    mut root: usize,
    end: usize,
    compare: &impl Fn(&T, &T) -> Ordering,
) {
    while root < end / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < end && compare(&values[child], &values[child + 1]).is_lt() {
            child += 1;
        }
        if !compare(&values[root], &values[child]).is_lt() {
            break;
        }
        values.swap(root, child);
        root = child;
    }
}

#[cfg(test)]
mod tests {
    use super::sort_unstable_by;

    #[test]
    fn canonical_order_preserves_owned_values_across_allocation_bounds() {
        for len in 0..=255 {
            for keys in [
                (0..len).collect::<Vec<_>>(),
                (0..len).rev().collect(),
                (0..len).map(|index| (index * 73 + 19) % 17).collect(),
            ] {
                let values: Vec<_> = keys
                    .into_iter()
                    .enumerate()
                    .map(|(index, key)| (key, format!("owned value {index}")))
                    .collect();
                for descending in [false, true] {
                    let compare = |a: &(usize, String), b: &(usize, String)| {
                        if descending { b.cmp(a) } else { a.cmp(b) }
                    };
                    let mut expected = values.clone();
                    expected.sort_unstable_by(compare);
                    let mut actual = values.clone();
                    sort_unstable_by(&mut actual, compare);
                    assert_eq!(actual, expected);
                }
            }
        }
    }
}
