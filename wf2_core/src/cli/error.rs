#[derive(Debug, Fail)]
pub enum CLIError {
    #[fail(display = "Invalid config {}", _0)]
    InvalidConfig(String),
    #[fail(display = "The following does not exist {:?}", _0)]
    MissingConfig(std::path::PathBuf),
    #[fail(display = "{}", _0)]
    VersionDisplayed(String),
}
