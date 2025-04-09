use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("Password is wrong")]
    WrongPassword,

    #[error("Reader Error: '{0}'")]
    Reader(String),

    #[error("File Error:'{0}'")]
    File(String),

    #[error("Parse object error: '{0}'")]
    ParseObject(String),

    #[error("Filter error:{0}")]
    Filter(String),

    #[error("Object error:{0}")]
    Object(String),

    #[error("Object error:{0}")]
    Xref(String),

    #[error("Document Structure error:{0}")]
    DocumentStructure(String),

    #[error("PDF Page Error {0}")]
    Page(String),

    #[error("Page Content Interpreter error:{0}")]
    Interpreter(String),

    #[error("Content parser error:{0}")]
    ContentParser(String),

    #[error("Path error:{0}")]
    Path(String),

    #[error("Font error:{0}")]
    Font(String),

    #[error("Character:{0}")]
    Character(String),

    #[error("Color:{0}")]
    Color(String),

    #[error("Function:{0}")]
    Function(String),

    #[error("Image:{0}")]
    Image(String),

    #[error("Pattern:{0}")]
    Pattern(String),
}

pub type Result<T> = std::result::Result<T, PdfError>;
