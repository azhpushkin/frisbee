use crate::types::Type;

pub fn get_type_from_tuple<T>(t: &Type<T>, i: usize) -> &Type<T> {
    match t {
        Type::Tuple(items) => &items[i],
        _ => panic!("something is wrong, semantics should have checked this.."),
    }
}

pub fn extract_custom_type<T>(t: &Type<T>) -> &T {
    match t {
        Type::Custom(s) => s,
        _ => panic!("something is wrong, semantics should have checked this.."),
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
                offset += items[i].get_size();
            }
            offset += get_tuple_offset(&items[current_index], &tuple_indexes[1..]) as u8;
            offset
        }
        _ => 0,
    }
}

pub fn get_tuple_subitem_size<T>(tuple_type: &Type<T>, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return tuple_type.get_size();
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
        _ => panic!("something is wrong, semantics should have checked this.."),
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
