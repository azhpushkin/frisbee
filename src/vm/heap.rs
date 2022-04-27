use std::collections::HashMap;

// Will be extended later on to support more metadata
pub type HeapObjectHeader = (bool, u64); // flag for future gc, index in heap hashmap

#[derive(Debug)]
pub struct List {
    pub item_size: usize,
    pub size: usize,
    pub data: Vec<u64>,
}

#[derive(Debug)]
pub enum HeapObject {
    String(String),
    List(List),
    Custom(Vec<u64>),
}

#[derive(Debug)]
pub struct Heap {
    data: HashMap<u64, (HeapObjectHeader, Box<HeapObject>)>,
    counter: u64,
}
// TODO: check performance gains from using unreachable_unchecked or smth like that

impl HeapObject {
    pub fn new_custom(size: usize) -> Box<Self> {
        Box::new(HeapObject::Custom(vec![0; size]))
    }
    pub fn new_string(s: String) -> Box<Self> {
        Box::new(HeapObject::String(s))
    }
    pub fn new_list(item_size: usize, initial_list_size: usize, copy_from: &[u64]) -> Box<Self> {
        let memory_size = item_size * initial_list_size;
        let mut list = vec![0; memory_size];
        for i in 0..memory_size {
            list[i] = copy_from[i];
        }
        Box::new(HeapObject::List(List{ item_size, size: initial_list_size, data: list }))
    }

    pub fn extract_string(&self) -> &String {
        // String are immutable so no &mut self needed
        match self {
            HeapObject::String(s) => s,
            _ => unreachable!("Trying to extract string from non-string object"),
        }
    }
    pub fn extract_list(&mut self) -> &mut List {
        match self {
            HeapObject::List(l) => l,
            _ => unreachable!("Trying to extract list item memory from non-list object"),
        }
    }
    pub fn extract_object_memory(&mut self) -> &mut Vec<u64> {
        match self {
            HeapObject::Custom(data) => data,
            _ => panic!("Trying to extract object memory from non-custom object"),
        }
    }
}

impl Heap {
    pub fn new() -> Self {
        Self { data: HashMap::new(), counter: 0 }
    }

    pub fn insert(&mut self, object: Box<HeapObject>) -> u64 {
        let index = u64::MAX - self.counter;
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

    pub fn simple_debug_view(&self) -> String {
        let mut s = String::from("HEAP STATE: \n");
        for (index, obj) in self.data.iter() {
            s.push_str(
                format!(
                    "\t{} => {:?}\t// meta: {{is_marked: {}, inner_index: {}}}\n",
                    index, obj.1, obj.0 .0, obj.0 .1
                )
                .as_str(),
            );
        }
        s
    }
}

impl List {
    pub fn get_item_mem(&mut self, index: usize) -> &mut [u64]{
        &mut self.data[index*self.item_size..]
    }

    pub fn normalize_index(&self, index: i64) -> usize {
        if index < 0 {
            if index.abs() > self.size as i64 {
                panic!("Negative out of bounds: list of size {} but {} requested", self.size, index);
            }
            (self.size as i64 + index) as usize
        } else {
            if index >= self.size as i64 {
                panic!("Out of bounds: list of size {} but {} requested", self.size, index);
            }
            index as usize
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normalize_index() {
        let l = List { item_size: 1, size: 10, data: vec![0; 10] };

        assert_eq!(l.normalize_index(0), 0);
        assert_eq!(l.normalize_index(1), 1);
        assert_eq!(l.normalize_index(-1), 9);
        assert_eq!(l.normalize_index(-10), 0);
    }

    #[test]
    #[should_panic(expected = "Out of bounds: list of size 10 but 10 requested")]
    fn too_big_index_panics() {
        let l = List { item_size: 1, size: 10, data: vec![0; 10] };
        l.normalize_index(10);
    }

    #[test]
    #[should_panic(expected = "Negative out of bounds: list of size 10 but -11 requested")]
    fn too_small_index_panics() {
        let l = List { item_size: 1, size: 10, data: vec![0; 10] };
        l.normalize_index(-11);
    }
}
