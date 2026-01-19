#[derive(Debug)]
pub enum ParserError {
    UserNotFound,
    UserAlreadyExists,
    InvalidCredentials,
    PostNotFound,
    Forbidden
}