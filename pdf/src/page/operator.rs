use crate::error::{PdfError, Result};
use crate::object::PdfObject;

#[derive(Debug, Clone)]
pub struct Operator {
    op: String,
    operands: Vec<PdfObject>,
}

impl Operator {
    pub fn new(op: String, operands: Vec<PdfObject>) -> Self {
        Operator { op, operands }
    }

    pub fn name(&self) -> &str {
        self.op.as_str()
    }

    pub fn operand(&self, index: usize) -> Result<&PdfObject> {
        self.operands
            .get(index)
            .ok_or(PdfError::Interpreter(format!(
                "{:?}, can't have enough operands {:?}",
                self.op, self.operands
            )))
    }

    pub fn num_operands(&self) -> usize {
        self.operands.len()
    }
}

// PDF 32000-1:2008 Table 51 – Operator Categories
const PDF_CONTENT_COMMANDS: [&str; 73] = [
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

pub fn is_command(key: &[u8]) -> bool {
    for cmd in PDF_CONTENT_COMMANDS.iter() {
        if cmd.as_bytes() == key {
            return true;
        }
    }
    false
}
