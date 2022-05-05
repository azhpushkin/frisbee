use crate::types::Type;

pub fn get_type_size<T>(t: &Type<T>) -> u8 {
    match t {
        Type::Int => 1,
        Type::Float => 1,
        Type::Bool => 1,
        Type::String => 1,
        Type::Maybe(inner) => get_type_size(inner) + 1,
        Type::Tuple(items) => items.iter().map(|t| get_type_size(t)).sum(),
        Type::List(_) => 1,
        Type::Custom(_) => 1,
    }
}

pub fn get_type_from_tuple<T>(t: &Type<T>, i: usize) -> &Type<T> {
    match t {
        Type::Tuple(items) => &items[i],
        Type::Maybe(inner) => match i {
            0 => &Type::Bool,
            1 => inner.as_ref(),
            i => panic!("Accessing maybe with wrong index {}, semantics failed", i),
        },
        _ => panic!("Tuple access on non-tuple type, semantics failed"),
    }
}

pub fn extract_custom_type<T>(t: &Type<T>) -> &T {
    match t {
        Type::Custom(s) => s,
        _ => panic!("Field or method access on non-custom type, emantics failed"),
    }
}

pub fn get_tuple_offset<T>(tuple_type: &Type<T>, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return 0;
    }

    match tuple_type {
        Type::Tuple(items) => {
            let mut offset: u8 = 0;
            let current_index = tuple_indexes[0];
            for i in 0..current_index {
                offset += get_type_size(&items[i]);
            }
            offset += get_tuple_offset(&items[current_index], &tuple_indexes[1..]) as u8;
            offset
        }
        _ => 0,
    }
}

pub fn get_tuple_subitem_size<T>(tuple_type: &Type<T>, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return get_type_size(tuple_type);
    } else {
        return get_tuple_subitem_size(
            &get_type_from_tuple(tuple_type, tuple_indexes[0]),
            &tuple_indexes[1..],
        );
    }
}

pub fn get_list_inner_type<T>(list_type: &Type<T>) -> &Type<T> {
    match list_type {
        Type::List(inner_type) => inner_type.as_ref(),
        _ => panic!("List index access on non-list type, semantics failed"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_tuple_offset() {
        // (Int, ((Float, SomeType), String), ((), [SomeType]))

        let test_type = Type::Tuple(vec![
            Type::Int,
            Type::Tuple(vec![
                Type::Tuple(vec![Type::Float, Type::Custom("SomeType")]),
                Type::String,
            ]),
            Type::Tuple(vec![
                Type::Tuple(vec![]),
                Type::List(Box::new(Type::Custom("SomeType"))),
            ]),
        ]);

        assert_eq!(get_tuple_offset(&test_type, &[]), 0);
        assert_eq!(get_tuple_offset(&test_type, &[0]), 0);
        assert_eq!(get_tuple_offset(&test_type, &[1]), 1); // skip int
        assert_eq!(get_tuple_offset(&test_type, &[1, 1]), 3); // skip int, float and SomeType
        assert_eq!(get_tuple_offset(&test_type, &[2]), 4); // skip int, float, SomeType and String
        assert_eq!(get_tuple_offset(&test_type, &[2, 0]), 4); // skip int, float, SomeType and String
        assert_eq!(get_tuple_offset(&test_type, &[2, 1]), 4); // skip int, float, SomeType and String
    }

    // TODO: add type::maybe to this test
}
