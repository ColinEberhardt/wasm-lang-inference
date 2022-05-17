# wasm-language-inference

Exploring techniques that may allow us to infer the language used to write WebAssembly modules for the [2022 Web Almanac WebAssembly chapter](https://github.com/HTTPArchive/almanac.httparchive.org/issues/2885).

NOTE: The wasm module analyser used by the HTTP Archive crawler is [found in the wasm-stats repo](https://github.com/HTTPArchive/wasm-stats). The purpose of this project is to rapid-prototype various approaches. If it is considered viable, it will be folded into `wasm-stats`.

## Usage

### Data download

First most recent catalogue of wasm modules is found in `downloader/wasm-urls-April-2022.csv`. This can be used to download the wasm modules as follows:

~~~
% mkdir wasm
% cd downloader
% npm i
% node index.mjs
====---------------------------
~~~

This will download ~1200 unique wasm modules, with filenames based on their hashed contents. It also writes a file `results.csv` which maps URLs to the hashed filenames

### Analysis

The analysis is performed by a simple Rust application, which mostly just interrogates exports / imports.

~~~
% cargo run
[...]
{Rust: 155, UnknownCompressed: 405, AssemblyScript: 140, Go: 30, Emscripten: 407, Unknown: 72}
6% unclassified
~~~