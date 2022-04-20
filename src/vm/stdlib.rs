pub fn std_println(stack: &mut [u64], memory: &mut Vec<String>) {
    println!("[Println] {}", memory[stack[1] as usize]);
}

pub fn std_print(stack: &mut [u64], memory: &mut Vec<String>) {
    print!("[Print] {}", memory[stack[1] as usize]);
}