use oxython::object::ObjectType;

#[test]
fn display_formats_boolean_and_nil() {
    assert_eq!(format!("{}", ObjectType::Boolean(true)), "true");
    assert_eq!(format!("{}", ObjectType::Boolean(false)), "false");
    assert_eq!(format!("{}", ObjectType::Nil), "nil");
}

#[test]
fn display_formats_float() {
    assert_eq!(format!("{}", ObjectType::Float(3.14)), "3.14");
}
