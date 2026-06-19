#[cfg(windows)]
pub fn detect_system_locale() -> super::Locale {
    extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }

    const LANG_CHINESE: u16 = 0x04;

    let lang_id = unsafe { GetUserDefaultUILanguage() };
    if lang_id & 0x3FF == LANG_CHINESE {
        super::Locale::ZhHans
    } else {
        super::Locale::En
    }
}

#[cfg(not(windows))]
pub fn detect_system_locale() -> super::Locale {
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_lowercase();

    if lang.starts_with("zh") {
        super::Locale::ZhHans
    } else {
        super::Locale::En
    }
}
