use crate::object::ObjectType;

pub fn is_truthy(value: &ObjectType) -> bool {
    match value {
        ObjectType::Nil => false,
        ObjectType::Boolean(b) => *b,
        _ => true,
    }
}
