use super::heap::HeapObject;
use super::metadata::{Metadata, MetadataBlock};
use super::opcodes::op;
use super::worker::ActiveObject;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

pub struct StoredActiveObject {
    // pub active_object: Arc<ActiveObject>,
    pub inbox: mpsc::Sender<Vec<u64>>,
}

pub struct Vm {
    ip: usize,

    pub program: Vec<u8>,
    pub constants: Vec<u64>,
    pub metadata: Metadata,
    pub entry: usize,

    gateways_for_active: mpsc::Sender<(u64, Vec<u64>)>,
    receiver: mpsc::Receiver<(u64, Vec<u64>)>,

    pub step_by_step: bool,
    pub show_debug: bool,

    active_objects: RwLock<Vec<StoredActiveObject>>,
}

unsafe impl Sync for Vm {}

impl Vm {
    pub fn setup(program: Vec<u8>, step_by_step: bool, show_debug: bool) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel();
        let mut new_vm = Self {
            ip: 0,
            program,
            constants: vec![],
            metadata: Metadata::default(),
            entry: 0,
            step_by_step,
            show_debug,
            active_objects: RwLock::new(vec![]),

            gateways_for_active: sender,
            receiver,
        };

        new_vm.check_header("Initial header");

        new_vm.load_consts();
        new_vm.load_metadata();
        new_vm.load_entry();
        Arc::new(new_vm)
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

    fn load_entry(&mut self) {
        self.entry = u16::from_be_bytes(self.read_several::<2>()) as usize;
        self.check_header("Entry loaded, start of functions");
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
        let mut string_repr: Vec<String> = vec![];

        loop {
            let const_type = self.read_opcode();
            match const_type {
                op::CONST_INT_FLAG => {
                    let i = i64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(i as u64);
                    string_repr.push(i.to_string());
                }
                op::CONST_FLOAT_FLAG => {
                    let f = u64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(f);
                    string_repr.push(f.to_string());
                }
                op::CONST_STRING_FLAG => {
                    let str_len = u16::from_be_bytes(self.read_several::<2>());
                    let str_bytes = self.read_bytes(str_len as usize);

                    let q = std::str::from_utf8(&str_bytes).unwrap();
                    let s = Box::new(HeapObject::String(q.to_owned()));
                    let pointer = Box::into_raw(s);
                    string_repr.push(format!("string {:x}: \"{}\"", pointer as u64, q));

                    self.constants.push(pointer as u64);
                }
                op::CONST_END_FLAG => break,
                c => panic!("Unknown const flag: {:02x}", c),
            };
        }
        self.check_header("End of constants table");
        if self.show_debug {
            println!("Loaded constants:");
            for (i, s) in string_repr.iter().enumerate() {
                println!("# {}  --  {}", i, s);
            }
        }
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

    pub fn spawn_new_active(vm: Arc<Vm>, item_type: usize, constructor_args: Vec<u64>) -> u64 {
        let item_size = vm.metadata.types_sizes[item_type];

        let active_index = vm.active_objects.read().unwrap().len() as u64;

        let mut active_object =
            ActiveObject::new(item_size, vm.clone(), vm.gateways_for_active.clone());

        let (send, recv) = mpsc::channel();
        send.send(constructor_args).unwrap();

        vm.active_objects
            .write()
            .unwrap()
            .push(StoredActiveObject { inbox: send });

        thread::spawn(move || loop {
            let msg = recv.recv().unwrap();
            active_object.run(msg);
        });

        active_index
    }

    pub fn setup_entry_and_run(vm: Arc<Vm>) {
        let mut active_object = ActiveObject::new(0, vm.clone(), vm.gateways_for_active.clone());
        active_object.run(vec![vm.entry as u64]);

        loop {
            let (target, msg) = vm.receiver.recv().unwrap();
            let sink = &vm.active_objects.read().unwrap()[target as usize];
            sink.inbox.send(msg).unwrap();
        }
    }
}

// pub fn spawn_worker(
//     vm: &'static Vm,
//     item_type: usize,
//     data: Vec<u64>,
// )  {
//     ACTIVE_SPAWNED.fetch_add(1, Ordering::SeqCst);
//     let item_size = vm.metadata.types_sizes[item_type];

//     let mut worker = ActiveObject::new(item_size, vm);

//     thread::spawn(move || {
//         worker.run(data);
//     })
// }

// pub fn run_entry_and_wait_if_spawned(vm: &Vm) {
//     let mut worker = ActiveObject::new(0, vm);
//     worker.run(vec![vm.entry as u64]);

//     loop {
//         let value = ACTIVE_SPAWNED.load(Ordering::SeqCst);
//         if value == 0 {
//             break;
//         }
//         println!("Waiting for {} spawned threads...", value);
//         thread::sleep(Duration::from_secs(1));
//     }
// }
