use std::collections::HashMap;

// Will be extended later on to support more metadata
pub type HeapObjectHeader = (bool, u64); // flag for future gc, index in heap hashmap

pub enum HeapObject {
    String(String),
    List(Vec<u64>),
    Custom(Vec<u64>),
}

pub struct Heap {
    data: HashMap<u64, (HeapObjectHeader, Box<HeapObject>)>,
    counter: u64,
}

impl HeapObject {
    pub fn new_custom(size: usize) -> Box<Self> {
        Box::new(HeapObject::Custom(vec![0; size]))
    }
    pub fn new_string(s: String) -> Box<Self> {
        Box::new(HeapObject::String(s))
    }

    pub fn extract_string(&self) -> &String {
        match self {
            HeapObject::String(s) => s,
            _ => unreachable!("Trying to extract string from non-string object"),
        }
    }
    pub fn extract_memory_mut(&mut self, offset: u8) -> &mut [u64] {
        let mem = match self {
            HeapObject::List(l) => l,
            HeapObject::Custom(i) => i,
            HeapObject::String(_) => panic!("Strings must be processed in a special way"),
        };
        &mut mem[offset as usize..]
    }
}

impl Heap {
    pub fn new() -> Self {
        Self { data: HashMap::new(), counter: 0 }
    }

    pub fn insert(&mut self, object: Box<HeapObject>) -> u64 {
        let index = self.counter;
        self.counter += 1;

        self.data.insert(index, ((false, index), object));
        index
    }

    pub fn get_mut(&mut self, index: u64) -> &mut HeapObject {
        let obj = self.data.get_mut(&index).unwrap();
        &mut obj.1
    }
    pub fn get(&self, index: u64) -> &HeapObject {
        let obj = self.data.get(&index).unwrap();
        &obj.1
    }
}
