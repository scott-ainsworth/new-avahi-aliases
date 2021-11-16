//! Alias validation and error handling.
//!
//! Validate aliases and encapsulate the result in `std::result::Result` to simplify
//! invalid aliases handling.

#![warn(clippy::all)]

use structopt::lazy_static::lazy_static;
use regex::Regex;

use crate::ErrorWrapper;

/// A `Result` used to represents an alias, valid or invalid. A valid alias is represented
/// as `Result::Ok<&str>`, where the **&str** is the alias (e.g. "gandalf.local"). An invalid
/// alias is represented as `Result::Err<&str>`, where the **&str** is the invalid alias.
///
/// # Examples
///
/// ```
/// use avahi_aliases::alias;
///
/// let a1 = alias::new("a1.local"); // Ok("a1.local")
/// let a2 = alias::new("a*.local"); // Err("a*.local")
/// ```
pub type Alias<'a> = Result<&'a str, &'a str>;

/// Determine the validity of a candidate alias.
///
/// Returns `true` if `alias` is a valid alias; otherwise, returns `false`.
///
/// # Examples
///
/// ```
/// use avahi_aliases::alias;
///
/// alias::is_valid("a1.local"); // true
/// alias::is_valid("a*.local"); // false
/// ```
///
/// # Notes
///
/// - The current definition of a valid alias is very simple: a word comprising at least one
///   letter, digit, or hyphen followed by `.local`; and, the word must begin and end with a
///   letter or digit.
pub fn is_valid(alias: &str) -> bool {
    lazy_static! {
        static ref VALIDATION_RE: Regex =
            Regex::new(r#"^[a-z0-9]([a-z0-9-]*[a-z0-9])?\.local$"#).unwrap();
    }
    VALIDATION_RE.is_match(alias)
}

/// Creates a new `Alias` from a specified alias.
///
/// # Examples
///
/// ```
/// use avahi_aliases::alias;
///
/// let a1 = alias::new("a1.local"); // Ok("a1.local")
/// let a2 = alias::new("a*.local"); // Err("a*.local")
/// ```
pub fn new(alias: &str) -> Alias<'_> {
    match is_valid(alias) {
        true => Ok(alias),
        false => Err(alias),
    }
}

/// Ensure all aliases in a collection are valid.
///
/// If an invalid alias is found, returns `Err(ErrorWrapper::InvalidAliasError)` for the
/// invalid alias. If all aliases are valid, , `Ok(())` is returned. Error checking stops
/// with the first error.
///
/// # Examples
///
/// ```
/// use avahi_aliases::{alias, ErrorWrapper};
///
/// fn some_action(aliases: &[&str]) -> Result<(), ErrorWrapper> {
///     alias::validate_aliases(aliases)?; // pass the error up the stack
///
///     // all aliases are valid
///     // ...
///
///     Ok(()) // return Ok(()) on success
/// }
/// ```

pub fn validate_aliases<T>(aliases: &[T]) -> Result<(), ErrorWrapper>
where
    T: AsRef<str>, {
    match aliases.iter().find(|a| !is_valid(a.as_ref())) {
        Some(invalid_alias) => {
            Err(ErrorWrapper::new_invalid_alias_error(invalid_alias.as_ref()))
        },
        None => Ok(()),
    }
}

//**********************************************************************************************
// Unit tests
//**********************************************************************************************

#[cfg(test)]
mod tests {
    use super::{is_valid, new, validate_aliases};

    static VALID_ALIASES: [&str; 4] = ["a.local", "xyzzy.local", "b0.local", "a-z.local"];
    static INVALID_ALIASES: [&str; 5] =
        ["a. local", "xyzz*.local", ".local", "-.local", "a-.local"];

    #[test]
    fn is_valid_returns_true_for_valid_alias() {
        VALID_ALIASES.iter().for_each(|a| assert!(is_valid(a)))
    }

    #[test]
    fn is_valid_returns_false_for_invalid_alias() {
        INVALID_ALIASES.iter().for_each(|a| assert!(!is_valid(a)))
    }

    #[test]
    fn validate_returns_ok_for_valid_alias() {
        VALID_ALIASES.iter().for_each(|a| assert!(new(a).is_ok()))
    }

    #[test]
    fn validate_returns_err_for_invalid_alias() {
        INVALID_ALIASES.iter().for_each(|a| assert!(new(a).is_err()))
    }

    #[test]
    fn validate_saves_valid_alias_as_ok() {
        VALID_ALIASES.iter().for_each(|a| assert_eq!(new(a).unwrap(), *a))
    }

    #[test]
    fn validate_saves_invalid_alias_as_err() {
        INVALID_ALIASES.iter().for_each(|a| assert_eq!(new(a).unwrap_err(), *a))
    }

    #[test]
    fn validate_aliases_returns_ok_for_valid_aliases() {
        let r = validate_aliases(&VALID_ALIASES);
        assert!(r.is_ok())
    }

    #[test]
    fn validate_aliases_returns_err_for_invalid_aliases() {
        let r = validate_aliases(&INVALID_ALIASES);
        assert!(r.is_err())
    }
}
