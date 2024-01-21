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

// PDF 32000-1:2008 Table 51 – Operator Categories
const PDF_CONTENT_COMMANDS: [&'static str; 73] = [
    "w", "J", "j", "M", "d", "i", "ri", "gs", // General graphics state
    "q", "Q", "cm", // Special graphics state
    "m", "l", "c", "v", "re", "y", "h", // Path construction
    "s", "F", "f*", "B", "B*", "b", "b*", "n", "S", "f", // Path painting
    "W", "W*", // Clipping paths
    "BT", "ET", // Text objects
    "Tc", "Tw", "Tz", "TL", "Tf", "Tr", "Ts", // Text state
    "Td", "TD", "Tm", "T*", // Text positioning
    "Tj", "'", "\"", "TJ", // Text showing
    "d0", "d1", // Type 3 fonts
    "cs", "CS", "sc", "SC", "scn", "SCN", "g", "G", "rg", "RG", "k", "K",  // Color
    "sh", // Shading patterns
    "BI", "ID", "EI", // Inline images
    "Do", // XObjects
    "MP", "DP", "BMC", "BDC", "EMC", // Marked content
    "BX", "EX", // Compatibility
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
