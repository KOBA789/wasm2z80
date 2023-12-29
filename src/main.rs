use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use wasmparser::{BlockType, FunctionBody, Operator};

mod loader;

#[derive(Parser)]
struct Opts {
    //#[clap(short, long)]
    //output: PathBuf,
    wasm: PathBuf,
}

struct Labeler {
    index: usize,
}

impl Labeler {
    fn new() -> Self {
        Self { index: 0 }
    }

    fn next(&mut self) -> Label {
        let index = self.index;
        self.index += 1;
        Label(index)
    }
}

struct Label(usize);
impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "label_{}", self.0)
    }
}

fn main() {
    let opts = Opts::parse();
    let wasm = std::fs::read(opts.wasm).unwrap();
    let module = loader::load(&wasm);
    let mut out = vec![];
    let mut labeler = Labeler::new();
    writeln!(&mut out, "CALL func_{}", module.entry).unwrap();
    writeln!(&mut out, "HALT").unwrap();
    for (index, func) in module.functions.into_iter().enumerate() {
        writeln!(&mut out, "func_{}:", index).unwrap();
        compile(&mut out, &mut labeler, func.body);
    }
    std::io::stdout().write_all(&out).unwrap();
}

fn compile(out: &mut Vec<u8>, labeler: &mut Labeler, body: FunctionBody<'_>) {
    let operators = body.get_operators_reader().unwrap();
    for op in operators {
        let op = op.unwrap();
        match op {
            Operator::I32Const { value } => {
                let lower = value as u16;
                let upper = (value >> 16) as u16;
                writeln!(out, "  ; i32.const").unwrap();
                writeln!(out, "  LD DE,{lower}").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
                writeln!(out, "  LD DE,{upper}").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
            }
            Operator::I32Store8 { memarg } => {
                let offset = memarg.offset;
                writeln!(out, "  ; i32.store8").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  LD BC,{offset}").unwrap();
                writeln!(out, "  ADD IX,BC").unwrap();
                writeln!(out, "  LD (IX+0),E").unwrap();
            }
            Operator::I32Load8U { memarg } => {
                let offset = memarg.offset;
                writeln!(out, "  ; i32.load8_u").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  LD BC,{offset}").unwrap();
                writeln!(out, "  ADD IX,BC").unwrap();
                writeln!(out, "  LD E,(IX+0)").unwrap();
                writeln!(out, "  LD D,0").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
                writeln!(out, "  LD E,0").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
            }
            Operator::I32Eqz => {
                let zero = labeler.next();
                let nonzero = labeler.next();
                writeln!(out, "  ; i32.eqz").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  LD A,D").unwrap();
                writeln!(out, "  OR E").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  OR D").unwrap();
                writeln!(out, "  OR E").unwrap();
                writeln!(out, "  JR Z,{zero}").unwrap();
                writeln!(out, "  LD DE,0").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
                writeln!(out, "  JR {nonzero}").unwrap();
                writeln!(out, "{zero}:").unwrap();
                writeln!(out, "  LD DE,1").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
                writeln!(out, "{nonzero}:").unwrap();
                writeln!(out, "  LD E,0").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
            }
            Operator::Br { relative_depth } => {
                assert_eq!(relative_depth, 0);
                writeln!(out, "  ; br").unwrap();
                writeln!(out, "  JP loop").unwrap();
            }
            Operator::BrIf { relative_depth } => {
                assert_eq!(relative_depth, 0);
                writeln!(out, "  ; br_if").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  LD A,D").unwrap();
                writeln!(out, "  OR E").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  OR D").unwrap();
                writeln!(out, "  OR E").unwrap();
                writeln!(out, "  JP NZ,loop").unwrap();
            }
            Operator::Loop { blockty } => {
                assert_eq!(blockty, BlockType::Empty);
                writeln!(out, "loop:").unwrap();
            }
            Operator::End => {}
            op => unimplemented!("operator {:?} not implemented", op),
        }
    }
    writeln!(out, "  RET").unwrap();
}
