pub fn f64_to_u64(f: f64) -> u64 {
    u64::from_be_bytes(f.to_be_bytes())
}

pub fn u64_to_f64(u: u64) -> f64 {
    f64::from_be_bytes(u.to_be_bytes())
}
