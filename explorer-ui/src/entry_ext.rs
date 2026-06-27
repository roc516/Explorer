use explorer_core::FileEntry;

use crate::i18n::LanguageBundle;

pub trait FileEntryExt {
    fn type_label(&self, bundle: &LanguageBundle) -> String;
    fn size_label(&self, bundle: &LanguageBundle) -> String;
    fn modified_label(&self, bundle: &LanguageBundle) -> String;
}

impl FileEntryExt for FileEntry {
    fn type_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir {
            return bundle.tr(crate::i18n::ids::ENTRY_FOLDER);
        }

        let extension = self.path.extension().or_else(|| {
            std::path::Path::new(&self.name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_ascii_lowercase)
        });
        extension
            .as_deref()
            .map(|ext| bundle.format_file_type(ext))
            .unwrap_or_else(|| bundle.tr(crate::i18n::ids::ENTRY_FILE))
    }

    fn size_label(&self, bundle: &LanguageBundle) -> String {
        if self.is_dir {
            return String::new();
        }
        bundle.format_size(self.size)
    }

    fn modified_label(&self, bundle: &LanguageBundle) -> String {
        self.modified
            .map(|time| bundle.format_datetime(time))
            .unwrap_or_default()
    }
}
