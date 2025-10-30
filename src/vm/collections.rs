use crate::object::{Object, ObjectType};
use std::rc::Rc;

/// Collects elements from an iterable object into a Vec.
///
/// This function converts various iterable types (List, Tuple, String) into a
/// uniform Vec<Object> representation that can be easily processed.
///
/// # Arguments
/// * `value` - The object to collect elements from
///
/// # Returns
/// * `Some(Vec<Object>)` - If the object is iterable, returns its elements
/// * `None` - If the object is not iterable
///
/// # Supported Types
/// - **List**: Returns a clone of the list elements
/// - **Tuple**: Returns a clone of the tuple elements
/// - **String**: Returns each character as a separate String object
///
/// # Example
/// ```rust,ignore
/// let list = Rc::new(ObjectType::List(vec![...]));
/// if let Some(elements) = collect_iterable(&list) {
///     // Process elements
/// }
/// ```
pub fn collect_iterable(value: &Object) -> Option<Vec<Object>> {
    match &**value {
        ObjectType::List(elements) => Some(elements.clone()),
        ObjectType::Tuple(elements) => Some(elements.clone()),
        ObjectType::String(text) => Some(
            text.chars()
                .map(|ch| Rc::new(ObjectType::String(ch.to_string())))
                .collect(),
        ),
        _ => None,
    }
}

/// Computes the indices for a slice operation with support for step values.
///
/// This function implements Python's slice semantics, including:
/// - Negative indices (counting from the end)
/// - Omitted start/end (defaulting to beginning/end)
/// - Positive and negative step values
/// - Bounds clamping to prevent out-of-range access
///
/// # Arguments
/// * `len` - The length of the sequence being sliced
/// * `start` - Optional start index (None means default)
/// * `end` - Optional end index (None means default)
/// * `step` - The step value (must be non-zero)
///
/// # Returns
/// * `Some(Vec<usize>)` - The indices to include in the slice
/// * `None` - If step is 0 (invalid)
///
/// # Examples
/// ```rust,ignore
/// // arr[1:4] -> indices [1, 2, 3]
/// slice_indices(10, Some(1), Some(4), 1); // Some(vec![1, 2, 3])
///
/// // arr[::2] -> indices [0, 2, 4, 6, 8]
/// slice_indices(10, None, None, 2); // Some(vec![0, 2, 4, 6, 8])
///
/// // arr[::-1] -> indices [9, 8, 7, ..., 0]
/// slice_indices(10, None, None, -1); // Some(vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0])
/// ```
pub fn slice_indices(
    len: usize,
    start: Option<i64>,
    end: Option<i64>,
    step: i64,
) -> Option<Vec<usize>> {
    if step == 0 {
        return None;
    }

    if len == 0 {
        return Some(Vec::new());
    }

    let len_isize = len as isize;
    let step_isize = step as isize;
    let step_positive = step_isize > 0;

    let start_idx = adjust_index(start, len_isize, false, step_positive);
    let end_idx = adjust_index(end, len_isize, true, step_positive);

    let mut indices = Vec::new();

    if step_positive {
        let mut idx = start_idx;
        while idx < end_idx {
            if idx >= 0 && idx < len_isize {
                indices.push(idx as usize);
            }
            idx += step_isize;
        }
    } else {
        let mut idx = start_idx;
        while idx > end_idx {
            if idx >= 0 && idx < len_isize {
                indices.push(idx as usize);
            }
            idx += step_isize;
        }
    }

    Some(indices)
}

/// Adjusts a slice index according to Python's slicing rules.
///
/// This helper function handles:
/// - Negative index conversion (e.g., -1 becomes len-1)
/// - Bounds clamping for positive steps (0 to len)
/// - Bounds clamping for negative steps (-1 to len-1)
/// - Default values when index is None
///
/// # Arguments
/// * `index` - The index value, or None for default
/// * `len` - The length of the sequence
/// * `is_end` - Whether this is the end index (affects defaults)
/// * `step_positive` - Whether the step is positive (affects defaults and clamping)
///
/// # Returns
/// The adjusted index as an isize
fn adjust_index(index: Option<i64>, len: isize, is_end: bool, step_positive: bool) -> isize {
    let len_i64 = len as i64;
    match index {
        Some(mut value) => {
            if value < 0 {
                value += len_i64;
            }
            if step_positive {
                if value < 0 {
                    value = 0;
                }
                if value > len_i64 {
                    value = len_i64;
                }
            } else {
                if value < -1 {
                    value = -1;
                }
                if value >= len_i64 {
                    value = len_i64 - 1;
                }
            }
            value as isize
        }
        None => {
            if step_positive {
                if is_end {
                    len
                } else {
                    0
                }
            } else if is_end || len_i64 <= 0 {
                -1
            } else {
                len - 1
            }
        }
    }
}
