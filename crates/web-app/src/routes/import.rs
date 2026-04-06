use crate::import::{
    PasteImportRequest, PdfImportRequest, PdfTextBridge, UnavailablePdfTextBridge, WebImportError,
    WebImportResult, WebImportService,
};
use crate::storage::WebStorage;

use super::RouteDescriptor;

pub fn descriptor() -> RouteDescriptor {
    RouteDescriptor {
        path: "/import",
        label: "Import",
        description: "Import local PDFs or pasted text into the local library.",
    }
}

pub struct ImportRouteModel<B = UnavailablePdfTextBridge> {
    service: WebImportService<B>,
}

impl ImportRouteModel<UnavailablePdfTextBridge> {
    pub fn new(storage: WebStorage) -> Self {
        Self {
            service: WebImportService::new(storage),
        }
    }
}

impl<B> ImportRouteModel<B>
where
    B: PdfTextBridge,
{
    pub fn new_with_pdf_bridge(storage: WebStorage, pdf_bridge: B) -> Self {
        Self {
            service: WebImportService::new_with_pdf_bridge(storage, pdf_bridge),
        }
    }

    pub fn submit_paste(
        &self,
        request: PasteImportRequest,
    ) -> Result<WebImportResult, WebImportError> {
        self.service.import_paste_text(request)
    }

    pub fn submit_pdf(&self, request: PdfImportRequest) -> Result<WebImportResult, WebImportError> {
        self.service.import_pdf_document(request)
    }
}
