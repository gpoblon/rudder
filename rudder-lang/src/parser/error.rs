// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use super::{PInput, Token};
use crate::{error::*, io::output::Backtrace};
use colored::Colorize;
use nom::{
    error::{ErrorKind, ParseError, VerboseError},
    Err, IResult, Offset,
};
use std::fmt;

/// Result for all parser
pub type PResult<'src, O> = IResult<PInput<'src>, O, PError<PInput<'src>>>;

/// This is the only error type that should be returned by the main parser.
/// It is an error that is suitable to give to the user as opposed to nom error that are suitable
/// for the developer.
/// Sub parsers may mix error types from nom or from finalerror
/// So this is a generir error type that must implement ParseError
#[derive(Debug, PartialEq, Clone)]
pub enum PErrorKind<I> {
    Nom(VerboseError<I>), // TODO remove this one
    #[cfg(test)]
    NomTest(String), // cannot be use outside of tests
    ExpectedKeyword(&'static str), // anywhere (keyword type)
    ExpectedToken(&'static str), // anywhere (expected token)
    InvalidEnumExpression, // in enum expression
    InvalidEscapeSequence, // in string definition
    InvalidFormat,        // in header
    InvalidName(I),       // in identifier expressions (type of expression)
    InvalidVariableReference, // during string interpolation
    NoMetadata,           // Temporary error, always caught, should not happen
    TomlError(I, usize, usize, String), // Error during toml parsing
    UnsupportedMetadata(I, &'static str), // metadata or comments are not supported everywhere (metadata key)
    UnterminatedDelimiter(I),             // after an opening delimiter (first delimiter)
    UnterminatedOrInvalid(I), // can't say whether a delimiter is missing or a statement format is invalid
    Unparsed(I),              // cannot be parsed
    Eof,                      // reached Eof
}

// This is the same thing as a closure (Fn() -> I) but I couldn't manage to cope with lifetime
#[derive(Debug, Clone)]
pub struct Context<I>
where
    I: fmt::Debug + Offset,
{
    pub extractor: fn(I, I) -> I,
    pub text: I,
    pub token: I,
}
#[derive(Debug, Clone)]
pub struct PError<I>
where
    I: fmt::Debug + Offset,
{
    pub context: Option<Context<I>>,
    pub kind: PErrorKind<I>,
    pub backtrace: Backtrace,
}

/// This trait must be implemented by the error type of a nom parser
/// Implement all method to differentiate between :
/// - nom VerboseError (stack errors)
/// - pure PError (keep last error)
impl<I: Clone> ParseError<I> for PError<I>
where
    I: fmt::Debug + Offset,
{
    /// creates an error from the input position and an [ErrorKind]
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        PError {
            context: None,
            kind: PErrorKind::Nom(VerboseError::from_error_kind(input, kind)),
            backtrace: Backtrace::empty(),
        }
    }

    /// combines an existing error with a new one created from the input
    /// position and an [ErrorKind]. This is useful when backtracking
    /// through a parse tree, accumulating error context on the way
    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        match other.kind {
            PErrorKind::Nom(e) => PError {
                context: other.context,
                kind: PErrorKind::Nom(VerboseError::append(input, kind, e)),
                backtrace: other.backtrace, // might be interesting to cumulate it
            },
            _ => other,
        }
    }

    /// creates an error from an input position and an expected character
    fn from_char(input: I, c: char) -> Self {
        PError {
            context: None,
            kind: PErrorKind::Nom(VerboseError::from_char(input, c)),
            backtrace: Backtrace::empty(),
        }
    }

    /// combines two existing error. This function is used to compare errors and return the one that went the farthest
    /// generated in various branches of [alt]
    fn or(self, other: Self) -> Self {
        match self.kind {
            PErrorKind::Nom(_) => other,
            _ => match (&self.context, &other.context) {
                (Some(first), Some(sec)) => {
                    if first.text.offset(&sec.text) < 0 {
                        return other;
                    }
                    self
                }
                (Some(_), None) => self,
                (None, Some(_)) => other,
                _ => self,
            },
        }
    }
}

/// Proper printing of errors.
impl<'src> fmt::Display for PError<PInput<'src>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match &self.kind {
            PErrorKind::Nom(e) => format!("Unprocessed parsing error: '{:#?}'.\nPlease fill a BUG with context on when this happened!", e),
            #[cfg(test)]
            PErrorKind::NomTest(msg) => format!("Testing only error message, this should never happen {}.\nPlease fill a BUG with context on when this happened!", msg),
            PErrorKind::ExpectedKeyword(s) => format!("The following keyword was expected: '{}'.", s.bright_magenta()),
            PErrorKind::ExpectedToken(s) => format!("The following token was expected '{}'.", s.bright_magenta()),
            PErrorKind::InvalidEnumExpression => "This enum expression is invalid".to_string(),
            PErrorKind::InvalidEscapeSequence => "This escape sequence cannot be used in a string".to_string(),
            PErrorKind::InvalidFormat => "Invalid header format, it must contain a single line '@format=x' where x is an integer. Shebang accepted.".to_string(),
            PErrorKind::InvalidName(i) => format!("The identifier is invalid in a {}.", i.fragment().bright_magenta()),
            PErrorKind::InvalidVariableReference => "This variable reference is invalid".to_string(),
            PErrorKind::NoMetadata => "Expecting metadata here".to_string(),
            PErrorKind::TomlError(i, line, col, e) => format!("Unable to parse metadata block at {}:{}:{}: '{}'", Token::from(*i).file().bright_yellow(), line, col, e),
            PErrorKind::UnsupportedMetadata(i, msg) => format!("{} do not support parsed comments or metadatas: '{}' found at {}", msg, i.fragment().bright_magenta(), Token::from(*i).position_str().bright_yellow()),
            PErrorKind::UnterminatedDelimiter(i) => format!("Missing closing delimiter for '{}'", i.fragment().bright_magenta()),
            PErrorKind::UnterminatedOrInvalid(i) => format!("Either an unexpected statement or no closing delimiter matching '{}'", i.fragment().bright_magenta()),
            PErrorKind::Unparsed(i) => format!("Could not parse the following: '{}'", i.fragment().bright_magenta()),
            PErrorKind::Eof => format!("Unexpectedly reached end of file"),
        };

        // simply removes superfluous line return (prettyfication)
        match &self.context {
            Some(ctx) => {
                let ctx = (ctx.extractor)(ctx.text, ctx.token);
                let context = ctx.fragment().trim_end_matches('\n');
                // Formats final error output
                f.write_str(&format!(
                    "{} near '{}'\n{} {}{:?}",
                    Token::from(ctx).position_str().bright_yellow(),
                    context,
                    "!-->".bright_blue(),
                    message.bold(),
                    self.backtrace
                ))
            }
            None => f.write_str(&format!(
                "{}\n{} {}{:?}",
                "undefined context".bright_yellow(),
                "!-->".bright_blue(),
                message.bold(),
                self.backtrace
            )),
        }
    }
}

/// Convert into a project error
impl Into<Error> for PError<PInput<'_>> {
    fn into(self) -> Error {
        Error::new(self.to_string())
    }
}

/// Combinator to force the error output to be a specific PError.
/// Having a combinator instead of a macro is much better since it can be used from within other
/// combinators.
// Here closures could use ParseError<I> but this way we force the type inference for
// all sub parsers to return PError<I>, which we won't have to specify during call.
// Pass a closure instead of a PErrorKind to allow lazy evaluation of its parameters
pub fn or_fail<'src, O, F, E>(f: F, e: E) -> impl Fn(PInput<'src>) -> PResult<'src, O>
where
    F: Fn(PInput<'src>) -> PResult<'src, O>,
    E: Fn() -> PErrorKind<PInput<'src>>,
{
    move |input| {
        match f(input) {
            // a non nom error cannot be superseded
            // keep original context when possible
            Err(Err::Failure(err)) => match err.kind {
                PErrorKind::Nom(_) => Err(Err::Failure(PError {
                    context: err.context,
                    kind: e(),
                    backtrace: Backtrace::new(),
                })),
                _ => Err(Err::Failure(err)),
            },
            Err(Err::Error(err)) => Err(Err::Failure(PError {
                context: None,
                kind: e(),
                backtrace: Backtrace::new(),
            })),
            res => res,
        }
    }
}

/// Similar code to `or_fail()` yet usage and behavior differ:
/// primarily it is non-terminating as it does not turn Errors into Failures
/// but rather gives it a PError context (E parameter) therefore updating the generic Nom::ErrorKind
pub fn or_err<'src, O, F, E>(f: F, e: E) -> impl Fn(PInput<'src>) -> PResult<'src, O>
where
    F: Fn(PInput<'src>) -> PResult<'src, O>,
    E: Fn() -> PErrorKind<PInput<'src>>,
{
    move |input| match f(input) {
        Err(Err::Error(err)) => Err(Err::Error(PError {
            context: err.context,
            kind: e(),
            backtrace: Backtrace::empty(),
        })),
        res => res,
    }
}

/// This function turns our own `Error`s (not nom ones) into `Failure`s so they are properly handled
/// by the `nom::multi::many0()` function which abstracts Errors, only breaking on failures which was an issue
pub fn or_fail_perr<'src, O, F>(f: F) -> impl Fn(PInput<'src>) -> PResult<'src, O>
where
    F: Fn(PInput<'src>) -> PResult<'src, O>,
{
    move |input| match f(input) {
        Err(Err::Error(e)) if e.kind != PErrorKind::Eof => Err(Err::Failure(e)),
        res => return res,
    }
}

/// Updates content of an error to fit and capture a better context
/// Solely exists for (w)sequence macro
pub fn update_error_context<'src>(
    e: Err<PError<PInput<'src>>>,
    new_ctx: Context<PInput<'src>>,
) -> Err<PError<PInput<'src>>> {
    match e {
        Err::Failure(err) => {
            let context = match err.context {
                None => Some(new_ctx),
                Some(context_to_keep) => Some(context_to_keep),
            };
            Err::Failure(PError {
                context,
                kind: err.kind,
                backtrace: err.backtrace,
            })
        }
        Err::Error(err) => {
            let context = match err.context {
                None => Some(new_ctx),
                Some(context_to_keep) => Some(context_to_keep),
            };
            Err::Error(PError {
                context,
                kind: err.kind,
                backtrace: err.backtrace,
            })
        }
        res => res,
    }
}

/// Extract error from a parsing result and transforms it to the project's global error type.
pub fn fix_error_type<T>(res: PResult<T>) -> Result<T> {
    match res {
        Ok((_, t)) => Ok(t),
        Err(Err::Failure(e)) => Err(e.into()),
        Err(Err::Error(e)) => Err(e.into()),
        Err(Err::Incomplete(_)) => panic!("Incomplete should never happen"),
    }
}
