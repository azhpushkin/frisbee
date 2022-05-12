use crate::vm::opcodes::op;
use owo_colors::OwoColorize;
use std::collections::HashMap;

pub struct Disassembler<'a> {
    program_iter: Box<dyn Iterator<Item = (usize, &'a u8)> + 'a>,
    result: Vec<String>,
    function_names: HashMap<usize, String>,
    type_names: HashMap<usize, String>,
    list_type_names: HashMap<usize, String>,
}

impl<'a> Disassembler<'a> {
    pub fn new(program: &'a [u8]) -> Self {
        Disassembler {
            program_iter: Box::new(program.iter().enumerate()),
            result: vec![],
            function_names: HashMap::new(),
            type_names: HashMap::new(),
            list_type_names: HashMap::new(),
        }
    }

    pub fn disassemble(&mut self) -> String {
        self.result.clear();

        self.read_header("Initial");

        self.read_constants();
        self.read_header("End of constants");

        // Read types metadata
        for (i, type_name) in self.read_info_block() {
            self.type_names.insert(i, type_name);
        }
        self.read_header("End of types metadata");

        // Read lists metadata
        for (i, item_type) in self.read_info_block() {
            self.list_type_names.insert(i, item_type);
        }
        self.read_header("End of list types metadata");

        // Read functions metadata
        let mut function_names = vec![];
        for (_, fname) in self.read_info_block() {
            function_names.push(fname);
        }
        self.read_header("End of function metadata");

        for fname in function_names.into_iter() {
            let pos = u16::from_be_bytes(self.get_bytes::<2>());
            self.function_names.insert(pos as usize, fname);
        }
        self.read_header("End of function positions");

        self.read_entry();
        self.read_header("Start of functions");

        self.read_functions();

        self.result.join("\n")
    }

    fn get_byte(&mut self) -> (usize, u8) {
        let (i, byte) = self.program_iter.next().unwrap();
        (i, *byte)
    }

    fn get_str(&mut self) -> String {
        let n = u16::from_be_bytes(self.get_bytes::<2>());
        let mut s = String::new();
        for _ in 0..n {
            s.push(self.get_byte().1 as char);
        }
        s
    }

    fn get_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0; N];
        for byte in bytes.iter_mut() {
            *byte = self.get_byte().1;
        }
        bytes
    }

    fn read_header(&mut self, header_name: &str) {
        let header = self.get_bytes::<2>();
        self.result.push(format!(
            "HEADER [{}] => {:02x?}",
            header_name,
            header.yellow()
        ));
    }

    fn read_constants(&mut self) {
        self.result.push("Constants table:".to_string());
        let mut i: usize = 0;

        loop {
            let const_text: String = match self.get_byte().1 {
                op::CONST_INT_FLAG => i64::from_be_bytes(self.get_bytes::<8>()).to_string(),
                op::CONST_FLOAT_FLAG => f64::from_be_bytes(self.get_bytes::<8>()).to_string(),
                op::CONST_STRING_FLAG => format!("\"{}\"", self.get_str()),
                op::CONST_END_FLAG => {
                    break;
                }
                c => panic!("Unknown const flag: {:02x}", c),
            };
            self.result.push(format!("   {} -> {}", i, const_text));
            i += 1;
        }
    }

    fn read_info_block(&mut self) -> Vec<(usize, String)> {
        let amount = self.get_byte().1;
        let mut res = vec![];
        for _ in 0..amount {
            let name = self.get_str();

            // Skip through sizes + pointer mappings as they are not needed for disassembly
            self.get_bytes::<2>();
            for _ in 0..self.get_byte().1 {
                self.get_byte();
            }

            res.push(name);
        }
        res.into_iter().enumerate().collect()
    }

    fn read_entry(&mut self) {
        let entry = self.get_bytes::<2>();
        let entry_name = &self.function_names[&(u16::from_be_bytes(entry) as usize)];
        self.result
            .push(format!("Entry point:\n   -> {} {:02x?}", entry_name, entry));
    }

    fn read_functions(&mut self) {
        while let Some((i, opcode)) = self.program_iter.next() {
            if let Some(name) = self.function_names.get(&i) {
                self.result
                    .push(format!("\n## Function {}", name).green().bold().to_string());
            }

            let mut number_of_args = op::get_args_num(*opcode);
            let mut args: Vec<u8> = vec![];
            while number_of_args > 0 {
                args.push(self.get_byte().1);
                number_of_args -= 1;
            }
            let mut op_text = format!(
                " {:>4x?}  {} {:02x?}",
                i.blue(),
                op::get_display_name(*opcode),
                args
            );

            // + 3 is added, because jump offset is relative to instruction pointer
            // but `i` var points to jump opcode, which is 3 steps behind
            // (1 step for jump itself and 2 for address of jump)
            if *opcode == op::JUMP_IF_FALSE || *opcode == op::JUMP {
                let x = u16::from_be_bytes([args[0], args[1]]) as usize;
                op_text = format!("{} (jumps to {:02x?}) ", op_text, x + i + 3);
            } else if *opcode == op::JUMP_BACK {
                let x = u16::from_be_bytes([args[0], args[1]]) as usize;
                op_text = format!("{} (jumps to {:02x?}) ", op_text, i - x + 3);
            } else if *opcode == op::ALLOCATE {
                let typename = &self.type_names[&(args[0] as usize)];
                op_text.push_str(&format!(" (type {}) ", typename).yellow().to_string());
            } else if *opcode == op::ALLOCATE_LIST {
                let typename = &self.list_type_names[&(args[0] as usize)];
                op_text.push_str(&format!(" (list of {}) ", typename).yellow().to_string());
            }

            self.result.push(op_text);
        }
    }
}
