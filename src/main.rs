use std::{io::Write, path::PathBuf};

use clap::Parser;

mod compile;
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
    module.compile(&mut out);
    std::io::stdout().write_all(&out).unwrap();
}
