// We are using the llvm-alt crate to interface with LLVM
extern crate llvm;
extern crate llvm_sys;
// The lexer module was written in chapter 1.
pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod jit;
