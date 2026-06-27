use std::cell::RefCell;
use std::fmt;
use std::time::SystemTime;

use chrono::Local;
use fluent::{FluentArgs, FluentResource, FluentValue};
use fluent_bundle::FluentBundle;
use icu_decimal::options::DecimalFormatterOptions;
use icu_decimal::DecimalFormatter;
use icu_locale::Locale as IcuLocale;
use unic_langid::langid;

use super::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    ZhHans,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    System,
    ZhHans,
    En,
}

impl Language {
    pub const ALL: &'static [Self] = &[Self::System, Self::ZhHans, Self::En];

    pub fn resolve(self, system: Locale) -> Locale {
        match self {
            Self::System => system,
            Self::ZhHans => Locale::ZhHans,
            Self::En => Locale::En,
        }
    }

    pub fn message_id(self) -> &'static str {
        match self {
            Self::System => ids::LANGUAGE_SYSTEM,
            Self::ZhHans => ids::LANGUAGE_ZH_HANS,
            Self::En => ids::LANGUAGE_EN,
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZhHans => "zh-Hans",
            Self::En => "en",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageBundle {
    locale: Locale,
}

impl LanguageBundle {
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn locale(self) -> Locale {
        self.locale
    }

    pub fn tr(&self, id: &str) -> String {
        self.format(id, None)
    }

    pub fn tr_with(&self, id: &str, args: FluentArgs) -> String {
        self.format(id, Some(&args))
    }

    pub fn format_count(&self, count: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("count", FluentValue::from(count as i64));
        self.tr_with(ids::STATUS_ITEM_COUNT, args)
    }

    pub fn format_opened(&self, name: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("name", name);
        self.tr_with(ids::STATUS_OPENED, args)
    }

    pub fn format_file_type(&self, extension: &str) -> String {
        let mut args = FluentArgs::new();
        args.set("extension", extension.to_uppercase());
        self.tr_with(ids::ENTRY_FILE_TYPE, args)
    }

    pub fn format_size(&self, bytes: u64) -> String {
        let (value, id, fraction_digits) = size_parts(bytes);
        let formatted_value = self.format_decimal(value, fraction_digits);
        let mut args = FluentArgs::new();
        args.set("value", formatted_value);
        self.tr_with(id, args)
    }

    pub fn format_datetime(&self, time: SystemTime) -> String {
        let datetime = chrono::DateTime::<Local>::from(time);
        match self.locale {
            Locale::ZhHans => datetime.format("%Y年%m月%d日 %H:%M:%S").to_string(),
            Locale::En => datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    fn format_decimal(&self, value: f64, fraction_digits: u8) -> String {
        with_decimal_formatter(self.locale, |formatter| {
            let decimal = if fraction_digits == 0 {
                fixed_decimal::Decimal::from(value.round() as i64)
            } else {
                let scale = 10f64.powi(fraction_digits as i32);
                let mut decimal =
                    fixed_decimal::Decimal::from((value * scale).round() as i64);
                decimal.multiply_pow10(-(fraction_digits as i16));
                decimal
            };
            formatter.format(&decimal).to_string()
        })
        .unwrap_or_else(|| {
            if fraction_digits == 0 {
                format!("{}", value.round() as i64)
            } else {
                format!("{value:.1}")
            }
        })
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        with_fluent_bundle(self.locale, |bundle| {
            let Some(message) = bundle.get_message(id) else {
                return id.to_string();
            };
            let Some(pattern) = message.value() else {
                return id.to_string();
            };

            let mut errors = Vec::new();
            let formatted = bundle.format_pattern(pattern, args, &mut errors);
            if errors.is_empty() {
                formatted.to_string()
            } else {
                id.to_string()
            }
        })
    }
}

fn with_fluent_bundle<R>(locale: Locale, f: impl FnOnce(&FluentBundle<FluentResource>) -> R) -> R {
    thread_local! {
        static CACHE: RefCell<Option<(FluentBundle<FluentResource>, FluentBundle<FluentResource>)>> =
            RefCell::new(None);
    }

    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.is_none() {
            *cache = Some((
                build_fluent_bundle(Locale::En),
                build_fluent_bundle(Locale::ZhHans),
            ));
        }

        let (english, chinese) = cache.as_ref().expect("bundle cache initialized");
        match locale {
            Locale::En => f(english),
            Locale::ZhHans => f(chinese),
        }
    })
}

fn with_decimal_formatter<R>(locale: Locale, f: impl FnOnce(&DecimalFormatter) -> R) -> Option<R> {
    thread_local! {
        static CACHE: RefCell<Option<[DecimalFormatter; 2]>> = RefCell::new(None);
    }

    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.is_none() {
            *cache = Some([
                DecimalFormatter::try_new(
                    icu_locale(Locale::En).into(),
                    DecimalFormatterOptions::default(),
                )
                .ok()?,
                DecimalFormatter::try_new(
                    icu_locale(Locale::ZhHans).into(),
                    DecimalFormatterOptions::default(),
                )
                .ok()?,
            ]);
        }

        cache
            .as_ref()
            .map(|formatters| f(&formatters[locale_index(locale)]))
    })
}

fn build_fluent_bundle(locale: Locale) -> FluentBundle<FluentResource> {
    let lang_id = match locale {
        Locale::En => langid!("en"),
        Locale::ZhHans => langid!("zh-Hans"),
    };
    let ftl = match locale {
        Locale::En => include_str!("../../../locales/en.ftl"),
        Locale::ZhHans => include_str!("../../../locales/zh-Hans.ftl"),
    };

    let mut bundle = FluentBundle::new(vec![lang_id]);
    let resource =
        FluentResource::try_new(ftl.to_string()).expect("invalid Fluent resource file");
    bundle
        .add_resource(resource)
        .expect("failed to add Fluent resource");
    bundle
}

fn icu_locale(locale: Locale) -> IcuLocale {
    match locale {
        Locale::En => IcuLocale::try_from_str("en").expect("valid ICU locale"),
        Locale::ZhHans => IcuLocale::try_from_str("zh-Hans").expect("valid ICU locale"),
    }
}

fn locale_index(locale: Locale) -> usize {
    match locale {
        Locale::En => 0,
        Locale::ZhHans => 1,
    }
}

fn size_parts(bytes: u64) -> (f64, &'static str, u8) {
    const UNITS: [(&str, u8); 5] = [
        (ids::SIZE_BYTE, 0),
        (ids::SIZE_KILOBYTE, 1),
        (ids::SIZE_MEGABYTE, 1),
        (ids::SIZE_GIGABYTE, 1),
        (ids::SIZE_TERABYTE, 1),
    ];

    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        (bytes as f64, UNITS[0].0, UNITS[0].1)
    } else {
        (size, UNITS[unit].0, UNITS[unit].1)
    }
}
