//! Integration tests for utility tools.

mod common;

use dx_media::tools::utility::*;

// =============================================================================
// Hash Tools
// =============================================================================

#[test]
fn test_hash_md5() {
    use common::TestFixture;
    let fixture = TestFixture::new();
    let file = fixture.create_temp_file("test.txt", b"Hello World");

    let result = hash::md5(&file).unwrap();
    assert!(result.success);
    assert!(!result.message.is_empty());
}

#[test]
fn test_hash_sha256() {
    use common::TestFixture;
    let fixture = TestFixture::new();
    let file = fixture.create_temp_file("test.txt", b"Hello World");

    let result = hash::sha256(&file).unwrap();
    assert!(result.success);
    assert!(!result.message.is_empty());
}

// =============================================================================
// Base64 Tools
// =============================================================================

#[test]
fn test_base64_encode_decode() {
    let input = "Hello World!";
    let encoded = base64::encode_string(input).unwrap();
    assert!(encoded.success);
    assert_eq!(encoded.message, "SGVsbG8gV29ybGQh");

    let decoded = base64::decode_string(&encoded.message).unwrap();
    assert!(decoded.success);
    assert_eq!(decoded.message, input);
}

#[test]
fn test_base64_encode_file() {
    use common::TestFixture;
    let fixture = TestFixture::new();
    let file = fixture.create_temp_file("test.txt", b"Hello");

    let result = base64::encode_file(&file).unwrap();
    assert!(result.success);
    assert_eq!(result.message, "SGVsbG8=");
}

// =============================================================================
// URL Encoding Tools
// =============================================================================

#[test]
fn test_url_encode_decode() {
    let input = "Hello World!";
    let encoded = url_encode::encode(input).unwrap();
    assert!(encoded.success);
    // URL encoding uses + for spaces
    assert!(encoded.message.contains("Hello") && encoded.message.contains("World"));

    let decoded = url_encode::decode(&encoded.message).unwrap();
    assert!(decoded.success);
    assert_eq!(decoded.message, input);
}

#[test]
fn test_url_encode_path() {
    let result = url_encode::encode_path("/path/to/file name.txt").unwrap();
    assert!(result.success);
    assert!(result.message.contains("%20"));
}

#[test]
fn test_url_parse_query_string() {
    let result = url_encode::parse_query_string("?name=John&age=30").unwrap();
    assert!(result.success);
}

// =============================================================================
// JSON Tools
// =============================================================================

#[test]
fn test_json_format() {
    let input = r#"{"name":"John","age":30}"#;
    let result = json_format::format_string(input).unwrap();
    assert!(result.success);
    assert!(result.message.contains('\n'));
}

#[test]
fn test_json_minify() {
    let input = r#"{
  "name": "John",
  "age": 30
}"#;
    let result = json_format::minify_string(input).unwrap();
    assert!(result.success);
    assert!(!result.message.contains('\n'));
}

#[test]
fn test_json_validate() {
    let valid = r#"{"name":"John"}"#;
    let result = json_format::validate_string(valid).unwrap();
    assert!(result.success);
    assert_eq!(result.metadata.get("valid"), Some(&"true".to_string()));

    let invalid = r#"{"name":"John""#;
    let result = json_format::validate_string(invalid).unwrap();
    assert!(result.success); // Returns success with error metadata
    assert_eq!(result.metadata.get("valid"), Some(&"false".to_string()));
}

// =============================================================================
// UUID Tools
// =============================================================================

#[test]
fn test_uuid_generate_v4() {
    let result = uuid::generate(uuid::UuidVersion::V4).unwrap();
    assert!(result.success);
    assert_eq!(result.message.len(), 36); // UUID format: 8-4-4-4-12
}

#[test]
fn test_uuid_validate() {
    let valid = "550e8400-e29b-41d4-a716-446655440000";
    let result = uuid::validate(valid).unwrap();
    assert!(result.success);
    assert_eq!(result.metadata.get("valid"), Some(&"true".to_string()));

    let invalid = "not-a-uuid";
    let result = uuid::validate(invalid).unwrap();
    assert!(!result.success);
}

#[test]
fn test_uuid_batch() {
    let result = uuid::generate_batch(5, uuid::UuidVersion::V4).unwrap();
    assert!(result.success);
}

// =============================================================================
// Timestamp Tools
// =============================================================================

#[test]
fn test_timestamp_now() {
    let result = timestamp::now(timestamp::TimestampFormat::Unix).unwrap();
    assert!(result.success);
    assert!(!result.message.is_empty());
}

#[test]
fn test_timestamp_convert() {
    let result = timestamp::convert(
        "1609459200",
        timestamp::TimestampFormat::Unix,
        timestamp::TimestampFormat::Iso8601,
    )
    .unwrap();
    assert!(result.success);
    assert!(result.message.contains("2021"));
}

// =============================================================================
// Random Tools
// =============================================================================

#[test]
fn test_random_string() {
    let result = random::string(16, random::CharSet::Alphanumeric).unwrap();
    assert!(result.success);
    assert_eq!(result.message.len(), 16);
}

#[test]
fn test_random_integer() {
    let result = random::integer(1, 100).unwrap();
    assert!(result.success);
    let num: i64 = result.message.parse().unwrap();
    assert!((1..=100).contains(&num));
}

#[test]
fn test_random_boolean() {
    let result = random::boolean().unwrap();
    assert!(result.success);
    assert!(result.message == "true" || result.message == "false");
}

#[test]
fn test_random_password() {
    let result = random::password(12, true).unwrap();
    assert!(result.success);
    assert!(result.message.len() >= 12);
}

#[test]
fn test_random_color() {
    let result = random::color("hex").unwrap();
    assert!(result.success);
    assert!(result.message.starts_with('#'));
}

// =============================================================================
// Checksum Tools
// =============================================================================

#[test]
fn test_checksum_calculate() {
    use common::TestFixture;
    let fixture = TestFixture::new();
    let file = fixture.create_temp_file("test.txt", b"Hello World");

    let result = checksum::calculate_checksum(&file, checksum::ChecksumAlgorithm::Md5).unwrap();
    assert_eq!(result.algorithm, checksum::ChecksumAlgorithm::Md5);
    assert!(!result.hash.is_empty());
}

#[test]
fn test_checksum_verify() {
    use common::TestFixture;
    let fixture = TestFixture::new();
    let file = fixture.create_temp_file("test.txt", b"Hello World");

    let calc = checksum::calculate_checksum(&file, checksum::ChecksumAlgorithm::Md5).unwrap();
    let verified =
        checksum::verify_checksum(&file, &calc.hash, checksum::ChecksumAlgorithm::Md5).unwrap();
    assert!(verified);
}

// =============================================================================
// Diff Tools
// =============================================================================

#[test]
fn test_diff_strings() {
    let s1 = "Hello\nWorld";
    let s2 = "Hello\nRust";
    let result = diff::diff_strings(s1, s2).unwrap();
    assert!(result.success);
}

// =============================================================================
// Duplicate Detection
// =============================================================================

#[test]
fn test_find_duplicates() {
    use common::TestFixture;
    let fixture = TestFixture::new();

    // Create duplicate files
    fixture.create_temp_file("file1.txt", b"content");
    fixture.create_temp_file("file2.txt", b"content");
    fixture.create_temp_file("file3.txt", b"different");

    let options = duplicate::DuplicateOptions::default();
    let result = duplicate::find_duplicates_tool(fixture.temp_dir.path(), Some(options));
    assert!(result.success);
}

// =============================================================================
// CSV Conversion
// =============================================================================

#[test]
fn test_csv_to_json() {
    use common::TestFixture;
    let fixture = TestFixture::new();

    let csv_file = fixture.create_temp_file("test.csv", b"name,age\nJohn,30\nJane,25");
    let json_file = fixture.path("output.json");

    let result = csv_convert::csv_to_json(&csv_file, &json_file).unwrap();
    assert!(result.success);

    let content = std::fs::read_to_string(&json_file).unwrap();
    assert!(content.contains("John"));
}

#[test]
fn test_json_to_csv() {
    use common::TestFixture;
    let fixture = TestFixture::new();

    let json_file = fixture.create_temp_file("test.json", b"[{\"name\":\"John\",\"age\":30}]");
    let csv_file = fixture.path("output.csv");

    let result = csv_convert::json_to_csv(&json_file, &csv_file).unwrap();
    assert!(result.success);

    // Check file was created
    assert!(csv_file.exists());
}

// =============================================================================
// YAML Conversion
// =============================================================================

#[test]
fn test_json_to_yaml() {
    let json = r#"{"name":"John","age":30}"#;
    let result = yaml_convert::json_string_to_yaml(json).unwrap();
    assert!(result.success);
    assert!(result.message.contains("name"));
}

#[test]
fn test_yaml_to_json() {
    let yaml = "name: John\nage: 30";
    let result = yaml_convert::yaml_string_to_json(yaml).unwrap();
    assert!(result.success);
    assert!(result.message.contains("John"));
}
