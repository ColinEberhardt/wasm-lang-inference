use std::collections::HashMap;
use std::fs;
use wasmparser::{Export, Import, Parser, Payload};

#[derive(Eq, PartialEq, Hash, Debug)]
enum Language {
    Rust,
    Emscripten,
    AssemblyScript,
    Blazor,
    Unknown,
    UnknownCompressed,
    Go,
}

struct WasmModule<'a> {
    imports: Vec<Import<'a>>,
    exports: Vec<Export<'a>>,
}

impl WasmModule<'_> {
    fn any_imports_match<F: Fn(&Import) -> bool>(&self, f: F) -> bool {
        self.imports.iter().any(|i| f(i))
    }

    fn any_exports_match<F: Fn(&Export) -> bool>(&self, f: F) -> bool {
        self.exports.iter().any(|i| f(i))
    }
}

fn parse_wasm(buf: &Vec<u8>) -> WasmModule {
    let mut imports = vec![];
    let mut exports = vec![];

    for payload in Parser::new(0).parse_all(buf) {
        match payload.unwrap() {
            Payload::ImportSection(s) => {
                for import in s {
                    imports.push(import.unwrap());
                }
            }
            Payload::ExportSection(s) => {
                for export in s {
                    exports.push(export.unwrap());
                }
            }
            _ => {}
        }
    }

    WasmModule { imports, exports }
}

fn is_emscripten(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.name.to_string().contains("emscripten"))
}

fn is_compressed(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.module == "a" || i.name == "a")
        || module.any_imports_match(|i| i.module == "a" || i.name == "b")
        || module.any_imports_match(|i| i.module == "a" || i.name == "c")
}

fn is_rust(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.name.to_string().contains("wbindgen") || i.module == "wbg")
        || module.any_exports_match(|e| e.name.to_string().contains("wbindgen"))
}

fn is_blazor(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.name.to_string().contains("mono"))
}

fn is_go(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.module == "go")
        || module.any_imports_match(|i| i.name.to_string().contains("go"))
        || module.any_imports_match(|e| e.name.to_string().contains("go_scheduler"))
}

fn is_assemblyscript(module: &WasmModule) -> bool {
    module.any_imports_match(|i| i.module == "env" && i.name == "abort")
        || module.any_exports_match(|e| e.name == "hyphenate")
}

fn infer_language(buf: &Vec<u8>) -> Language {
    let module = parse_wasm(buf);

    if is_emscripten(&module) {
        return Language::Emscripten;
    }
    if is_blazor(&module) {
        return Language::Blazor;
    }
    if is_rust(&module) {
        return Language::Rust;
    }
    if is_go(&module) {
        return Language::Go;
    }
    if is_assemblyscript(&module) {
        return Language::AssemblyScript;
    }
    if is_compressed(&module) {
        return Language::UnknownCompressed;
    }
    return Language::Unknown;
}

fn main() -> () {
    let paths = fs::read_dir("./wasm").unwrap();

    let mut langs = vec![];

    for path in paths {
        let f = path.unwrap();
        let buf: Vec<u8> = fs::read(f.path()).unwrap();
        let lang = infer_language(&buf);
        println!("{:?}, {}", lang, f.path().display());
        langs.push(lang);
    }

    let mut counts = HashMap::new();
    langs.iter().for_each(|val| {
        counts
            .entry(val)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    });
    println!();
    println!("{counts:?}");
    println!(
        "{:.0}% unclassified",
        *counts.get(&Language::Unknown).unwrap() as f32 * 100.0 / langs.len() as f32
    );
}
