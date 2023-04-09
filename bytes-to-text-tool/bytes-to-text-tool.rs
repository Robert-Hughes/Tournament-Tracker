// A tool to encode the WASM binary code into a text array of bytes, for inclusion in a javascript file.
// This was written to speed up build times, as the powershell code to do this was quite slow.
//
// Input is a binary file, output is a text file like this:
//
// var WASM_BYTES = new Uint8Array([0,97,....]);

use std::{process::ExitCode, io::{BufReader, BufWriter, Write}, fs::File};
use std::io::Read;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("Missing first arg");
    let output = args.next().expect("Missing second arg");

    let mut input = BufReader::new(File::open(input).expect("Failed to open input file"));
    let mut output = BufWriter::new(File::create(output).expect("Failed to create output file"));

    output.write(b"var WASM_BYTES = new Uint8Array([").expect("Error writing to output file");

    let mut buf = [0; 64];
    loop {
        match input.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for b in &buf[0..n] {
                    output.write(b.to_string().as_bytes()).expect("Error writing to output file");
                    output.write(b",").expect("Error writing to output file");
                }
            },
            Err(_) => panic!("Error reading input file"),
        }
    }

    output.write(b"]);").expect("Error writing to output file");

    ExitCode::SUCCESS
}