use crate::object::ObjectType;

/// Determines whether a value is "truthy" according to Python semantics.
///
/// In Python, certain values are considered "falsy" and evaluate to False in
/// boolean contexts, while all other values are "truthy" and evaluate to True.
///
/// # Falsy Values
/// - `None` (represented as `ObjectType::Nil`)
/// - `False` (represented as `ObjectType::Boolean(false)`)
///
/// # Truthy Values
/// All other values are considered truthy, including:
/// - `True` (represented as `ObjectType::Boolean(true)`)
/// - Any non-zero number (Integer, Float)
/// - Any non-empty collection (List, Tuple, Dict, String)
/// - Any object instance
/// - Any function or class
///
/// # Arguments
/// * `value` - A reference to the ObjectType to evaluate
///
/// # Returns
/// * `true` - If the value is truthy
/// * `false` - If the value is falsy
///
/// # Examples
/// ```rust,ignore
/// let nil = ObjectType::Nil;
/// assert_eq!(is_truthy(&nil), false);
///
/// let false_val = ObjectType::Boolean(false);
/// assert_eq!(is_truthy(&false_val), false);
///
/// let true_val = ObjectType::Boolean(true);
/// assert_eq!(is_truthy(&true_val), true);
///
/// let num = ObjectType::Integer(0);
/// assert_eq!(is_truthy(&num), true); // Even 0 is truthy (different from Python!)
///
/// let empty_list = ObjectType::List(vec![]);
/// assert_eq!(is_truthy(&empty_list), true); // Even empty collections are truthy
/// ```
///
/// # Note
/// This implementation is simplified compared to full Python semantics.
/// In real Python:
/// - Empty collections ([], {}, "", ()) are falsy
/// - Zero values (0, 0.0) are falsy
/// - Custom objects can define `__bool__` or `__len__` to customize truthiness
///
/// This implementation treats all values as truthy except `None` and `False`.
pub fn is_truthy(value: &ObjectType) -> bool {
    match value {
        ObjectType::Nil => false,
        ObjectType::Boolean(b) => *b,
        _ => true,
    }
}
