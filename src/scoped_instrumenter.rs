use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use lazy_static::lazy_static;

use crate::isa::Instruction;

const TARGET_ENTRY_COUNT: usize = 10_000;

lazy_static! {
    static ref INSTRUMENTATION_FILES: Mutex<HashMap<&'static str, InstrumentationFile>> =
        Mutex::new(HashMap::new());
    static ref OUTPUT_DIR: PathBuf = {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("metrics");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("should create {}: {:?}", dir.display(), error));
        dir
    };
}

#[derive(Debug)]
struct InstrumentationFile {
    path: PathBuf,
    file: File,
    entry_count: usize,
}

impl InstrumentationFile {
    fn new(instruction_name: &str) -> Self {
        let path = OUTPUT_DIR.join(format!("{}.csv", instruction_name));

        match fs::read_to_string(&path) {
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    let mut file = File::create(&path)
                        .unwrap_or_else(|_| panic!("should create {}", path.display()));
                    writeln!(file, "args,n_exec,total_elapsed_time")
                        .unwrap_or_else(|_| panic!("should write to {}", path.display()));
                    InstrumentationFile {
                        path,
                        file,
                        entry_count: 0,
                    }
                } else {
                    panic!("failed to read {}: {:?}", path.display(), error);
                }
            }
            Ok(contents) => {
                let file = OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .unwrap_or_else(|_| panic!("should open {}", path.display()));
                let entry_count = contents.lines().count() - 1; // discount the header line
                InstrumentationFile {
                    path,
                    file,
                    entry_count,
                }
            }
        }
    }

    fn instrument(&mut self, duration: Duration, properties: &[String]) {
        if self.entry_count < TARGET_ENTRY_COUNT {
            write!(self.file, "\"(")
                .unwrap_or_else(|_| panic!("should write to {}", self.path.display()));
            for value in properties {
                write!(self.file, "{},", value)
                    .unwrap_or_else(|_| panic!("should write to {}", self.path.display()));
            }
            writeln!(self.file, ")\",1,{:.06e}", duration.as_secs_f64())
                .unwrap_or_else(|_| panic!("should write to {}", self.path.display()));
            self.entry_count += 1;
        }
    }
}

pub(super) struct ScopedInstrumenter {
    start: Instant,
    instruction: &'static str,
    properties: Vec<String>,
}

impl ScopedInstrumenter {
    pub fn new(instruction: &Instruction) -> Option<Self> {
        match instruction {
            Instruction::F32Load(_)
            | Instruction::F64Load(_)
            | Instruction::F32Store(_)
            | Instruction::F64Store(_)
            | Instruction::F32Const(_)
            | Instruction::F64Const(_)
            | Instruction::F32Eq
            | Instruction::F32Ne
            | Instruction::F32Lt
            | Instruction::F32Gt
            | Instruction::F32Le
            | Instruction::F32Ge
            | Instruction::F64Eq
            | Instruction::F64Ne
            | Instruction::F64Lt
            | Instruction::F64Gt
            | Instruction::F64Le
            | Instruction::F64Ge
            | Instruction::F32Abs
            | Instruction::F32Neg
            | Instruction::F32Ceil
            | Instruction::F32Floor
            | Instruction::F32Trunc
            | Instruction::F32Nearest
            | Instruction::F32Sqrt
            | Instruction::F32Add
            | Instruction::F32Sub
            | Instruction::F32Mul
            | Instruction::F32Div
            | Instruction::F32Min
            | Instruction::F32Max
            | Instruction::F32Copysign
            | Instruction::F64Abs
            | Instruction::F64Neg
            | Instruction::F64Ceil
            | Instruction::F64Floor
            | Instruction::F64Trunc
            | Instruction::F64Nearest
            | Instruction::F64Sqrt
            | Instruction::F64Add
            | Instruction::F64Sub
            | Instruction::F64Mul
            | Instruction::F64Div
            | Instruction::F64Min
            | Instruction::F64Max
            | Instruction::F64Copysign
            | Instruction::I32TruncSF32
            | Instruction::I32TruncUF32
            | Instruction::I32TruncSF64
            | Instruction::I32TruncUF64
            | Instruction::I64TruncSF32
            | Instruction::I64TruncUF32
            | Instruction::I64TruncSF64
            | Instruction::I64TruncUF64
            | Instruction::F32ConvertSI32
            | Instruction::F32ConvertUI32
            | Instruction::F32ConvertSI64
            | Instruction::F32ConvertUI64
            | Instruction::F32DemoteF64
            | Instruction::F64ConvertSI32
            | Instruction::F64ConvertUI32
            | Instruction::F64ConvertSI64
            | Instruction::F64ConvertUI64
            | Instruction::F64PromoteF32
            | Instruction::I32ReinterpretF32
            | Instruction::I64ReinterpretF64
            | Instruction::F32ReinterpretI32
            | Instruction::F64ReinterpretI64 => return None,
            _ => (),
        };

        let instruction_str = match instruction {
            Instruction::Unreachable => "Unreachable",
            Instruction::GetLocal(_) => "GetLocal",
            Instruction::SetLocal(_) => "SetLocal",
            Instruction::TeeLocal(_) => "TeeLocal",
            Instruction::Br(_) => "Br",
            Instruction::BrIfEqz(_) => "BrIfEqz",
            Instruction::BrIfNez(_) => "BrIfNez",
            Instruction::BrTable(_) => "BrTable",
            Instruction::Return(_) => "Return",
            Instruction::Call(_) => "Call",
            Instruction::CallIndirect(_) => "CallIndirect",
            Instruction::Drop => "Drop",
            Instruction::Select => "Select",
            Instruction::GetGlobal(_) => "GetGlobal",
            Instruction::SetGlobal(_) => "SetGlobal",
            Instruction::I32Load(_) => "I32Load",
            Instruction::I64Load(_) => "I64Load",
            Instruction::F32Load(_) => "F32Load",
            Instruction::F64Load(_) => "F64Load",
            Instruction::I32Load8S(_) => "I32Load8S",
            Instruction::I32Load8U(_) => "I32Load8U",
            Instruction::I32Load16S(_) => "I32Load16S",
            Instruction::I32Load16U(_) => "I32Load16U",
            Instruction::I64Load8S(_) => "I64Load8S",
            Instruction::I64Load8U(_) => "I64Load8U",
            Instruction::I64Load16S(_) => "I64Load16S",
            Instruction::I64Load16U(_) => "I64Load16U",
            Instruction::I64Load32S(_) => "I64Load32S",
            Instruction::I64Load32U(_) => "I64Load32U",
            Instruction::I32Store(_) => "I32Store",
            Instruction::I64Store(_) => "I64Store",
            Instruction::F32Store(_) => "F32Store",
            Instruction::F64Store(_) => "F64Store",
            Instruction::I32Store8(_) => "I32Store8",
            Instruction::I32Store16(_) => "I32Store16",
            Instruction::I64Store8(_) => "I64Store8",
            Instruction::I64Store16(_) => "I64Store16",
            Instruction::I64Store32(_) => "I64Store32",
            Instruction::CurrentMemory => "CurrentMemory",
            Instruction::GrowMemory => "GrowMemory",
            Instruction::I32Const(_) => "I32Const",
            Instruction::I64Const(_) => "I64Const",
            Instruction::F32Const(_) => "F32Const",
            Instruction::F64Const(_) => "F64Const",
            Instruction::I32Eqz => "I32Eqz",
            Instruction::I32Eq => "I32Eq",
            Instruction::I32Ne => "I32Ne",
            Instruction::I32LtS => "I32LtS",
            Instruction::I32LtU => "I32LtU",
            Instruction::I32GtS => "I32GtS",
            Instruction::I32GtU => "I32GtU",
            Instruction::I32LeS => "I32LeS",
            Instruction::I32LeU => "I32LeU",
            Instruction::I32GeS => "I32GeS",
            Instruction::I32GeU => "I32GeU",
            Instruction::I64Eqz => "I64Eqz",
            Instruction::I64Eq => "I64Eq",
            Instruction::I64Ne => "I64Ne",
            Instruction::I64LtS => "I64LtS",
            Instruction::I64LtU => "I64LtU",
            Instruction::I64GtS => "I64GtS",
            Instruction::I64GtU => "I64GtU",
            Instruction::I64LeS => "I64LeS",
            Instruction::I64LeU => "I64LeU",
            Instruction::I64GeS => "I64GeS",
            Instruction::I64GeU => "I64GeU",
            Instruction::F32Eq => "F32Eq",
            Instruction::F32Ne => "F32Ne",
            Instruction::F32Lt => "F32Lt",
            Instruction::F32Gt => "F32Gt",
            Instruction::F32Le => "F32Le",
            Instruction::F32Ge => "F32Ge",
            Instruction::F64Eq => "F64Eq",
            Instruction::F64Ne => "F64Ne",
            Instruction::F64Lt => "F64Lt",
            Instruction::F64Gt => "F64Gt",
            Instruction::F64Le => "F64Le",
            Instruction::F64Ge => "F64Ge",
            Instruction::I32Clz => "I32Clz",
            Instruction::I32Ctz => "I32Ctz",
            Instruction::I32Popcnt => "I32Popcnt",
            Instruction::I32Add => "I32Add",
            Instruction::I32Sub => "I32Sub",
            Instruction::I32Mul => "I32Mul",
            Instruction::I32DivS => "I32DivS",
            Instruction::I32DivU => "I32DivU",
            Instruction::I32RemS => "I32RemS",
            Instruction::I32RemU => "I32RemU",
            Instruction::I32And => "I32And",
            Instruction::I32Or => "I32Or",
            Instruction::I32Xor => "I32Xor",
            Instruction::I32Shl => "I32Shl",
            Instruction::I32ShrS => "I32ShrS",
            Instruction::I32ShrU => "I32ShrU",
            Instruction::I32Rotl => "I32Rotl",
            Instruction::I32Rotr => "I32Rotr",
            Instruction::I64Clz => "I64Clz",
            Instruction::I64Ctz => "I64Ctz",
            Instruction::I64Popcnt => "I64Popcnt",
            Instruction::I64Add => "I64Add",
            Instruction::I64Sub => "I64Sub",
            Instruction::I64Mul => "I64Mul",
            Instruction::I64DivS => "I64DivS",
            Instruction::I64DivU => "I64DivU",
            Instruction::I64RemS => "I64RemS",
            Instruction::I64RemU => "I64RemU",
            Instruction::I64And => "I64And",
            Instruction::I64Or => "I64Or",
            Instruction::I64Xor => "I64Xor",
            Instruction::I64Shl => "I64Shl",
            Instruction::I64ShrS => "I64ShrS",
            Instruction::I64ShrU => "I64ShrU",
            Instruction::I64Rotl => "I64Rotl",
            Instruction::I64Rotr => "I64Rotr",
            Instruction::F32Abs => "F32Abs",
            Instruction::F32Neg => "F32Neg",
            Instruction::F32Ceil => "F32Ceil",
            Instruction::F32Floor => "F32Floor",
            Instruction::F32Trunc => "F32Trunc",
            Instruction::F32Nearest => "F32Nearest",
            Instruction::F32Sqrt => "F32Sqrt",
            Instruction::F32Add => "F32Add",
            Instruction::F32Sub => "F32Sub",
            Instruction::F32Mul => "F32Mul",
            Instruction::F32Div => "F32Div",
            Instruction::F32Min => "F32Min",
            Instruction::F32Max => "F32Max",
            Instruction::F32Copysign => "F32Copysign",
            Instruction::F64Abs => "F64Abs",
            Instruction::F64Neg => "F64Neg",
            Instruction::F64Ceil => "F64Ceil",
            Instruction::F64Floor => "F64Floor",
            Instruction::F64Trunc => "F64Trunc",
            Instruction::F64Nearest => "F64Nearest",
            Instruction::F64Sqrt => "F64Sqrt",
            Instruction::F64Add => "F64Add",
            Instruction::F64Sub => "F64Sub",
            Instruction::F64Mul => "F64Mul",
            Instruction::F64Div => "F64Div",
            Instruction::F64Min => "F64Min",
            Instruction::F64Max => "F64Max",
            Instruction::F64Copysign => "F64Copysign",
            Instruction::I32WrapI64 => "I32WrapI64",
            Instruction::I32TruncSF32 => "I32TruncSF32",
            Instruction::I32TruncUF32 => "I32TruncUF32",
            Instruction::I32TruncSF64 => "I32TruncSF64",
            Instruction::I32TruncUF64 => "I32TruncUF64",
            Instruction::I64ExtendSI32 => "I64ExtendSI32",
            Instruction::I64ExtendUI32 => "I64ExtendUI32",
            Instruction::I64TruncSF32 => "I64TruncSF32",
            Instruction::I64TruncUF32 => "I64TruncUF32",
            Instruction::I64TruncSF64 => "I64TruncSF64",
            Instruction::I64TruncUF64 => "I64TruncUF64",
            Instruction::F32ConvertSI32 => "F32ConvertSI32",
            Instruction::F32ConvertUI32 => "F32ConvertUI32",
            Instruction::F32ConvertSI64 => "F32ConvertSI64",
            Instruction::F32ConvertUI64 => "F32ConvertUI64",
            Instruction::F32DemoteF64 => "F32DemoteF64",
            Instruction::F64ConvertSI32 => "F64ConvertSI32",
            Instruction::F64ConvertUI32 => "F64ConvertUI32",
            Instruction::F64ConvertSI64 => "F64ConvertSI64",
            Instruction::F64ConvertUI64 => "F64ConvertUI64",
            Instruction::F64PromoteF32 => "F64PromoteF32",
            Instruction::I32ReinterpretF32 => "I32ReinterpretF32",
            Instruction::I64ReinterpretF64 => "I64ReinterpretF64",
            Instruction::F32ReinterpretI32 => "F32ReinterpretI32",
            Instruction::F64ReinterpretI64 => "F64ReinterpretI64",
        };

        let mut properties = Vec::new();
        match instruction {
            Instruction::Br(target)
            | Instruction::BrIfEqz(target)
            | Instruction::BrIfNez(target) => {
                properties.push(target.drop_keep.keep.count().to_string())
            }
            Instruction::Return(drop_keep) => properties.push(drop_keep.keep.count().to_string()),
            _ => (),
        };

        Some(ScopedInstrumenter {
            start: Instant::now(),
            instruction: instruction_str,
            properties,
        })
    }
}

impl Drop for ScopedInstrumenter {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        INSTRUMENTATION_FILES
            .lock()
            .unwrap()
            .entry(self.instruction)
            .or_insert_with(|| InstrumentationFile::new(self.instruction))
            .instrument(duration, &self.properties);
    }
}
