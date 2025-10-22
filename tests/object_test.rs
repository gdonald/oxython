use oxython::object::ObjectType;

#[test]
fn display_formats_boolean_and_nil() {
    assert_eq!(format!("{}", ObjectType::Boolean(true)), "True");
    assert_eq!(format!("{}", ObjectType::Boolean(false)), "False");
    assert_eq!(format!("{}", ObjectType::Nil), "nil");
}

#[test]
fn display_formats_float() {
    assert_eq!(format!("{}", ObjectType::Float(3.14)), "3.14");
}
