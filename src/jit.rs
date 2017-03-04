use std::io::{Read, Write, stdin, stdout};
use llvm::*;
use llvm::Attribute::*;
use llvm::Function;
use parser;
use lexer::Token;
use codegen;

pub fn run(opt_level: usize) {
    let context = Context::new();
    let module = Module::new("my jit", &context);
    let engine = JitEngine::new(&module, JitOptions {
        opt_level: opt_level,
    }).unwrap();
    loop {
        let builder = Builder::new(&context);
        let mut input = String::new();
        print!("> ", );
        stdout().flush();
        match stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(_) => break,
        }
        if input.trim_left() == "" {
            continue;
        }
        if input == "exit\n" {
            break;
        }
        let mut parser = parser::Parser::from_source(&input);
        match parser.current {
            Some(Token::Define) => {
                let func = match parser.parse_definition() {
                    Ok(func) => func,
                    Err(e) => {
                        println!("Error parsing definition: {}", e);
                        continue;
                    }
                };
                codegen::generate_function(&func, &builder, &module, &context).unwrap();
            },
            Some(Token::Extern) => {
                let proto = match parser.parse_extern() {
                    Ok(proto) => proto,
                    Err(e) => {
                        println!("Error parsing extern: {}", e);
                        continue;
                    }
                };
                codegen::generate_prototype(&proto, &module, &context);
            },
            // Top level expression
            _ => {
                let expr = parser.parse_top_level_expr().unwrap();
                let new_module = module.clone();
                let func = codegen::generate_function(&expr, &builder, &new_module, &context).unwrap();
                engine.add_module(&new_module);
                let res = engine.run_function(&func, &[]);
                println!("{}", f64::from_generic(&res, &context));
                engine.remove_module(&new_module);
            }
        }
    }
}
