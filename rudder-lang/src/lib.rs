#[macro_use]
extern crate log;
#[macro_use]
pub mod error;
pub mod compile;
pub mod translate;
mod ast;
mod generators;
mod parser;