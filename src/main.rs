use std::path::PathBuf;
use std::io::Write;

use clap::Parser;
use wasmparser::{FunctionBody, Operator};

mod loader;

#[derive(Parser)]
struct Opts {
    //#[clap(short, long)]
    //output: PathBuf,
    wasm: PathBuf,
}

fn main() {
    let opts = Opts::parse();
    let wasm = std::fs::read(opts.wasm).unwrap();
    let module = loader::load(&wasm);
    let mut out = vec![];
    writeln!(&mut out, "CALL func_{}", module.entry).unwrap();
    writeln!(&mut out, "HALT").unwrap();
    for (index, func) in module.functions.into_iter().enumerate() {
        writeln!(&mut out, "func_{}:", index).unwrap();
        compile(&mut out, func.body);
    }
    std::io::stdout().write_all(&out).unwrap();
}

fn compile(out: &mut Vec<u8>, body: FunctionBody<'_>) {
    let operators = body.get_operators_reader().unwrap();
    for op in operators {
        let op = op.unwrap();
        match op {
            Operator::I32Const { value } => {
                let lower = value as u16;
                let upper = (value >> 16) as u16;
                writeln!(out, "  LD DE,{lower}").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
                writeln!(out, "  LD DE,{upper}").unwrap();
                writeln!(out, "  PUSH DE").unwrap();
            }
            Operator::I32Store8 { memarg } => {
                let offset = memarg.offset;
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  POP DE").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  POP IX").unwrap();
                writeln!(out, "  LD BC,{offset}").unwrap();
                writeln!(out, "  ADD IX,BC").unwrap();
                writeln!(out, "  LD (IX+0),E").unwrap();
            },
            Operator::End => {},
            op => unimplemented!("operator {:?} not implemented", op),
        }
    }
    writeln!(out, "  RET").unwrap();
}
