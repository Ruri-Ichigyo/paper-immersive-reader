use paper_immersive_reader_core::domain::OcrJob;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaddleOcrRunOutput {
    pub text: String,
}

pub trait PaddleOcrRunner {
    fn run(&self, job: &OcrJob) -> Result<PaddleOcrRunOutput, String>;
}
