use std::sync::OnceLock;
use windows::core::{Result, HRESULT};
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;

static OCR_STATUS: OnceLock<OcrStatus> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct OcrStatus {
    pub is_supported: bool,
    pub language_tag: String,
}

/// Returns the cached OCR status and detected language tag on this system.
pub fn get_ocr_status() -> &'static OcrStatus {
    OCR_STATUS.get_or_init(|| {
        // 1. Check if an engine can be initialized with user profile languages
        if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
            if let Ok(lang) = engine.RecognizerLanguage() {
                if let Ok(tag) = lang.LanguageTag() {
                    let s = tag.to_string();
                    if !s.is_empty() {
                        return OcrStatus {
                            is_supported: true,
                            language_tag: s,
                        };
                    }
                }
            }
        }
        // 2. Fallback: check if any language is in available recognizers list
        if let Ok(langs) = OcrEngine::AvailableRecognizerLanguages() {
            if let Ok(size) = langs.Size() {
                if size > 0 {
                    if let Ok(first) = langs.GetAt(0) {
                        if let Ok(tag) = first.LanguageTag() {
                            return OcrStatus {
                                is_supported: true,
                                language_tag: tag.to_string(),
                            };
                        }
                    }
                }
            }
        }
        OcrStatus {
            is_supported: false,
            language_tag: String::new(),
        }
    })
}

/// Checks whether Windows.Media.Ocr is supported on this system.
pub fn is_ocr_supported() -> bool {
    get_ocr_status().is_supported
}

/// Creates the most accurate OcrEngine available.
/// Prioritizes user profile languages list (system preferences), falling back to first available.
fn create_engine() -> Result<OcrEngine> {
    if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
        return Ok(engine);
    }
    if let Ok(langs) = OcrEngine::AvailableRecognizerLanguages() {
        if let Ok(size) = langs.Size() {
            if size > 0 {
                if let Ok(first) = langs.GetAt(0) {
                    if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&first) {
                        return Ok(engine);
                    }
                }
            }
        }
    }
    Err(windows::core::Error::new(
        HRESULT(-1),
        "No OCR language available on this system",
    ))
}

/// Performs OCR on raw 32-bit BGRA pixels.
/// Returns Ok(Some(text)) if non-empty text was recognized, Ok(None) if no text was found,
/// or Err if recognition failed.
pub fn recognize_text_from_bgra(width: u32, height: u32, bgra_pixels: &[u8]) -> Result<Option<String>> {
    if width == 0 || height == 0 || bgra_pixels.is_empty() {
        return Ok(None);
    }

    let engine = create_engine()?;

    let writer = DataWriter::new()?;
    writer.WriteBytes(bgra_pixels)?;
    let buffer = writer.DetachBuffer()?;

    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        width as i32,
        height as i32,
    )?;

    let async_op = engine.RecognizeAsync(&bitmap)?;
    let result = async_op.get()?;

    let h_str = result.Text()?;
    let text = h_str.to_string();
    let trimmed = text.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_availability() {
        let supported = is_ocr_supported();
        println!("Windows.Media.Ocr supported on this system: {}", supported);
        if supported {
            let engine = create_engine();
            assert!(engine.is_ok(), "Engine creation should succeed when supported");
            if let Ok(e) = engine {
                if let Ok(lang) = e.RecognizerLanguage() {
                    if let Ok(tag) = lang.LanguageTag() {
                        println!("Active OCR recognizer language: {}", tag);
                    }
                }
            }

            // Test recognizing an empty 10x10 transparent image (should return Ok(None))
            let blank_pixels = vec![255u8; 10 * 10 * 4];
            let res = recognize_text_from_bgra(10, 10, &blank_pixels);
            assert!(res.is_ok(), "Recognize on blank image should not error");
            assert_eq!(res.unwrap(), None, "Blank image should have no text");
        }
    }
}
