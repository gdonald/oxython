use crate::object::{Object, ObjectType};
use std::rc::Rc;

/// Collects elements from an iterable object into a Vec.
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
