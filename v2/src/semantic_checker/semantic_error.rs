pub type SemanticError = String;
pub type SemanticResult = Result<(), SemanticError>;

macro_rules! sem_err {
    ($($arg : tt), *) => { 
        Err(format!($($arg), *))
     };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_sem_err() {
        let res: SemanticResult = sem_err!("The Answer is {}", 42);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "The Answer is 42");
    }
}