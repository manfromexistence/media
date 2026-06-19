//! Tests for utility tools.

mod common;

use common::TestFixture;
use dx_media::tools::utility;

// =============================================================================
// 51. hash - File hashing
// =============================================================================

#[test]
fn test_hash_algorithm_enum() {
    let _ = utility::HashAlgorithm::Md5;
    let _ = utility::HashAlgorithm::Sha1;
    let _ = utility::HashAlgorithm::Sha256;
    let _ = utility::HashAlgorithm::Sha512;
}

#[test]
fn test_hash_file() {
    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("test.txt", "Hello World");

    let result = utility::hash_file(&file, utility::HashAlgorithm::Sha256);
    assert!(result.is_ok());
}

#[test]
fn test_sha256() {
    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("test.txt", "hello");

    let result = utility::sha256(&file);
    assert!(result.is_ok());
}

#[test]
fn test_md5() {
    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("test.txt", "hello");

    let result = utility::md5(&file);
    assert!(result.is_ok());
}

#[test]
fn test_multi_hash() {
    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("test.txt", "hello");

    let result = utility::multi_hash(&file);
    assert!(result.is_ok());
}

// =============================================================================
// 52. base64 - Base64 encoding/decoding
// =============================================================================

#[test]
fn test_base64_encode_string() {
    let result = utility::encode_string("Hello World");
    assert!(result.is_ok());
}

#[test]
fn test_base64_decode_string() {
    let result = utility::decode_string("SGVsbG8gV29ybGQ=");
    assert!(result.is_ok());
}

#[test]
fn test_base64_encode_url_safe() {
    let result = utility::encode_url_safe("Hello+World/Test");
    assert!(result.is_ok());
}

#[test]
fn test_base64_file_encoding() {
    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("test.txt", "Test content");

    let result = utility::encode_file(&file);
    assert!(result.is_ok());
}

// =============================================================================
// 53. url_encode - URL encoding/decoding
// =============================================================================

#[test]
fn test_url_encode() {
    let result = utility::encode("hello world");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.message.contains("hello%20world") || output.message.contains("hello+world"));
}

#[test]
fn test_url_decode() {
    let result = utility::decode("hello%20world");
    assert!(result.is_ok());
}

#[test]
fn test_url_encode_component() {
    let result = utility::encode_component("name=value&other=test");
    assert!(result.is_ok());
}

#[test]
fn test_url_encode_path() {
    let result = utility::encode_path("/path/to/file with spaces");
    assert!(result.is_ok());
}

#[test]
fn test_url_parse_query() {
    let result = utility::parse_query_string("name=value&other=test");
    assert!(result.is_ok());
}

// =============================================================================
// 54. json_format - JSON formatting
// =============================================================================

#[test]
fn test_json_format_string() {
    let json = r#"{"name":"test","value":123}"#;
    let result = utility::format_string(json);
    assert!(result.is_ok());
}

#[test]
fn test_json_format_with_indent() {
    let json = r#"{"name":"test"}"#;

    // Test indent variants
    let _ = utility::JsonIndent::None;
    let _ = utility::JsonIndent::Spaces2;
    let _ = utility::JsonIndent::Spaces4;
    let _ = utility::JsonIndent::Tab;

    let result = utility::format_string_with_indent(json, utility::JsonIndent::Spaces4);
    assert!(result.is_ok());
}

#[test]
fn test_json_minify() {
    let json = r#"{
        "name": "test",
        "value": 123
    }"#;
    let result = utility::minify_string(json);
    assert!(result.is_ok());
}

#[test]
fn test_json_validate() {
    let valid_json = r#"{"name":"test"}"#;
    let result = utility::validate_string(valid_json);
    assert!(result.is_ok());
}

#[test]
fn test_json_sort_keys() {
    let json = r#"{"z":"last","a":"first"}"#;
    let result = utility::sort_keys(json);
    assert!(result.is_ok());
}

// =============================================================================
// 55. yaml_convert - YAML/JSON conversion
// =============================================================================

#[test]
fn test_yaml_string_to_json() {
    let yaml = "name: test\nvalue: 123";
    let result = utility::yaml_string_to_json(yaml);
    assert!(result.is_ok());
}

#[test]
fn test_json_string_to_yaml() {
    let json = r#"{"name":"test","value":123}"#;
    let result = utility::json_string_to_yaml(json);
    assert!(result.is_ok());
}

#[test]
fn test_yaml_file_conversion() {
    let fixture = TestFixture::new();
    let yaml_file = fixture.create_test_text_file("test.yaml", "name: test\nvalue: 123");
    let json_file = fixture.path("test.json");

    let result = utility::yaml_to_json(&yaml_file, &json_file);
    let _ = result;
}

// =============================================================================
// 56. csv_convert - CSV conversion
// =============================================================================

#[test]
fn test_csv_to_json_file() {
    let fixture = TestFixture::new();
    let csv_file = fixture.create_test_text_file("test.csv", "name,age\nAlice,30\nBob,25");
    let json_file = fixture.path("test.json");

    let result = utility::csv_to_json(&csv_file, &json_file);
    assert!(result.is_ok());

    let written = std::fs::read_to_string(&json_file).expect("JSON output should be readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&written).expect("CSV conversion should write valid JSON");
    assert_eq!(
        parsed,
        serde_json::json!([
            {"name": "Alice", "age": "30"},
            {"name": "Bob", "age": "25"}
        ])
    );
}

#[test]
fn test_json_to_csv_file() {
    let fixture = TestFixture::new();
    let json_file = fixture.create_test_text_file(
        "test.json",
        r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#,
    );
    let csv_file = fixture.path("test.csv");

    let result = utility::json_to_csv(&json_file, &csv_file);
    assert!(result.is_ok());

    let written = std::fs::read_to_string(&csv_file).expect("CSV output should be readable");
    assert_ne!(
        written.trim(),
        "Note: Full JSON parsing requires serde_json crate"
    );
    assert_eq!(written, "age,name\n30,Alice\n25,Bob\n");
}

#[test]
fn test_csv_options() {
    let options = utility::CsvOptions::default();
    let _ = options;
}

// =============================================================================
// 57. diff - Text diff
// =============================================================================

#[test]
fn test_diff_strings() {
    let text1 = "Hello\nWorld";
    let text2 = "Hello\nRust";
    let result = utility::diff_strings(text1, text2);
    assert!(result.is_ok());
}

#[test]
fn test_diff_files() {
    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello\nWorld");
    let file2 = fixture.create_test_text_file("file2.txt", "Hello\nRust");

    let result = utility::diff_files(&file1, &file2);
    assert!(result.is_ok());
}

#[test]
fn test_diff_format() {
    // Test DiffFormat enum
    let _ = utility::DiffFormat::Unified;
    let _ = utility::DiffFormat::Context;
    let _ = utility::DiffFormat::SideBySide;

    let result = utility::diff_strings_with_format(
        "line1\nline2",
        "line1\nline3",
        utility::DiffFormat::Unified,
    );
    assert!(result.is_ok());
}

#[test]
fn test_files_identical() {
    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Same content");
    let file2 = fixture.create_test_text_file("file2.txt", "Same content");

    let result = utility::files_identical(&file1, &file2);
    assert!(result.is_ok());
}

// =============================================================================
// 58. uuid - UUID generation
// =============================================================================

#[test]
fn test_uuid_generate_v4() {
    let uuid = utility::generate_v4();
    assert!(!uuid.is_empty());
    assert!(uuid.len() == 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
}

#[test]
fn test_uuid_generate() {
    // Test UuidVersion variants
    let _ = utility::UuidVersion::V4;

    let result = utility::generate(utility::UuidVersion::V4);
    assert!(result.is_ok());
}

#[test]
fn test_uuid_validate() {
    let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let result = utility::validate(valid_uuid);
    assert!(result.is_ok());
}

#[test]
fn test_uuid_parse() {
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    let result = utility::parse(uuid);
    assert!(result.is_ok());
}

#[test]
fn test_uuid_batch() {
    let result = utility::generate_batch(5, utility::UuidVersion::V4);
    assert!(result.is_ok());
}

// =============================================================================
// 59. timestamp - Timestamp utilities
// =============================================================================

#[test]
fn test_timestamp_format_enum() {
    let _ = utility::TimestampFormat::Unix;
    let _ = utility::TimestampFormat::Iso8601;
    let _ = utility::TimestampFormat::Rfc2822;
}

#[test]
fn test_timestamp_now() {
    let result = utility::now(utility::TimestampFormat::Iso8601);
    assert!(result.is_ok());
}

#[test]
fn test_timestamp_convert() {
    let result = utility::convert(
        "1609459200",
        utility::TimestampFormat::Unix,
        utility::TimestampFormat::Iso8601,
    );
    let _ = result;
}

// =============================================================================
// 60. random - Random generation
// =============================================================================

#[test]
fn test_charset_enum() {
    let _ = utility::CharSet::Alphanumeric;
    let _ = utility::CharSet::Alphabetic;
    let _ = utility::CharSet::Lowercase;
    let _ = utility::CharSet::Uppercase;
    let _ = utility::CharSet::Numeric;
    let _ = utility::CharSet::Hex;
    let _ = utility::CharSet::Ascii;
}

#[test]
fn test_random_string() {
    let result = utility::string(16, utility::CharSet::Alphanumeric);
    assert!(result.is_ok());
}

#[test]
fn test_random_integer() {
    let result = utility::integer(1, 100);
    assert!(result.is_ok());
}

#[test]
fn test_random_float() {
    let result = utility::float(0.0, 1.0);
    assert!(result.is_ok());
}

#[test]
fn test_random_bytes() {
    let result = utility::bytes(32);
    assert!(result.is_ok());
}

#[test]
fn test_random_boolean() {
    let result = utility::boolean();
    assert!(result.is_ok());
}

#[test]
fn test_random_password() {
    let result = utility::password(16, true);
    assert!(result.is_ok());
}

#[test]
fn test_random_pick() {
    let items = ["apple", "banana", "cherry"];
    let result = utility::pick(&items);
    assert!(result.is_ok());
}

#[test]
fn test_random_shuffle() {
    let items = ["a", "b", "c", "d"];
    let result = utility::shuffle(&items);
    assert!(result.is_ok());
}

#[test]
fn test_random_color() {
    let result = utility::color("hex");
    assert!(result.is_ok());
}

#[test]
fn test_random_batch_integers() {
    let result = utility::batch_integers(10, 1, 100);
    assert!(result.is_ok());
}
