use super::heap;
use super::utils::{f64_to_u64, u64_to_f64};
use std::io::{self, Write};

pub type RawStdRunner = for<'r, 's> fn(&'r mut [u64], &'s mut heap::Heap);

fn std_println(stack: &mut [u64], memory: &mut heap::Heap) {
    let obj = memory.get(stack[0]);
    println!("[Println] {}", obj.extract_string());
}

fn std_print(stack: &mut [u64], memory: &mut heap::Heap) {
    let obj = memory.get(stack[0]);
    print!("[Print] {}", obj.extract_string());
    io::stdout().flush().unwrap();
}

fn std_get_input(stack: &mut [u64], memory: &mut heap::Heap) {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    let obj_pos = memory.insert(heap::HeapObject::String(input.trim().into()));
    stack[0] = obj_pos;
}

fn std_bool_to_string(stack: &mut [u64], memory: &mut heap::Heap) {
    let s = if stack[1] > 0 { "true" } else { "false" };
    
    let obj_pos = memory.insert(heap::HeapObject::String(s.into()));
    stack[0] = obj_pos;
}

fn std_int_to_string(stack: &mut [u64], memory: &mut heap::Heap) {
    let s = (stack[1] as i64).to_string();

    let obj_pos = memory.insert(heap::HeapObject::String(s));
    stack[0] = obj_pos;
}

fn std_int_to_float(stack: &mut [u64], memory: &mut heap::Heap) {
    stack[0] = f64_to_u64((stack[1] as i64) as f64);
}

fn std_int_abs(stack: &mut [u64], memory: &mut heap::Heap) {
    stack[0] = (stack[1] as i64).abs() as u64;
}

fn std_float_round(stack: &mut [u64], memory: &mut heap::Heap) {
    stack[0] = (u64_to_f64(stack[1]).round() as i64) as u64;
}

fn std_float_to_string(stack: &mut [u64], memory: &mut heap::Heap) {
    let s = u64_to_f64(stack[1]).to_string();
    
    let obj_pos = memory.insert(heap::HeapObject::String(s));
    stack[0] = obj_pos;
}

fn std_float_abs(stack: &mut [u64], memory: &mut heap::Heap) {
    stack[0] = f64_to_u64(u64_to_f64(stack[1]).abs());
}

fn noop(stack: &mut [u64], memory: &mut heap::Heap) {
    panic!("not implemented yet");
}

pub static STD_RAW_FUNCTION_RUNNERS: [(&'static str, RawStdRunner); 17] = [
    ("std::print", std_print),
    ("std::println", std_println),
    ("std::range", noop),
    ("std::get_input", std_get_input),
    ("std::Bool::to_string", std_bool_to_string),
    ("std::Int::to_float", std_int_to_float),
    ("std::Int::to_string", std_int_to_string),
    ("std::Int::abs", std_int_abs),
    ("std::Float::to_string", std_float_to_string),
    ("std::Float::abs", std_float_abs),
    ("std::Float::ceil", noop),
    ("std::Float::floor", noop),
    ("std::Float::round", std_float_round),
    ("std::String::len", noop),
    ("std::String::is_empty", noop),
    ("std::String::find", noop),
    ("std::String::contains", noop),
];
