use alloc::vec::Vec;

/// Helper function to extend a sorted vector with another sorted vector.
/// Values from `other` take precedence for duplicate keys.
///
/// This function efficiently merges two sorted vectors by:
/// 1. Iterating through the target vector with mutable references
/// 2. Using a peekable iterator for the other vector
/// 3. For each target item, processing other items that come before or equal to it
/// 4. Collecting items from other that need to be inserted
/// 5. Appending and re-sorting only if new items were added
pub(crate) fn extend_sorted_vec<K, V>(target: &mut Vec<(K, V)>, other: &[(K, V)])
where
    K: Clone + Ord + core::hash::Hash + Eq,
    V: Clone,
{
    if other.is_empty() {
        return;
    }

    let mut other_iter = other.iter().peekable();
    let mut to_insert = Vec::new();

    // Iterate through target and update/collect items from other
    for target_item in target.iter_mut() {
        while let Some(other_item) = other_iter.peek() {
            use core::cmp::Ordering;
            match other_item.0.cmp(&target_item.0) {
                Ordering::Less => {
                    // Other item comes before current target item, collect it
                    to_insert.push(other_iter.next().unwrap().clone());
                }
                Ordering::Equal => {
                    // Same key, update target with other's value
                    target_item.1 = other_iter.next().unwrap().1.clone();
                    break;
                }
                Ordering::Greater => {
                    // Other item comes after current target item, keep target unchanged
                    break;
                }
            }
        }
    }

    // Append collected new items, as well as any remaining from `other` which are necessarily also
    // new. Since both `to_insert` and remaining `other_iter` came from the sorted `other` vector
    // in order, they are already sorted relative to each other. We can merge them with the sorted
    // target more efficiently than re-sorting the entire result.
    if to_insert.is_empty() && other_iter.peek().is_none() {
        // No new items, target is unchanged
        return;
    }

    let original_len = target.len();

    // Collect all new items (both to_insert and remaining from other_iter)
    // These are already in sorted order since they came from sorted `other`
    to_insert.extend(other_iter.cloned());

    // If target was empty, just replace it with the new items
    if original_len == 0 {
        *target = to_insert;
        return;
    }

    // Merge the sorted target with the sorted new items
    // This is O(n + m) instead of O((n+m) log (n+m))
    let mut result = Vec::with_capacity(original_len + to_insert.len());
    let mut target_idx = 0;
    let mut insert_idx = 0;

    while target_idx < original_len && insert_idx < to_insert.len() {
        if target[target_idx].0 <= to_insert[insert_idx].0 {
            result.push(target[target_idx].clone());
            target_idx += 1;
        } else {
            result.push(to_insert[insert_idx].clone());
            insert_idx += 1;
        }
    }

    // Append remaining elements from whichever slice has items left
    if target_idx < original_len {
        result.extend_from_slice(&target[target_idx..]);
    } else if insert_idx < to_insert.len() {
        result.extend_from_slice(&to_insert[insert_idx..]);
    }

    *target = result;
}
