use crate::config;

#[test]
fn find_preset_returns_known_providers() {
    let groq = config::find_preset("groq");
    assert!(groq.is_some());
    let groq = groq.unwrap();
    assert_eq!(groq.id, "groq");
    assert!(groq.needs_key);
    assert!(!groq.base_url.is_empty());

    let ollama = config::find_preset("ollama");
    assert!(ollama.is_some());
    assert!(!ollama.unwrap().needs_key);
}

#[test]
fn find_preset_returns_none_for_unknown() {
    assert!(config::find_preset("nonexistent").is_none());
}

#[test]
fn all_api_presets_have_required_fields() {
    for preset in config::API_PRESETS {
        assert!(!preset.id.is_empty(), "preset id must not be empty");
        assert!(!preset.label.is_empty(), "preset label must not be empty");
        assert!(
            preset.base_url.starts_with("http"),
            "preset {} base_url must start with http",
            preset.id
        );
        assert!(
            !preset.default_model.is_empty(),
            "preset {} default_model must not be empty",
            preset.id
        );
    }
}

#[test]
fn find_local_model_returns_known_models() {
    let tiny = config::find_local_model("local-tiny");
    assert!(tiny.is_some());
    let tiny = tiny.unwrap();
    assert_eq!(tiny.file_name, "ggml-tiny.bin");
    assert_eq!(tiny.label, "Tiny Multilingual");

    let medium = config::find_local_model("local-medium");
    assert!(medium.is_some());
}

#[test]
fn local_model_presets_use_multilingual_files() {
    let expected = [
        (
            "local-tiny",
            "Tiny Multilingual",
            "ggml-tiny.bin",
            "~78 MB",
        ),
        (
            "local-base",
            "Base Multilingual",
            "ggml-base.bin",
            "~148 MB",
        ),
        (
            "local-small",
            "Small Multilingual",
            "ggml-small.bin",
            "~488 MB",
        ),
        (
            "local-medium",
            "Medium Multilingual",
            "ggml-medium.bin",
            "~1.53 GB",
        ),
    ];

    for (id, label, file_name, size_label) in expected {
        let model = config::find_local_model(id).expect("missing local model preset");
        assert_eq!(model.label, label);
        assert_eq!(model.file_name, file_name);
        assert_eq!(model.size_label, size_label);
    }
}

#[test]
fn find_local_model_returns_none_for_unknown() {
    assert!(config::find_local_model("local-nonexistent").is_none());
}

#[test]
fn all_local_model_presets_have_required_fields() {
    for model in config::LOCAL_MODEL_PRESETS {
        assert!(!model.id.is_empty());
        assert!(!model.label.is_empty());
        assert!(
            model.file_name.ends_with(".bin"),
            "model {} file_name should end with .bin",
            model.id
        );
        assert!(!model.size_label.is_empty());
    }
}

#[test]
fn default_local_model_is_valid() {
    assert!(config::find_local_model(config::DEFAULT_LOCAL_MODEL).is_some());
}

#[test]
fn model_url_produces_valid_huggingface_url() {
    let url = config::model_url("ggml-tiny.bin");
    assert!(url.starts_with("https://huggingface.co/"));
    assert!(url.ends_with("ggml-tiny.bin"));
}

#[test]
fn mouse_hotkey_parse_accepts_supported_values() {
    assert_eq!(
        config::MouseHotkey::parse("middle"),
        Some(config::MouseHotkey::Middle)
    );
    assert_eq!(
        config::MouseHotkey::parse("xbutton1"),
        Some(config::MouseHotkey::XButton1)
    );
    assert_eq!(
        config::MouseHotkey::parse("xbutton2"),
        Some(config::MouseHotkey::XButton2)
    );
    assert_eq!(
        config::MouseHotkey::parse("disabled"),
        Some(config::MouseHotkey::Disabled)
    );
}

#[test]
fn default_whisper_language_is_russian() {
    assert_eq!(
        config::WhisperLanguage::default(),
        config::WhisperLanguage::Ru
    );
}

#[test]
fn whisper_language_parse_accepts_supported_values() {
    assert_eq!(
        config::WhisperLanguage::parse("ru"),
        Some(config::WhisperLanguage::Ru)
    );
    assert_eq!(
        config::WhisperLanguage::parse("en"),
        Some(config::WhisperLanguage::En)
    );
    assert_eq!(
        config::WhisperLanguage::parse("kk"),
        Some(config::WhisperLanguage::Kk)
    );
    assert_eq!(
        config::WhisperLanguage::parse("auto"),
        Some(config::WhisperLanguage::Auto)
    );
}

#[test]
fn whisper_language_codes_match_local_whisper_params() {
    assert_eq!(config::WhisperLanguage::Ru.whisper_code(), Some("ru"));
    assert_eq!(config::WhisperLanguage::En.whisper_code(), Some("en"));
    assert_eq!(config::WhisperLanguage::Kk.whisper_code(), Some("kk"));
    assert_eq!(config::WhisperLanguage::Auto.whisper_code(), None);
}

#[test]
fn invalid_whisper_language_falls_back_to_default() {
    assert_eq!(
        config::WhisperLanguage::parse_or_default("de"),
        config::WhisperLanguage::Ru
    );
}

#[test]
fn mouse_hotkey_parse_rejects_scroll_wheel_values() {
    assert_eq!(config::MouseHotkey::parse("wheelup"), None);
    assert_eq!(config::MouseHotkey::parse("wheeldown"), None);
}

#[test]
fn auto_paste_parse_disables_false_like_values() {
    assert!(!config::parse_auto_paste("false"));
    assert!(!config::parse_auto_paste("off"));
    assert!(!config::parse_auto_paste("0"));
    assert!(!config::parse_auto_paste("no"));
}

#[test]
fn auto_paste_parse_keeps_true_for_other_values() {
    assert!(config::parse_auto_paste("true"));
    assert!(config::parse_auto_paste("on"));
    assert!(config::parse_auto_paste("1"));
    assert!(config::parse_auto_paste(""));
}
