use std::io::{Cursor, Write};
use tar::Builder as TarBuilder;
use xz2::write::XzEncoder;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

pub fn build_zip(rendered_htmls: Vec<(String, Vec<u8>)>) -> std::io::Result<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);

    let file_opts = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for (name, html) in rendered_htmls {
        zip.start_file(format!("{}.html", name), file_opts)?;
        zip.write_all(&html)?;
    }

    Ok(zip.finish()?.into_inner())
}

pub fn build_tzip(rendered_htmls: Vec<(String, Vec<u8>)>) -> std::io::Result<Vec<u8>> {
    // XZ encoder wraps the inner Vec<u8> cursor
    let cursor = Cursor::new(Vec::new());
    let xz = XzEncoder::new(cursor, 6); // level 6 is a good default
    let mut tar = TarBuilder::new(xz);

    for (name, html) in rendered_htmls {
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(html.len() as u64);
        header.set_cksum();
        tar.append_data(&mut header, format!("{}.html", name), html.as_slice())?;
    }

    // Close TAR, then XZ, then extract inner Vec<u8>
    let xz = tar.into_inner()?;
    let cursor = xz.finish()?;
    Ok(cursor.into_inner())
}
