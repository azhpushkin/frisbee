use super::heap::Heap;
use super::utils::{f64_to_u64, u64_to_f64};
use std::io::{self, Write};

pub type RawStdRunner = for<'r, 's> fn(&'r mut [u64], &'s mut Heap);

fn std_println(stack: &mut [u64], memory: &mut Heap) {
    let obj = memory.get_mut(stack[0]);
    println!("[Println] {}", obj.extract_string());
}

fn std_print(stack: &mut [u64], memory: &mut Heap) {
    let obj = memory.get_mut(stack[0]);
    print!("[Print] {}", obj.extract_string());
    io::stdout().flush().unwrap();
}

fn std_range(stack: &mut [u64], memory: &mut Heap) {
    let start = stack[1] as i64;
    let end = stack[2] as i64;
    println!(">> generating range from {} to {}", start, end - 1);

    let (list_pos, list_object) = memory.new_list(1, 0, &[]);
    let mut list_inner = list_object.extract_list();
    list_inner.size = (end - start) as usize;
    list_inner.data.resize(list_inner.size, 0);
    for i in start..end {
        list_inner.data[i as usize - start as usize] = i as u64;
    }

    stack[0] = list_pos;
}

fn std_get_input(stack: &mut [u64], memory: &mut Heap) {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    stack[0] = memory.new_string(input.trim().into()).0;
}

fn std_bool_to_string(stack: &mut [u64], memory: &mut Heap) {
    let s = if stack[1] > 0 { "true" } else { "false" };

    stack[0] = memory.new_string(s.into()).0;
}

fn std_int_to_string(stack: &mut [u64], memory: &mut Heap) {
    let s = (stack[1] as i64).to_string();

    stack[0] = memory.new_string(s).0;
}

fn std_int_to_float(stack: &mut [u64], _memory: &mut Heap) {
    stack[0] = f64_to_u64((stack[1] as i64) as f64);
}

fn std_int_abs(stack: &mut [u64], _memory: &mut Heap) {
    stack[0] = (stack[1] as i64).abs() as u64;
}

fn std_float_round(stack: &mut [u64], _memory: &mut Heap) {
    stack[0] = (u64_to_f64(stack[1]).round() as i64) as u64;
}

fn std_float_to_string(stack: &mut [u64], memory: &mut Heap) {
    let s = u64_to_f64(stack[1]).to_string();

    stack[0] = memory.new_string(s).0;
}

fn std_float_abs(stack: &mut [u64], _memory: &mut Heap) {
    stack[0] = f64_to_u64(u64_to_f64(stack[1]).abs());
}

fn std_list_push(stack: &mut [u64], memory: &mut Heap) {
    let list_obj = memory.get_mut(stack[0]);
    let list = list_obj.extract_list();

    list.size += 1;
    let item_size = list.item_size;

    for i in 0..item_size {
        list.data.push(stack[1 + i]);
    }
}

fn std_list_pop(stack: &mut [u64], memory: &mut Heap) {
    let list_obj = memory.get_mut(stack[stack.len() - 1]);
    let list = list_obj.extract_list();

    list.size -= 1;
    let item_size = list.item_size;

    for i in 0..item_size {
        stack[item_size - i - 1] = list.data.pop().unwrap();
    }
}

fn std_list_len(stack: &mut [u64], memory: &mut Heap) {
    let list_obj = memory.get_mut(stack[1]);
    let list = list_obj.extract_list();

    stack[0] = list.size as u64;
}

fn noop(_stack: &mut [u64], _memory: &mut Heap) {
    panic!("not implemented yet");
}

#[rustfmt::skip]
pub static STD_RAW_FUNCTION_RUNNERS: [(&'static str, RawStdRunner); 21] = [
    ("std::print", std_print),
    ("std::println", std_println),
    ("std::range", std_range),
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

    ("std::List::push", std_list_push),
    ("std::List::pop", std_list_pop),
    ("std::List::len", std_list_len),
    ("std::List::is_empty", noop),
];
