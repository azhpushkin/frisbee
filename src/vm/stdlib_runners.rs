use std::io;

pub type RawStdRunner = for<'r, 's> fn(&'r mut [u64], &'s mut Vec<String>);

fn std_println(stack: &mut [u64], memory: &mut Vec<String>) {
    println!("[Println] {}", memory[stack[1] as usize]);
}

fn std_print(stack: &mut [u64], memory: &mut Vec<String>) {
    print!("[Print] {}", memory[stack[1] as usize]);
}

fn std_get_input(stack: &mut [u64], memory: &mut Vec<String>) {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    memory.push(input.trim().into());
    stack[0] = memory.len() as u64 - 1;
}

fn noop(stack: &mut [u64], memory: &mut Vec<String>) {
    panic!("not implemented yet");
}

pub static STD_RAW_FUNCTION_RUNNERS: [(&'static str, RawStdRunner); 17] = [
    ("std::print", std_print),
    ("std::println", std_println),
    ("std::range", noop),
    ("std::get_input", std_get_input),
    ("std::Bool::to_string", noop),
    ("std::Int::to_float", noop),
    ("std::Int::to_string", noop),
    ("std::Int::abs", noop),
    ("std::Float::to_string", noop),
    ("std::Float::abs", noop),
    ("std::Float::ceil", noop),
    ("std::Float::floor", noop),
    ("std::Float::round", noop),
    ("std::String::len", noop),
    ("std::String::is_empty", noop),
    ("std::String::find", noop),
    ("std::String::contains", noop),
];
