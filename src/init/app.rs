use std::path::Path;

use super::json;
use crate::App;

impl App {
    pub fn try_from_files<P: AsRef<Path>>(
        sim_file_path: P,
        settings_file_path: P,
    ) -> Result<Self, String> {
        let (initial_lander_state, terrain) = json::parse_sim(sim_file_path)?;
        let settings = json::parse_settings(settings_file_path)?;
        Self::try_new(initial_lander_state, terrain, settings)
    }
}
