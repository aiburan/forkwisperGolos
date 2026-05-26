use std::path::PathBuf;

/// Active transcription backend.
#[derive(Clone, Copy, PartialEq)]
pub enum TranscriptionService {
    Api,
    Local,
}

/// Global mouse button used to toggle recording on Windows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseHotkey {
    Disabled,
    Middle,
    XButton1,
    XButton2,
}

impl MouseHotkey {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "disabled" | "off" | "none" => Some(Self::Disabled),
            "middle" | "mbutton" | "mouse3" => Some(Self::Middle),
            "xbutton1" | "mouse4" | "button4" => Some(Self::XButton1),
            "xbutton2" | "mouse5" | "button5" => Some(Self::XButton2),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Middle => "middle",
            Self::XButton1 => "xbutton1",
            Self::XButton2 => "xbutton2",
        }
    }
}

/// Built-in API provider configuration.
pub struct ApiPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub base_url: &'static str,
    pub default_model: &'static str,
    pub needs_key: bool,
}

/// Pre-configured API providers (Groq, Ollama, OpenRouter, LM Studio).
pub const API_PRESETS: &[ApiPreset] = &[
    ApiPreset {
        id: "groq",
        label: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        default_model: "whisper-large-v3-turbo",
        needs_key: true,
    },
    ApiPreset {
        id: "ollama",
        label: "Ollama",
        base_url: "http://localhost:11434/v1",
        default_model: "whisper",
        needs_key: false,
    },
    ApiPreset {
        id: "openrouter",
        label: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        default_model: "openai/whisper-1",
        needs_key: true,
    },
    ApiPreset {
        id: "lmstudio",
        label: "LM Studio",
        base_url: "http://localhost:1234/v1",
        default_model: "whisper-1",
        needs_key: false,
    },
];

/// Look up an API preset by its short identifier.
pub fn find_preset(id: &str) -> Option<&'static ApiPreset> {
    API_PRESETS.iter().find(|p| p.id == id)
}

/// Built-in local whisper model preset.
pub struct LocalModelPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub file_name: &'static str,
    pub size_label: &'static str,
}

/// Available local whisper model sizes (Tiny through Medium).
pub const LOCAL_MODEL_PRESETS: &[LocalModelPreset] = &[
    LocalModelPreset {
        id: "local-tiny",
        label: "Tiny Multilingual",
        file_name: "ggml-tiny.bin",
        size_label: "~78 MB",
    },
    LocalModelPreset {
        id: "local-base",
        label: "Base Multilingual",
        file_name: "ggml-base.bin",
        size_label: "~148 MB",
    },
    LocalModelPreset {
        id: "local-small",
        label: "Small Multilingual",
        file_name: "ggml-small.bin",
        size_label: "~488 MB",
    },
    LocalModelPreset {
        id: "local-medium",
        label: "Medium Multilingual",
        file_name: "ggml-medium.bin",
        size_label: "~1.53 GB",
    },
];

/// Default local model preset ID.
pub const DEFAULT_LOCAL_MODEL: &str = "local-tiny";

/// Look up a local model preset by its short identifier.
pub fn find_local_model(id: &str) -> Option<&'static LocalModelPreset> {
    LOCAL_MODEL_PRESETS.iter().find(|m| m.id == id)
}

/// Build the HuggingFace download URL for a whisper model file.
pub fn model_url(file_name: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        file_name
    )
}

// ── TTS (text-to-speech) ────────────────────────────────────────────────────

/// Active TTS provider.
#[derive(Clone, Copy, PartialEq)]
pub enum TtsProvider {
    None,
    Piper,
}

/// A Piper voice preset.
pub struct PiperVoice {
    pub id: &'static str,
    pub label: &'static str,
    pub locale: &'static str,
    pub name: &'static str,
    pub quality: &'static str,
}

impl PiperVoice {
    /// HuggingFace URL for the ONNX model file.
    pub fn onnx_url(&self) -> String {
        format!(
            "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/{}/{}/{}/{}-{}-{}.onnx",
            self.locale, self.name, self.quality, self.locale, self.name, self.quality
        )
    }
    /// HuggingFace URL for the ONNX config file.
    pub fn config_url(&self) -> String {
        format!("{}.json", self.onnx_url())
    }
}

/// Available Piper voice presets.
pub const PIPER_VOICES: &[PiperVoice] = &[
    PiperVoice {
        id: "amy",
        label: "Amy (US Female)",
        locale: "en_US",
        name: "amy",
        quality: "medium",
    },
    PiperVoice {
        id: "lessac",
        label: "Lessac (US Female)",
        locale: "en_US",
        name: "lessac",
        quality: "medium",
    },
    PiperVoice {
        id: "ryan",
        label: "Ryan (US Male)",
        locale: "en_US",
        name: "ryan",
        quality: "medium",
    },
    PiperVoice {
        id: "kristin",
        label: "Kristin (US Female)",
        locale: "en_US",
        name: "kristin",
        quality: "medium",
    },
    PiperVoice {
        id: "joe",
        label: "Joe (US Male)",
        locale: "en_US",
        name: "joe",
        quality: "medium",
    },
    PiperVoice {
        id: "cori",
        label: "Cori (UK Female)",
        locale: "en_GB",
        name: "cori",
        quality: "medium",
    },
];

/// Default voice ID.
pub const DEFAULT_PIPER_VOICE: &str = "amy";

/// Look up a Piper voice preset by ID.
pub fn find_piper_voice(id: &str) -> Option<&'static PiperVoice> {
    PIPER_VOICES.iter().find(|v| v.id == id)
}

/// Check if Piper venv is installed.
pub fn piper_venv_exists(piper_dir: &std::path::Path) -> bool {
    piper_dir.join("venv/bin/piper").exists()
}

/// Check if a specific voice model is downloaded.
pub fn piper_voice_exists(piper_dir: &std::path::Path, voice_id: &str) -> bool {
    piper_dir.join(format!("{voice_id}.onnx")).exists()
        && piper_dir.join(format!("{voice_id}.onnx.json")).exists()
}

/// Application configuration loaded from environment and `.env` file.
pub struct Config {
    pub transcription_service: TranscriptionService,
    pub api_base_url: String,
    pub api_key: Option<String>,
    pub api_model: String,
    pub db_path: PathBuf,
    pub models_dir: PathBuf,
    pub sound_notification: bool,
    pub mouse_hotkey: MouseHotkey,
    pub consume_mouse_hotkey: bool,
    pub auto_paste_after_transcribe: bool,
}

impl Config {
    pub fn load() -> Self {
        // Try loading .env from current dir, ignore if missing
        let _ = dotenvy::dotenv();

        let transcription_service = match std::env::var("PRIMARY_TRANSCRIPTION_SERVICE")
            .unwrap_or_else(|_| "api".into())
            .to_lowercase()
            .as_str()
        {
            "local" => TranscriptionService::Local,
            // Accept "api" and legacy "groq"
            _ => TranscriptionService::Api,
        };

        // API_BASE_URL with default pointing to Groq (backwards compatible)
        let api_base_url = std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| "https://api.groq.com/openai/v1".into());

        // API_KEY with GROQ_API_KEY as legacy fallback
        let api_key = std::env::var("API_KEY")
            .or_else(|_| std::env::var("GROQ_API_KEY"))
            .ok();

        // API_MODEL with GROQ_STT_MODEL as legacy fallback
        let api_model = std::env::var("API_MODEL")
            .or_else(|_| std::env::var("GROQ_STT_MODEL"))
            .unwrap_or_else(|_| "whisper-large-v3-turbo".into());

        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whispercrabs");
        std::fs::create_dir_all(&data_dir).ok();

        // Restrict directory permissions on Unix (owner-only: rwx------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700));
        }

        let db_path = data_dir.join("history.db");

        let models_dir = data_dir.join("models");
        std::fs::create_dir_all(&models_dir).ok();

        let sound_notification = std::env::var("SOUND_NOTIFICATION_ON_COMPLETION")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let mouse_hotkey = env_any(&["MOUSE_HOTKEY", "mouse_hotkey"])
            .as_deref()
            .and_then(MouseHotkey::parse)
            .unwrap_or(MouseHotkey::XButton1);

        let consume_mouse_hotkey = env_any(&["CONSUME_MOUSE_HOTKEY", "consume_mouse_hotkey"])
            .map(|v| parse_bool(&v))
            .unwrap_or(true);

        let auto_paste_after_transcribe =
            env_any(&["AUTO_PASTE_AFTER_TRANSCRIBE", "auto_paste_after_transcribe"])
                .map(|v| parse_auto_paste(&v))
                .unwrap_or(default_auto_paste_after_transcribe());

        Self {
            transcription_service,
            api_base_url,
            api_key,
            api_model,
            db_path,
            models_dir,
            sound_notification,
            mouse_hotkey,
            consume_mouse_hotkey,
            auto_paste_after_transcribe,
        }
    }
}

fn env_any(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| std::env::var(key).ok())
}

fn parse_bool(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("yes")
        || value.eq_ignore_ascii_case("on")
        || value == "1"
}

pub fn parse_auto_paste(value: &str) -> bool {
    let value = value.trim();
    !(value.eq_ignore_ascii_case("false")
        || value.eq_ignore_ascii_case("off")
        || value.eq_ignore_ascii_case("no")
        || value == "0")
}

fn default_auto_paste_after_transcribe() -> bool {
    cfg!(target_os = "windows")
}
