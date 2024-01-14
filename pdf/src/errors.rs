use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PDFError {
    #[error("IO:{msg}")]
    IO { source: io::Error, msg: String },

    #[error("Eof:{msg}")]
    Eof { msg: String },

    #[error("Invalid PDF FileStructure: `{0}` ")]
    InvalidFileStructure(String),

    #[error("InvalidSyntax`{0}`")]
    InvalidSyntax(String),

    #[error("Tokenizer Error:`{0}`")]
    LexFailure(String),

    #[error("Token Convert Failure:`{0}`")]
    TokenConvertFailure(String),

    #[error("Invalud Content: `{0}`")]
    InvalidContentSyntax(String),

    #[error("PDFObject Convert Failure:`{0}`")]
    ObjectConvertFailure(String),

    #[error("Font Cmap Error: '{0}'")]
    FontCmapFailure(String),

    #[error("FontFreeType Error:{0}")]
    FontFreeType(String),

    #[error("FontSimple Error:{0}")]
    FontEncoding(String),

    #[error("Filter Error `{0}`")]
    Filter(String),

    #[error("Content Interpret Error `{0}`")]
    ContentInterpret(String),

    #[error("OperationError `{0}`")]
    OperationError(String),

    #[error("PathCreatError `{0}`")]
    PathCreatError(String),
}

pub type PDFResult<T> = std::result::Result<T, PDFError>;
