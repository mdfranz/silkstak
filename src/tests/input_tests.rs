use std::collections::HashMap;

use crate::config::QuickModelConfig;
use crate::ui::input::InputEditor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn type_str(editor: &mut InputEditor, s: &str) {
    for c in s.chars() {
        editor.handle_key(press(KeyCode::Char(c)));
    }
}

#[test]
fn typing_ascii_keeps_cursor_in_sync() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "hello");
    assert_eq!(editor.buffer.as_str(), "hello");
    assert_eq!(editor.cursor, 5);
}

#[test]
fn typing_multibyte_chars_does_not_panic() {
    // Regression for bug where `cursor += 1` (char step) was used with
    // `CompactString::insert(byte_idx, ch)` (byte boundary required).
    // Two Norwegian characters in a row were enough to trigger a panic.
    let mut editor = InputEditor::new();
    type_str(&mut editor, "på "); // used to panic on the space after 'å'
    assert_eq!(editor.buffer.as_str(), "på ");
    assert_eq!(editor.cursor, editor.buffer.len()); // cursor in bytes
}

#[test]
fn typing_mixed_ascii_and_multibyte() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "hei på deg så fin dag æøå");
    assert_eq!(editor.buffer.as_str(), "hei på deg så fin dag æøå");
    assert_eq!(editor.cursor, editor.buffer.len());
}

#[test]
fn backspace_after_multibyte_does_not_panic() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "å");
    editor.handle_key(press(KeyCode::Backspace));
    assert_eq!(editor.buffer.as_str(), "");
    assert_eq!(editor.cursor, 0);
}

#[test]
fn left_arrow_steps_one_char_not_one_byte() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "aåb");
    // cursor is after 'b', byte-idx 4 (a=1 + å=2 + b=1)
    assert_eq!(editor.cursor, 4);
    editor.handle_key(press(KeyCode::Left));
    // after 'å' → byte-idx 3
    assert_eq!(editor.cursor, 3);
    editor.handle_key(press(KeyCode::Left));
    // after 'a' → byte-idx 1 (skips the 2 bytes of 'å')
    assert_eq!(editor.cursor, 1);
}

#[test]
fn right_arrow_steps_one_char_not_one_byte() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "aåb");
    editor.cursor = 0;
    editor.handle_key(press(KeyCode::Right));
    assert_eq!(editor.cursor, 1); // after 'a'
    editor.handle_key(press(KeyCode::Right));
    assert_eq!(editor.cursor, 3); // after 'å' (skipped 2 bytes)
}

#[test]
fn enter_returns_buffer_and_resets() {
    let mut editor = InputEditor::new();
    type_str(&mut editor, "hei på");
    let out = editor.handle_key(press(KeyCode::Enter)).unwrap();
    assert_eq!(out.as_str(), "hei på");
    assert_eq!(editor.cursor, 0);
    assert_eq!(editor.buffer.as_str(), "");
}

fn make_qm(provider: &str, model: &str) -> QuickModelConfig {
    QuickModelConfig {
        provider: provider.into(),
        model: model.into(),
        input_token_cost: 0.0,
        output_token_cost: 0.0,
    }
}

#[test]
fn quick_models_filtered_to_current_provider() {
    let mut editor = InputEditor::new();
    let mut qm: HashMap<String, QuickModelConfig> = HashMap::new();
    qm.insert(
        "haiku".to_string(),
        make_qm("anthropic", "claude-haiku-4-5"),
    );
    qm.insert(
        "sonnet".to_string(),
        make_qm("anthropic", "claude-sonnet-4-6"),
    );
    qm.insert("gpt".to_string(), make_qm("openai", "gpt-5"));
    qm.insert(
        "gemini-pro".to_string(),
        make_qm("gemini", "gemini-2.5-pro"),
    );

    editor.update_quick_models_for_provider("anthropic", &qm);
    let mut names = editor.quick_model_names().to_vec();
    names.sort();
    assert_eq!(names, vec!["haiku", "sonnet"]);

    editor.update_quick_models_for_provider("openai", &qm);
    let names = editor.quick_model_names().to_vec();
    assert_eq!(names, vec!["gpt"]);

    editor.update_quick_models_for_provider("gemini", &qm);
    let names = editor.quick_model_names().to_vec();
    assert_eq!(names, vec!["gemini-pro"]);
}

#[test]
fn quick_models_empty_for_unknown_provider() {
    let mut editor = InputEditor::new();
    let mut qm: HashMap<String, QuickModelConfig> = HashMap::new();
    qm.insert(
        "sonnet".to_string(),
        make_qm("anthropic", "claude-sonnet-4-6"),
    );
    editor.update_quick_models_for_provider("ollama", &qm);
    assert!(editor.quick_model_names().is_empty());
}
