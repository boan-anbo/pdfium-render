//! Defines the [PdfDocument] struct, the entry point to all Pdfium functionality
//! related to a single PDF file.

use crate::bindgen::FPDF_DOCUMENT;
use crate::bindings::PdfiumLibraryBindings;
use crate::bookmarks::PdfBookmarks;
use crate::error::PdfiumError;
use crate::form::PdfForm;
use crate::metadata::PdfMetadata;
use crate::pages::PdfPages;
use crate::permissions::PdfPermissions;
use crate::utils::files::FpdfFileAccessExt;
use std::os::raw::c_int;

#[cfg(not(target_arch = "wasm32"))]
use crate::utils::files::get_pdfium_file_writer_from_writer;

#[cfg(not(target_arch = "wasm32"))]
use crate::error::PdfiumInternalError;

#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

/// The file version of a [PdfDocument].
///
/// A list of PDF file versions is available at <https://en.wikipedia.org/wiki/History_of_PDF>.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PdfDocumentVersion {
    /// No version information is available. This is the case if the [PdfDocument]
    /// was created via a call to `Pdfium::create_new_pdf()` rather than loaded from a file.
    Unset,

    /// PDF 1.0, first published in 1993, supported by Acrobat Reader Carousel (1.0) onwards.
    Pdf1_0,

    /// PDF 1.1, first published in 1994, supported by Acrobat Reader 2.0 onwards.
    Pdf1_1,

    /// PDF 1.2, first published in 1996, supported by Acrobat Reader 3.0 onwards.
    Pdf1_2,

    /// PDF 1.3, first published in 2000, supported by Acrobat Reader 4.0 onwards.
    Pdf1_3,

    /// PDF 1.4, first published in 2001, supported by Acrobat Reader 5.0 onwards.
    Pdf1_4,

    /// PDF 1.5, first published in 2003, supported by Acrobat Reader 6.0 onwards.
    Pdf1_5,

    /// PDF 1.6, first published in 2004, supported by Acrobat Reader 7.0 onwards.
    Pdf1_6,

    /// PDF 1.7, first published in 2006, supported by Acrobat Reader 8.0 onwards,
    /// adopted as ISO open standard 32000-1 in 2008. Certain proprietary Adobe
    /// extensions to PDF 1.7 are only fully supported in Acrobat Reader X (10.0)
    /// and later.
    Pdf1_7,

    /// PDF 2.0, first published in 2017, ISO open standard 32000-2.
    Pdf2_0,

    /// A two-digit raw file version number. For instance, a value of 21 would indicate
    /// PDF version 2.1, a value of 34 would indicate PDF version 3.4, and so on.
    /// Only used when the file version number is not directly recognized by
    /// pdfium-render.
    Other(i32),
}

impl PdfDocumentVersion {
    /// The default [PdfDocumentVersion] applied to new documents.
    pub const DEFAULT_VERSION: PdfDocumentVersion = PdfDocumentVersion::Pdf1_7;

    #[inline]
    pub(crate) fn from_pdfium(version: i32) -> Self {
        match version {
            10 => PdfDocumentVersion::Pdf1_0,
            11 => PdfDocumentVersion::Pdf1_1,
            12 => PdfDocumentVersion::Pdf1_2,
            13 => PdfDocumentVersion::Pdf1_3,
            14 => PdfDocumentVersion::Pdf1_4,
            15 => PdfDocumentVersion::Pdf1_5,
            16 => PdfDocumentVersion::Pdf1_6,
            17 => PdfDocumentVersion::Pdf1_7,
            20 => PdfDocumentVersion::Pdf2_0,
            _ => PdfDocumentVersion::Other(version),
        }
    }

    #[inline]
    pub(crate) fn as_pdfium(&self) -> Option<i32> {
        match self {
            PdfDocumentVersion::Pdf1_0 => Some(10),
            PdfDocumentVersion::Pdf1_1 => Some(11),
            PdfDocumentVersion::Pdf1_2 => Some(12),
            PdfDocumentVersion::Pdf1_3 => Some(13),
            PdfDocumentVersion::Pdf1_4 => Some(14),
            PdfDocumentVersion::Pdf1_5 => Some(15),
            PdfDocumentVersion::Pdf1_6 => Some(16),
            PdfDocumentVersion::Pdf1_7 => Some(17),
            PdfDocumentVersion::Pdf2_0 => Some(20),
            PdfDocumentVersion::Other(value) => Some(*value),
            PdfDocumentVersion::Unset => None,
        }
    }
}

/// An entry point to all the various object collections contained in a single PDF file.
/// These collections include:
/// * [PdfDocument::pages()], all the [PdfPages] in the document.
/// * [PdfDocument::metadata()], all the [PdfMetadata] tags in the document.
/// * [PdfDocument::form()], the [PdfForm] optionally embedded in the document.
/// * [PdfDocument::bookmarks()], all the [PdfBookmarks] in the document.
/// * [PdfDocument::permissions()], settings relating to security handlers and document permissions
/// for the document.
pub struct PdfDocument<'a> {
    handle: FPDF_DOCUMENT,
    form: Option<PdfForm<'a>>,
    bindings: &'a dyn PdfiumLibraryBindings,
    output_version: Option<PdfDocumentVersion>,
    file_access_reader: Option<Box<FpdfFileAccessExt>>,
}

impl<'a> PdfDocument<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        handle: FPDF_DOCUMENT,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        Self {
            handle,
            form: PdfForm::from_pdfium(handle, bindings),
            bindings,
            output_version: None,
            file_access_reader: None,
        }
    }

    /// Returns the internal FPDF_DOCUMENT handle for this [PdfDocument].
    #[inline]
    pub(crate) fn get_handle(&self) -> &FPDF_DOCUMENT {
        &self.handle
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfDocument].
    #[inline]
    pub(crate) fn get_bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Binds an `FPDF_FILEACCESS` reader to the lifetime of this [PdfDocument], so that
    /// it will always be available for Pdfium to read data from as needed.
    #[inline]
    pub(crate) fn set_file_access_reader(&mut self, reader: Box<FpdfFileAccessExt>) {
        self.file_access_reader = Some(reader);
    }

    /// Returns the file version of this [PdfDocument].
    pub fn version(&self) -> PdfDocumentVersion {
        let mut version: c_int = 0;

        if self.bindings.FPDF_GetFileVersion(self.handle, &mut version) != 0 {
            PdfDocumentVersion::from_pdfium(version)
        } else {
            PdfDocumentVersion::Unset
        }
    }

    /// Sets the file version that will be used the next time this [PdfDocument] is saved
    /// using the [PdfDocument::save_to_writer()] function.
    pub fn set_version(&mut self, version: PdfDocumentVersion) {
        self.output_version = Some(version);
    }

    /// Returns the collection of [PdfPages] in this [PdfDocument].
    #[inline]
    pub fn pages(&self) -> PdfPages {
        PdfPages::new(self, self.bindings)
    }

    /// Returns the collection of [PdfMetadata] tags in this [PdfDocument].
    #[inline]
    pub fn metadata(&self) -> PdfMetadata {
        PdfMetadata::new(self, self.bindings)
    }

    /// Returns a reference to the [PdfForm] embedded in this [PdfDocument], if any.
    #[inline]
    pub fn form(&self) -> Option<&PdfForm> {
        self.form.as_ref()
    }

    /// Returns the collection of [PdfBookmarks] in this [PdfDocument].
    #[inline]
    pub fn bookmarks(&self) -> PdfBookmarks {
        PdfBookmarks::new(self, self.bindings)
    }

    /// Returns the collection of [PdfPermissions] for this [PdfDocument].
    #[inline]
    pub fn permissions(&self) -> PdfPermissions {
        PdfPermissions::new(self)
    }

    /// Copies all pages in the given [PdfDocument] into this [PdfDocument], appending them
    /// to the end of this document's [PdfPages] collection.
    ///
    /// For finer control over which pages are imported, and where they should be inserted,
    /// use one of the [PdfPages::copy_page_from_document()], [PdfPages::copy_pages_from_document()],
    ///  or [PdfPages::copy_page_range_from_document()] functions.
    ///
    /// Calling this function is equivalent to
    ///
    /// ```
    /// self.pages().copy_page_range_from_document(
    ///     document, // Source
    ///     document.pages().as_range_inclusive(), // Select all pages
    ///     self.pages().len() // Append to end of current document
    /// );
    /// ```
    pub fn append(&mut self, document: &PdfDocument) -> Result<(), PdfiumError> {
        self.pages().copy_page_range_from_document(
            document,
            document.pages().as_range_inclusive(),
            self.pages().len(),
        )
    }

    /// Writes this [PdfDocument] to the given writer.
    ///
    /// This function is not available when compiling to WASM.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_writer<W: Write>(&self, writer: W) -> Result<(), PdfiumError> {
        // TODO: AJRC - 25/5/22 - investigate supporting the FPDF_INCREMENTAL, FPDF_NO_INCREMENTAL,
        // and FPDF_REMOVE_SECURITY flags defined in fpdf_save.h. There's not a lot of information
        // on what they actually do, however.
        // Some small info at https://forum.patagames.com/posts/t155-PDF-SaveFlags.

        let flags = 0;

        let mut pdfium_file_writer = get_pdfium_file_writer_from_writer(writer);

        let result = match self.output_version {
            Some(version) => self.bindings.FPDF_SaveWithVersion(
                self.handle,
                pdfium_file_writer.as_fpdf_file_write_mut_ptr(),
                flags,
                version
                    .as_pdfium()
                    .unwrap_or_else(|| PdfDocumentVersion::DEFAULT_VERSION.as_pdfium().unwrap()),
            ),
            None => self.bindings.FPDF_SaveAsCopy(
                self.handle,
                pdfium_file_writer.as_fpdf_file_write_mut_ptr(),
                flags,
            ),
        };

        match self.bindings.is_true(result) {
            true => {
                // Pdfium's return value indicated success. Flush the buffer,
                // returning any final I/O error.

                pdfium_file_writer.flush().map_err(PdfiumError::IoError)
            }
            false => {
                // Pdfium's return value indicated failure.

                Err(PdfiumError::PdfiumLibraryInternalError(
                    self.bindings
                        .get_pdfium_last_error()
                        .unwrap_or(PdfiumInternalError::Unknown),
                ))
            }
        }
    }
}

impl<'a> Drop for PdfDocument<'a> {
    /// Closes this [PdfDocument], releasing held memory and, if the document was loaded
    /// from a file, the file handle on the document.
    #[inline]
    fn drop(&mut self) {
        self.bindings.FPDF_CloseDocument(self.handle);
    }
}