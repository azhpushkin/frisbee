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

macro_rules! unwrap_type_as {
    ($value:expr, $variant:path $(,)?) => {
        match $value {
            $variant(a) => a,
            other => panic!(
                "Wrong unwrap, expected {:?}, got {:?}",
                stringify!($variant),
                other
            ),
        }
    };
}
pub(crate) use unwrap_type_as;

pub fn get_tuple_offset<T: std::fmt::Debug>(tuple_type: &Type<T>, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        return 0;
    }
    let current_index = tuple_indexes[0];
    let next_indexes = &tuple_indexes[1..];

    if let Type::Maybe(inner) = tuple_type {
        match current_index {
            0 if !next_indexes.is_empty() => panic!("Accessing inners of maybe flag"),
            0 => 0,
            1 => 1 + get_tuple_offset(inner.as_ref(), next_indexes),
            _ => panic!("Maybe indexes must be 0 or 1, but got {}", current_index),
        }
    } else {
        let items = unwrap_type_as!(tuple_type, Type::Tuple);

        let current_item_offset: u8 = items.iter().take(current_index).map(get_type_size).sum();
        let offset_inside_current_item = get_tuple_offset(&items[current_index], next_indexes);

        current_item_offset + offset_inside_current_item
    }
}

pub fn get_tuple_subitem_type<T>(t: &Type<T>, i: usize) -> &Type<T> {
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

pub fn get_tuple_subitem_size<T>(tuple_type: &Type<T>, tuple_indexes: &[usize]) -> u8 {
    if tuple_indexes.is_empty() {
        get_type_size(tuple_type)
    } else {
        return get_tuple_subitem_size(
            get_tuple_subitem_type(tuple_type, tuple_indexes[0]),
            &tuple_indexes[1..],
        );
    }
}

pub fn get_pointers_map_for_type<T>(t: &Type<T>) -> Vec<usize> {
    match t {
        Type::Int | Type::Float | Type::Bool => vec![],
        Type::Maybe(t) => {
            let inner = get_pointers_map_for_type(t.as_ref());
            inner.into_iter().map(|i| i + 1).collect()
        }
        Type::List(_) | Type::Custom(_) | Type::String => vec![0],

        Type::Tuple(items) => get_pointers_map_for_sequence(items),
    }
}

pub fn get_pointers_map_for_sequence<T>(types: &[Type<T>]) -> Vec<usize> {
    if types.is_empty() {
        return vec![];
    }
    if types.len() == 1 {
        return get_pointers_map_for_type(&types[0]);
    }

    let mut result = vec![];
    let mut current_offset: usize = 0;
    for t in types {
        let inner = get_pointers_map_for_type(t);
        result.extend(inner.into_iter().map(|i| i + current_offset));
        current_offset += get_type_size(t) as usize;
    }
    result.sort_unstable();

    result
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

    #[test]
    fn check_tuple_offset_for_maybe() {
        // (Int?, (Float, SomeType))?

        let test_type = Type::Maybe(Box::new(Type::Tuple(vec![
            Type::Maybe(Box::new(Type::Int)),
            Type::Tuple(vec![Type::Float, Type::Custom("SomeType")]),
        ])));

        assert_eq!(get_tuple_offset(&test_type, &[]), 0);
        assert_eq!(get_tuple_offset(&test_type, &[0]), 0);
        assert_eq!(get_tuple_offset(&test_type, &[1]), 1); // skip initial header
        assert_eq!(get_tuple_offset(&test_type, &[1, 0, 1]), 2); // skip initial header + Int? header
        assert_eq!(get_tuple_offset(&test_type, &[1, 1]), 3); // skip both headers + Int? value
        assert_eq!(get_tuple_offset(&test_type, &[1, 1, 1]), 4); // skip both headers + Int? value + Float
    }

    #[test]
    fn check_get_pointers_mapping() {
        // (Int, ((Float, SomeType), String), ((), [SomeType]))

        let items = vec![
            Type::Int,
            Type::Tuple(vec![
                Type::Tuple(vec![Type::Float, Type::Custom("SomeType")]),
                Type::String,
            ]),
            Type::Tuple(vec![
                Type::Tuple(vec![]),
                Type::List(Box::new(Type::Custom("SomeType"))),
            ]),
        ];

        let mapping = get_pointers_map_for_sequence(&items);
        assert_eq!(mapping, vec![2, 3, 4]);
    }

    #[test]
    fn check_get_pointers_mapping_for_maybe() {
        // (Int?, (Float, SomeType))?
        // (bool, bool, int, float, sometype)

        let test_type = Type::Maybe(Box::new(Type::Tuple(vec![
            Type::Maybe(Box::new(Type::Int)),
            Type::Tuple(vec![Type::Float, Type::Custom("SomeType")]),
        ])));

        let mapping = get_pointers_map_for_type(&test_type);
        assert_eq!(mapping, vec![4]);
    }
}
