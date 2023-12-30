use wasmparser::{Export, FuncType, FunctionBody, Payload, RecGroup, SectionLimited};

use crate::compile::{Module, FunctionDef};

struct FunctionDecl {
    typ: FuncType,
}

#[derive(Default)]
struct ModuleBuilder<'a> {
    types: Vec<FuncType>,
    func_decls: Vec<FunctionDecl>,
    functions: Vec<FunctionDef<'a>>,
    entry: Option<usize>,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_types(&mut self, types: SectionLimited<'_, RecGroup>) {
        self.types = types
            .into_iter_err_on_gc_types()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    }

    pub fn add_funcs(&mut self, funcs: SectionLimited<'_, u32>) {
        self.func_decls = funcs
            .into_iter()
            .map(|idx| {
                idx.map(|idx| FunctionDecl {
                    typ: self.types[idx as usize].clone(),
                })
            })
            .collect::<Result<_, _>>()
            .unwrap();
    }

    pub fn add_code(&mut self, body: FunctionBody<'a>) {
        let typ = self.func_decls[self.functions.len()].typ.clone();
        self.functions.push(FunctionDef { func_type: typ, body });
    }

    pub fn add_exports(&mut self, exports: SectionLimited<'_, Export<'_>>) {
        self.entry = exports
            .into_iter()
            .filter_map(|exp| exp.ok())
            .filter_map(|exp| {
                if exp.kind == wasmparser::ExternalKind::Func && exp.name == "entry" {
                    Some(exp.index as usize)
                } else {
                    None
                }
            })
            .next();
    }

    pub fn build(self) -> Module<'a> {
        Module {
            entry: self.entry.unwrap(),
            functions: self.functions,
        }
    }
}

pub fn load(data: &[u8]) -> Module {
    let parser = wasmparser::Parser::new(0);
    let mut builder = ModuleBuilder::new();
    for payload in parser.parse_all(data) {
        let payload = payload.unwrap();
        match payload {
            Payload::End(_) => break,
            Payload::TypeSection(types) => {
                builder.add_types(types);
            }
            Payload::FunctionSection(funcs) => {
                builder.add_funcs(funcs);
            }
            Payload::ExportSection(exports) => builder.add_exports(exports),
            Payload::CodeSectionEntry(body) => {
                builder.add_code(body);
            }
            Payload::DataSection(_data) => {
                todo!();
            }
            Payload::CustomSection(_)
            | Payload::Version { .. }
            | Payload::MemorySection(_)
            | Payload::GlobalSection(_)
            | Payload::CodeSectionStart { .. } => { /* ignore */ }
            payload => {
                panic!("{:?}", payload);
            }
        }
    }
    builder.build()
}
