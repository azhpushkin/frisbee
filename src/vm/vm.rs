use super::heap::HeapObject;
use super::metadata::{Metadata, MetadataBlock};
use super::opcodes::op;
use super::worker::Worker;

pub static LOCAL_VM: Vm = Vm::default();

#[derive(Default)]
pub struct Vm {
    ip: usize,

    pub program: Vec<u8>,
    pub constants: Vec<u64>,
    pub metadata: Metadata,
    pub entry: usize,

    pub step_by_step: bool,
    pub show_debug: bool,
}

impl Vm {
    pub fn setup(&mut self, program: Vec<u8>) {
        self.program = program;
        self.ip = 0;
        self.check_header("Initial header");
        self.load_consts();
        // TODO: show constants?
        // if show_debug {
        //     println!("Loaded constants: {:?}", self.constants);
        // }

        self.load_metadata();

        let entry = self.load_entry();
    }

    pub fn set_debug_params(&mut self, step_by_step: bool, show_debug: bool) {
        self.step_by_step = step_by_step;
        self.show_debug = show_debug;
    }

    pub fn spawn_entry(&mut self) {
        let worker = Worker::new(self);
        worker.run(self.entry);
    }

    fn check_header(&mut self, header_name: &'static str) {
        let header = self.read_several::<2>();
        if header != [0xff, 0xff] {
            panic!("Cannot find header: {}", header_name);
        }
    }

    fn read_metadata_block(&mut self, info_name: &'static str) -> MetadataBlock {
        let amount = self.read_opcode();
        let mut res = vec![];
        for _ in 0..amount {
            // Read string repr, do not save it - we have no use for it
            let symbol_name_len = u16::from_be_bytes(self.read_several::<2>());
            self.read_bytes(symbol_name_len as usize);

            let flag = u16::from_be_bytes(self.read_several::<2>()) as usize;

            let pointers_amount = self.read_opcode();
            let pointer_mapping = self.read_bytes(pointers_amount as usize);
            res.push((flag, pointer_mapping));
        }
        self.check_header(info_name);
        res
    }

    fn load_entry(&mut self) -> usize {
        let entry = u16::from_be_bytes(self.read_several::<2>());
        self.check_header("Entry loaded, start of functions");
        entry as usize
    }

    fn load_metadata(&mut self) {
        let tm = self.read_metadata_block("Types metadata");
        self.metadata.fill_types_metadata(tm);

        let lm = self.read_metadata_block("Lists metadata");
        self.metadata.fill_lists_metadata(lm);

        let fm = self.read_metadata_block("Functions metadata");
        let functions_count = fm.len();
        self.metadata.fill_function_metadata(fm);

        for i in 0..functions_count {
            let pos = u16::from_be_bytes(self.read_several::<2>()) as usize;
            self.metadata.function_positions.insert(pos, i);
        }
        self.check_header("End of function positions");
    }

    fn load_consts(&mut self) {
        loop {
            let const_type = self.read_opcode();
            match const_type {
                op::CONST_INT_FLAG => {
                    let i = i64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(i as u64);
                }
                op::CONST_FLOAT_FLAG => {
                    let f = u64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(f);
                }
                op::CONST_STRING_FLAG => {
                    let str_len = u16::from_be_bytes(self.read_several::<2>());
                    let str_bytes = self.read_bytes(str_len as usize);

                    let q = std::str::from_utf8(&str_bytes).unwrap();
                    let s = Box::new(HeapObject::String(q.to_owned()));
                    let pointer = Box::into_raw(s);

                    self.constants.push(pointer as u64);
                }
                op::CONST_END_FLAG => break,
                c => panic!("Unknown const flag: {:02x}", c),
            };
        }
        self.check_header("End of constants table");
    }

    fn read_bytes(&mut self, num: usize) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        for _ in 0..num {
            bytes.push(self.read_opcode());
        }
        bytes
    }

    fn read_opcode(&mut self) -> u8 {
        let byte = self.program[self.ip];
        self.ip += 1;
        byte
    }

    fn read_several<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes: [u8; N] = [0; N];
        for byte in bytes.iter_mut() {
            *byte = self.read_opcode();
        }
        bytes
    }
}
