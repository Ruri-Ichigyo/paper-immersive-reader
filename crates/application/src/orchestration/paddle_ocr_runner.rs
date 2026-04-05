use std::path::{Path, PathBuf};
use std::process::Command;

use paper_immersive_reader_core::domain::OcrJob;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaddleOcrRunOutput {
    pub text: String,
}

pub trait PaddleOcrRunner {
    fn run(&self, job: &OcrJob, pdf_path: &Path) -> Result<PaddleOcrRunOutput, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptPaddleOcrRunner {
    script_path: PathBuf,
}

impl ScriptPaddleOcrRunner {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path }
    }
}

impl PaddleOcrRunner for ScriptPaddleOcrRunner {
    fn run(&self, _job: &OcrJob, pdf_path: &Path) -> Result<PaddleOcrRunOutput, String> {
        if !pdf_path.is_file() {
            return Err(format!("managed pdf asset missing: {}", pdf_path.display()));
        }

        let output = Command::new(&self.script_path)
            .arg(pdf_path)
            .output()
            .map_err(|error| format!("failed to run OCR script: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let mut message = format!("OCR script exited with status {}", output.status);
            if !stderr.is_empty() {
                message.push_str(": ");
                message.push_str(&stderr);
            }
            return Err(message);
        }

        let text = String::from_utf8(output.stdout)
            .map_err(|error| format!("OCR script produced invalid UTF-8: {error}"))?;
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err("OCR script produced empty output".to_string());
        }

        Ok(PaddleOcrRunOutput { text })
    }
}
