use std::collections::HashMap;


// Will be extended later on to support more metadata
pub type HeapObjectHeader = (bool, u64);  // flag for future gc, index in heap hashmap

pub enum HeapObject {
    String(String),
    List(Vec<HeapObject>),
    Custom(u64)  // ??
}

pub struct Heap {
    data: HashMap<u64, Box<(HeapObjectHeader, HeapObject)>>,
    counter: u64,
}


impl HeapObject {
    pub fn extract_string(&self) -> &String {
        match self {
            HeapObject::String(s) => s,
            _ => unreachable!("Trying to extract string from non-string object"),
        }
    }
    pub fn extract_list(&self) -> &Vec<HeapObject> {
        match self {
            HeapObject::List(l) => l,
            _ => unreachable!("Trying to extract list from non-list object"),
        }
    }
    pub fn extract_custom(&self) -> u64 {
        match self {
            HeapObject::Custom(i) => *i,
            _ => unreachable!("Trying to extract custom from non-custom object"),
        }
    }
}

impl Heap {
    pub fn new() -> Self {
        Self{data: HashMap::new(), counter: 0}
    }

    pub fn insert(&mut self, object: HeapObject) -> u64 {
        let index = self.counter;
        self.counter += 1;
        
        self.data.insert(index, Box::new(((false, index), object)));
        index
    }

    pub fn get(&self, index: u64) -> &HeapObject {
        &self.data.get(&index).unwrap().1
    }

    
}
