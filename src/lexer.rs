/// This module contains the lexer for Kaleidoscope.

/// All the different tokens that the lexer can return.
///
// Using Rust enums instead of integers is much safer and more readable.
pub enum Token {
    /// End of file
    Eof,
    // Commands
    Define,
    Extern,
    /// An Identifier contains the identifier as a String.
    /// This is much safer and easier to manage than using global variables.
    Identifier(String),
    /// All numbers in Kaleidoscope are 64 bit floats.
    /// We store the number in the variant istead of in a global variable
    /// for the same reasons as Identifier.
    Number(f64),
    /// UnknownChar corresponds to returning a positive integer from gettok.
    UnknownChar(char),
}
