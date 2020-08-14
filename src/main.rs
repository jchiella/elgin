#[macro_use]
extern crate lazy_static;

mod errors;
mod types;
mod analysis;
mod ir;
mod lexer;
mod llvm;
mod parser;

use std::io::prelude::*;
use std::{env, fs};

fn main() {
    if let Some(_) = env::args().nth(1) {
        file();
    } else {
        panic!("Expected File")
    }
}

fn file() {
    let mut file = fs::File::open(env::args().nth(1).unwrap()).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let chars = &input.chars().collect::<Vec<_>>()[..];

    let mut lexer = lexer::Lexer::new(chars);
    let lex_results_option = lexer.go();
    println!("______________________");
    println!("lex errors:");
    println!("{:#?}", errors::ERRORS.lock().unwrap());
    let lex_results = lex_results_option.unwrap();
    println!("______________________");
    println!("lexer output:");
    lex_results.iter().map(|t| println!("{:?}", t)).for_each(drop);

    let mut parser = parser::Parser::new(&lex_results);
    let parse_results = parser.go();
    println!("______________________");
    println!("parse errors:");
    println!("{:#?}", errors::ERRORS.lock().unwrap());
    println!("______________________");
    println!("parser output:");
    println!("{:#?}", parse_results);

    let unwrapped = parse_results.unwrap();
    let mut irbuilder = ir::IRBuilder::new(&unwrapped);
    let ir_results = irbuilder.go();
    println!("______________________");
    println!("IR gen errors:");
    println!("{:#?}", errors::ERRORS.lock().unwrap());
    println!("______________________");
    println!("IR output:");
    println!("{:#?}", *ir_results.unwrap());

    println!("______________________");
    println!("analysis output:");
    let analysis_option = irbuilder.analyze();
    println!("______________________");
    println!("analysis errors:");
    println!("{:#?}", errors::ERRORS.lock().unwrap());
    analysis_option.unwrap();

    let mut generator = llvm::Generator::new(&irbuilder.procs, "chi", &env::args().nth(1).unwrap());
    generator.go();
    println!("______________________");
    println!("codegen output:");
    println!("Dumping to file...");
    let mut file_name = env::args().nth(1).unwrap();
    file_name.push_str(".ll");
    generator.dump_to_file(&file_name);
    println!("File done!");

    println!("______________________");
    println!("Errors:");
    println!("{:#?}", errors::ERRORS.lock().unwrap());
}
