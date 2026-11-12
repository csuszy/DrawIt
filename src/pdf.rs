use image::{codecs::jpeg::JpegEncoder, ExtendedColorType, ImageEncoder};
use std::io::Write;

/// Generates a single-page PDF containing the provided RGB image according to ISO 32000-1.
/// The image is placed on a page whose dimensions match the image aspect ratio at 72 DPI (or 96 DPI).
pub fn create_pdf_from_rgba(width: u32, height: u32, rgba_pixels: &[u8]) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 || rgba_pixels.is_empty() {
        return Err("Érvénytelen képméret a PDF generáláshoz.".to_string());
    }

    // Convert RGBA to RGB for standard JPEG encoding
    let total_pixels = (width as usize) * (height as usize);
    let mut rgb_pixels = Vec::with_capacity(total_pixels * 3);
    for chunk in rgba_pixels.chunks_exact(4) {
        rgb_pixels.push(chunk[0]); // R
        rgb_pixels.push(chunk[1]); // G
        rgb_pixels.push(chunk[2]); // B
    }

    // Encode to JPEG in memory with quality 90
    let mut jpeg_bytes = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 90);
    encoder
        .write_image(&rgb_pixels, width, height, ExtendedColorType::Rgb8)
        .map_err(|e| format!("JPEG kódolási hiba: {}", e))?;

    // Page dimensions in PDF points (72 points per inch; standard display is 96 DPI)
    // 1 pixel = 72 / 96 points = 0.75 points
    let page_w = (width as f64) * 0.75;
    let page_h = (height as f64) * 0.75;

    let mut pdf = Vec::with_capacity(jpeg_bytes.len() + 2048);
    let mut offsets = Vec::new();

    // 1. Header
    pdf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    // Helper to start an object and record offset
    let write_obj = |pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, obj_num: usize, content: &[u8]| {
        offsets.push(pdf.len());
        let _ = write!(pdf, "{} 0 obj\n", obj_num);
        pdf.extend_from_slice(content);
        pdf.extend_from_slice(b"\nendobj\n");
    };

    // Object 1: Catalog
    write_obj(
        &mut pdf,
        &mut offsets,
        1,
        b"<< /Type /Catalog /Pages 2 0 R >>",
    );

    // Object 2: Pages
    let pages_dict = format!("<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
    write_obj(&mut pdf, &mut offsets, 2, pages_dict.as_bytes());

    // Object 3: Page
    let page_dict = format!(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents 4 0 R /Resources << /XObject << /Im0 5 0 R >> >> >>",
        page_w, page_h
    );
    write_obj(&mut pdf, &mut offsets, 3, page_dict.as_bytes());

    // Object 4: Contents (Stream drawing Im0 to fill page)
    let content_stream = format!(
        "q {:.2} 0 0 {:.2} 0 0 cm /Im0 Do Q\n",
        page_w, page_h
    );
    let contents_obj = format!(
        "<< /Length {} >>\nstream\n{}endstream",
        content_stream.len(),
        content_stream
    );
    write_obj(&mut pdf, &mut offsets, 4, contents_obj.as_bytes());

    // Object 5: Image XObject with JPEG stream
    let image_header = format!(
        "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
        width,
        height,
        jpeg_bytes.len()
    );
    offsets.push(pdf.len());
    let _ = write!(&mut pdf, "5 0 obj\n");
    pdf.extend_from_slice(image_header.as_bytes());
    pdf.extend_from_slice(&jpeg_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // XRef table
    let xref_offset = pdf.len();
    let num_objects = offsets.len() + 1; // including 0
    let _ = write!(&mut pdf, "xref\n0 {}\n0000000000 65535 f \n", num_objects);
    for off in &offsets {
        let _ = write!(&mut pdf, "{:010} 00000 n \n", off);
    }

    // Trailer
    let _ = write!(
        &mut pdf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        num_objects, xref_offset
    );

    Ok(pdf)
}
