use std::io;

fn std_println(stack: &mut [u64], memory: &mut Vec<String>) {
    println!("[Println] {}", memory[stack[1] as usize]);
}

fn std_print(stack: &mut [u64], memory: &mut Vec<String>) {
    print!("[Print] {}", memory[stack[1] as usize]);
}

fn std_range(stack: &mut [u64], memory: &mut Vec<String>) {
    panic!("Range is not implemented yet");
}

fn std_get_input(stack: &mut [u64], memory: &mut Vec<String>) {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    memory.push(input.trim().into());
    stack[0] = memory.len() as u64 - 1;
}

pub static STD_FUNCTIONS: [for<'r, 's> fn(&'r mut [u64], &'s mut Vec<String>); 4] =
    [std_print, std_println, std_range, std_get_input];
