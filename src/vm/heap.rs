use std::collections::HashMap;

use super::metadata::Metadata;

// Will be extended later on to support more metadata
pub type HeapObjectHeader = (bool, u64); // flag for future gc, index in heap hashmap

#[derive(Debug)]
pub struct List {
    pub item_size: usize,
    pub size: usize,
    pub data: Vec<u64>,
}

#[derive(Debug)]
pub struct CustomObject {
    pub type_index: u64,
    pub data: Vec<u64>,
}

#[derive(Debug)]
pub enum HeapObject {
    String(String),
    List(List),
    CustomObject(CustomObject),
}

#[derive(Debug, Default)]
pub struct Heap {
    data: HashMap<u64, (HeapObjectHeader, Box<HeapObject>)>,
    counter: u64,
}
// TODO: check performance gains from using unreachable_unchecked or smth like that

impl HeapObject {
    pub fn extract_string(&self) -> &String {
        match self {
            HeapObject::String(s) => s,
            _ => unreachable!("Trying to extract string from non-string object"),
        }
    }
    pub fn extract_string_mut(&mut self) -> &mut String {
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
    pub fn extract_custom_object(&mut self) -> &mut CustomObject {
        match self {
            HeapObject::CustomObject(c) => c,
            _ => panic!("Trying to extract object memory from non-custom object"),
        }
    }
}

impl Heap {
    pub fn new() -> Self {
        Self { data: HashMap::new(), counter: 0 }
    }

    pub fn allocate_custom(
        &mut self,
        type_index: usize,
        meta: &Metadata,
    ) -> (u64, &mut CustomObject) {
        let obj_size = meta.types_sizes[type_index];
        let obj = Box::new(HeapObject::CustomObject(CustomObject {
            type_index: type_index as u64,
            data: vec![0; obj_size as usize],
        }));
        let (pos, obj) = self.insert(obj);
        (pos, obj.extract_custom_object())
    }

    pub fn allocate_string(&mut self, capacity: usize) -> (u64, &mut String) {
        let obj = Box::new(HeapObject::String(String::with_capacity(capacity)));

        let (pos, obj) = self.insert(obj);
        (pos, obj.extract_string_mut())
    }

    pub fn move_string(&mut self, s: String) -> (u64, &mut String) {
        let obj = Box::new(HeapObject::String(s));
        let (pos, obj) = self.insert(obj);
        (pos, obj.extract_string_mut())
    }
    
    pub fn allocate_list(
        &mut self,
        list_type_index: usize,
        initial_list_size: usize,
        copy_from: &[u64],
        meta: &Metadata,
    ) -> (u64, &mut List) {
        let item_size = meta.list_types_sizes[list_type_index];
        let memory_size = item_size * initial_list_size;

        let mut list = vec![0; memory_size];
        list[..memory_size].clone_from_slice(&copy_from[..memory_size]);

        let obj = Box::new(HeapObject::List(List {
            item_size,
            size: initial_list_size,
            data: list,
        }));

        let (pos, obj) = self.insert(obj);
        (pos, obj.extract_list())
    }

    fn insert(&mut self, object: Box<HeapObject>) -> (u64, &mut HeapObject) {
        let index = u64::MAX - self.counter;
        self.counter += 1;

        self.data.insert(index, ((false, index), object));
        (index, self.get_mut(index))
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
    pub fn get_item_mem(&mut self, index: usize) -> &mut [u64] {
        &mut self.data[index * self.item_size..]
    }

    pub fn normalize_index(&self, index: i64) -> usize {
        if index < 0 {
            if index.abs() > self.size as i64 {
                panic!(
                    "Negative out of bounds: list of size {} but {} requested",
                    self.size, index
                );
            }
            (self.size as i64 + index) as usize
        } else {
            if index >= self.size as i64 {
                panic!(
                    "Out of bounds: list of size {} but {} requested",
                    self.size, index
                );
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
