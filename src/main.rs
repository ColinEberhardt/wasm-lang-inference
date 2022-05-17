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
    UnknownCompressedOne,
    UnknownCompressedTwo,
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

fn is_likely_emscripten(module: &WasmModule) -> bool {
    // Many of the wasm modules have been compressed with this very distinctive pattern. From looking at a number of wasm modules
    // and inspecting their contents, or the page that hosts them, it seems quite likely this is Emscripten. For example:
    //
    // https://tweet2doom.github.io/t2d-explorer.wasm
    //   => https://github.com/tweet2doom/tweet2doom.github.io - strong evidence of Emscripten
    //
    // https://graphonline.ru/script/Graphoffline.Emscripten.wasm - the clue is in the filename!
    //
    // https://wsr-starfinder.com/js/stellarium-web-engine.06229ae9.wasm
    //  => https://github.com/Stellarium/stellarium-web-engine - code makes reference to using Emscripten
    (module.any_imports_match(|i| i.module == "a" && i.name == "a")
        && module.any_imports_match(|i| i.module == "a" && i.name == "b"))

    // another distinctive pattern, again, evidence suggests Emscripten
    // https://tx.me/
    // => https://github.com/Samsung/rlottie/blob/master/src/wasm/rlottiewasm.cpp - this is a cool project ;-)
    //
    // https://demo.harmonicvision.com - Emscripten mentioned in the page source
    //
    // https://webcamera.io - uses FFMpeg, which is an Emscripten project
    || (module.any_imports_match(|i| i.module == "env" && i.name == "a")
        && module.any_imports_match(|i| i.module == "env" && i.name == "b"))

    // exporting malloc is a C giveaway!
    || module.any_exports_match(|e| e.name == "malloc")

    // standard memory management functions
    || module.any_imports_match(|i| i.module == "env" && i.name == "__memory_base")
}

fn is_rust(module: &WasmModule) -> bool {
    module.any_imports_match(|i| {
        i.name.to_string().contains("wbindgen")
            || i.name.to_string().contains("wbg")
            || i.module == "wbg"
            || i.module == "wbindgen"
    }) || module.any_exports_match(|e| e.name.to_string().contains("wbindgen"))
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
        // OK, so this one is *very* hacky! The hyphenate lib (https://github.com/mnater/Hyphenopoly) is found on a number of
        // websites. It is written in AssemblyScript, and has a variety of different bundles. They all export the function 
        // 'hyphenate'. 
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
    if is_likely_emscripten(&module) {
        return Language::Emscripten;
    }

    // Unknown modules
    // 2735d1055ef617dbb1e84cdfa8eb5a9c05f50201a7aa8c06d44533166124fec6.wasm => https://tikzjax.com / webjs / Pascal

    // b8ea049ced002e39f3e32203c3d08f2efa964437887c92c39dd22e50945d7438.wasm => https://github.com/gasman/jsspeccy3 / AssemblyScript
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
