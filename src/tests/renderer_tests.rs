use crossterm::style::Color;

use crate::ui::renderer::{Renderer, base64_encode, contrasting_text_color, copy_to_clipboard};

#[test]
fn base64_encode_empty() {
    assert_eq!(base64_encode(b""), "");
}

#[test]
fn base64_encode_single_byte() {
    assert_eq!(base64_encode(b"f"), "Zg==");
}

#[test]
fn base64_encode_two_bytes() {
    assert_eq!(base64_encode(b"fo"), "Zm8=");
}

#[test]
fn base64_encode_three_bytes() {
    assert_eq!(base64_encode(b"foo"), "Zm9v");
}

#[test]
fn base64_encode_known_values() {
    assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    assert_eq!(base64_encode(b"Hi!"), "SGkh");
    assert_eq!(base64_encode(b"ab"), "YWI=");
    assert_eq!(base64_encode(b"abc"), "YWJj");
    assert_eq!(base64_encode(b"Man"), "TWFu");
}

#[test]
fn base64_encode_long_input() {
    let input = "The quick brown fox jumps over the lazy dog. ".repeat(10);
    let encoded = base64_encode(input.as_bytes());
    assert!(encoded.len() > input.len());
    assert!(encoded.ends_with('=') || !encoded.contains('='));
}

#[test]
fn copy_to_clipboard_does_not_panic() {
    copy_to_clipboard("test text");
}

#[test]
fn copy_to_clipboard_empty_string() {
    copy_to_clipboard("");
}

#[test]
fn renderer_defaults_to_terminal_foreground_for_text() {
    let renderer = Renderer::new().unwrap();
    assert_eq!(renderer.text_color(), Color::Reset);
}

#[test]
fn renderer_infers_light_text_for_dark_background_theme() {
    let mut renderer = Renderer::new().unwrap();
    renderer.set_background_colors(
        Some(Color::Rgb {
            r: 0x28,
            g: 0x2c,
            b: 0x34,
        }),
        Some(Color::Rgb {
            r: 0x2c,
            g: 0x31,
            b: 0x3c,
        }),
        Some(Color::Rgb {
            r: 0x21,
            g: 0x25,
            b: 0x2b,
        }),
    );
    assert_eq!(renderer.text_color(), Color::White);
}

#[test]
fn contrast_helper_picks_dark_text_for_light_backgrounds() {
    assert_eq!(
        contrasting_text_color(Color::Rgb {
            r: 0xf8,
            g: 0xf8,
            b: 0xf2,
        }),
        Color::Black
    );
}
