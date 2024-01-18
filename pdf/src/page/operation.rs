use crate::errors::PDFResult;
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub struct Operation {
    op: String,
    operands: Vec<PDFObject>,
}

impl Operation {
    pub fn new(op: String, operands: Vec<PDFObject>) -> Self {
        Operation { op, operands }
    }

    pub fn name(&self) -> &str {
        &self.op
    }

    pub fn operand(&self, index: usize) -> PDFResult<&PDFObject> {
        self.operands
            .get(index)
            .ok_or(crate::errors::PDFError::OperationError(format!(
                "{:?}, can't have enough operands {:?}",
                self.op, self.operands
            )))
    }
}

const PDF_CONTENT_COMMANDS: [&'static str; 58] = [
    "q", "Q", "cm", "Do", "BMC", "BDC", "EMC", "BT", "ET", "Tc", "Tw", "Tz", "TL", "Tf", "Tr",
    "Ts", "Td", "TD", "Tm", "T*", "Tj", "'", "\"", "TJ", "w", "J", "j", "M", "d", "m", "l", "c",
    "v", "re", "y", "h", "S", "s", "F", "f*", "B", "B*", "b", "b*", "n", "f", "g", "G", "rg", "RG",
    "k", "K", "cs", "CS", "sc", "SC", "gs", "EI",
];

pub fn to_command(bytes: &[u8]) -> Option<String> {
    match String::from_utf8(bytes.to_owned()) {
        Ok(s) => {
            if PDF_CONTENT_COMMANDS.contains(&s.as_str()) {
                return Some(s);
            }
            None
        }
        _ => None,
    }
}
