macro_rules! extract_result_if_ok {
    ($parse_result:expr) => {
        match $parse_result {
            Ok(res) => res,
            // Re-wrap pf parsing error is required to coerce type
            // from Result<T, ParseError> to Result<Program, ParseError>
            Err(t) => return Err(t),
        }
    };
}

pub(crate) use extract_result_if_ok;
