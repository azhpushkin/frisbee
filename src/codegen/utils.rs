use crate::types::Type;

pub fn get_type_from_tuple<'a>(t: &'a Type, i: usize) -> &'a Type {
    match t {
        Type::Tuple(items) => &items[i],
        _ => panic!("something is wrong, semantics should have checked this.."),
    }
}

pub fn get_type_size(t: &Type) -> u8 {
    match t {
        Type::Int => 1,
        Type::Float => 1,
        Type::Bool => 1,
        Type::String => 1,
        Type::Maybe(inner) => get_type_size(inner.as_ref()) + 1,
        Type::Tuple(items) => items.iter().map(|t| get_type_size(t)).sum(),
        Type::List(_) => 1,
        Type::Ident(_) => 1,
    }
}

pub fn get_tuple_offset(tuple_type: &Type, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return 0;
    }

    println!("Got tuple_type {:?} and {:?}", tuple_type, tuple_indexes);
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

pub fn get_tuple_subitem_size(tuple_type: &Type, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return get_type_size(tuple_type);
    } else {
        return get_tuple_subitem_size(
            &get_type_from_tuple(tuple_type, tuple_indexes[0]),
            &tuple_indexes[1..],
        );
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
                Type::Tuple(vec![Type::Float, Type::Ident("SomeType".into())]),
                Type::String,
            ]),
            Type::Tuple(vec![
                Type::Tuple(vec![]),
                Type::List(Box::new(Type::Ident("SomeType".into()))),
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
