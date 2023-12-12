use crate::errors::PDFResult;
use crate::object::PDFObject;

#[derive(Debug, Clone)]
pub(crate) struct Operation {
    op: String,
    operands: Vec<PDFObject>,
}

impl Operation {
    pub fn new(op: String, operands: Vec<PDFObject>) -> Self {
        // TODO fix this values
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
