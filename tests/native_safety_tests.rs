mod common;

use common::TestFixture;

#[cfg(feature = "image-core")]
#[test]
fn native_image_compress_rejects_non_jpeg_output() {
    use dx_media::tools::image::native::compress_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_image("test.png");
    let output = fixture.path("compressed.png");

    let result = compress_native(&input, &output, 80);

    assert!(
        result.is_err(),
        "compression should reject non-JPEG output paths"
    );
    assert_eq!(
        result.err().unwrap().kind(),
        std::io::ErrorKind::InvalidInput
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_rejects_symlink_entries() {
    use dx_media::tools::archive::native::extract_tar_gz_native;
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let fixture = TestFixture::new();
    let archive_path = fixture.path("unsafe.tar.gz");
    let output_dir = fixture.path("unsafe_extract");

    let file = std::fs::File::create(&archive_path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_link_name("../outside.txt").unwrap();
    header.set_cksum();
    builder
        .append_data(&mut header, "link", std::io::empty())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap();

    let result = extract_tar_gz_native(&archive_path, &output_dir);

    assert!(result.is_err(), "symlink entries should not be unpacked");
    assert_eq!(
        result.err().unwrap().kind(),
        std::io::ErrorKind::InvalidInput
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_rejects_parent_directory_entries() {
    use dx_media::tools::archive::native::extract_zip_native;
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let fixture = TestFixture::new();
    let archive_path = fixture.path("unsafe.zip");
    let output_dir = fixture.path("zip_extract");

    let file = std::fs::File::create(&archive_path).unwrap();
    let mut zip = ZipWriter::new(file);
    zip.start_file("../outside.txt", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"escape").unwrap();
    zip.finish().unwrap();

    let result = extract_zip_native(&archive_path, &output_dir);

    assert!(
        result.is_err(),
        "parent-directory ZIP entries should not be unpacked"
    );
    assert_eq!(
        result.err().unwrap().kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert!(
        !fixture.path("outside.txt").exists(),
        "unsafe ZIP entry must not escape the extraction directory"
    );
}
