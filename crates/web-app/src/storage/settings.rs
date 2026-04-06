use super::schema::ReaderSettings;

#[derive(Debug, Clone, Default)]
pub struct SettingsRecords {
    settings: ReaderSettings,
}

impl SettingsRecords {
    pub fn save(&mut self, settings: ReaderSettings) {
        self.settings = settings;
    }

    pub fn load(&self) -> ReaderSettings {
        self.settings.clone()
    }
}
