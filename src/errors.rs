use failure::Fail;

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "empty expression")]
    Empty,

    #[fail(display = "mismatched parentheses")]
    MismatchedParens,

    #[fail(display = "encountered erroneous '{}'", _0)]
    ErroneousToken(String),
}

#[derive(Debug, Fail)]
pub enum RunError {
    #[fail(display = "{}: {}", name, msg)]
    ProcError { name: String, msg: String },

    #[fail(display = "{}: index out of bounds", _0)]
    IndexOutOfBounds(usize),

    #[fail(display = "{}: expected a {}, got a {} instead", name, expected, got)]
    TypeError {
        name: String,
        expected: String,
        got: String
    },

    #[fail(display = "value `{}` (of type {}) is uncallable", name, typename)]
    UncallableValue { name: String, typename: String },

    #[fail(display = "{}: expected {} params, got {} instead", name, expected, got)]
    WrongNumArgs {
        name: String,
        expected: usize,
        got: usize
    },

    // #[fail(display = "division by zero is undefined")]
    // DivideByZero,
}
