#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandBarMode {
    Find,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveField {
    Find,
    Replace,
}

pub struct CommandBarState {
    pub mode: CommandBarMode,
    pub active: ActiveField,
    find: String,
    replace: String,
    status_cache: String,
}

impl CommandBarState {
    pub fn new_find(initial: Option<&str>) -> Self {
        let mut s = Self {
            mode: CommandBarMode::Find,
            active: ActiveField::Find,
            find: initial.unwrap_or_default().to_string(),
            replace: String::new(),
            status_cache: String::new(),
        };
        s.recompute_status();
        s
    }

    pub fn new_replace(initial_find: Option<&str>) -> Self {
        let mut s = Self {
            mode: CommandBarMode::Replace,
            active: ActiveField::Find,
            find: initial_find.unwrap_or_default().to_string(),
            replace: String::new(),
            status_cache: String::new(),
        };
        s.recompute_status();
        s
    }

    pub fn find_query(&self) -> &str {
        &self.find
    }

    pub fn replace_text(&self) -> &str {
        &self.replace
    }

    pub fn status_text(&self) -> &str {
        &self.status_cache
    }

    pub fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        match self.active {
            ActiveField::Find => self.find.push_str(text),
            ActiveField::Replace => self.replace.push_str(text),
        }
        self.recompute_status();
    }

    pub fn backspace(&mut self) {
        match self.active {
            ActiveField::Find => {
                self.find.pop();
            }
            ActiveField::Replace => {
                self.replace.pop();
            }
        }
        self.recompute_status();
    }

    pub fn toggle_field(&mut self) {
        if self.mode != CommandBarMode::Replace {
            return;
        }
        self.active = match self.active {
            ActiveField::Find => ActiveField::Replace,
            ActiveField::Replace => ActiveField::Find,
        };
        self.recompute_status();
    }

    fn recompute_status(&mut self) {
        self.status_cache.clear();
        match self.mode {
            CommandBarMode::Find => {
                self.status_cache.push_str("Find: ");
                self.status_cache.push_str(&self.find);
            }
            CommandBarMode::Replace => {
                self.status_cache.push_str("Find: ");
                self.status_cache.push_str(&self.find);
                self.status_cache.push_str("    Replace: ");
                self.status_cache.push_str(&self.replace);
            }
        }

        match self.mode {
            CommandBarMode::Find => {}
            CommandBarMode::Replace => {
                // crude active-field indicator that doesn't require extra UI geometry
                match self.active {
                    ActiveField::Find => self.status_cache.push_str("   [editing find]"),
                    ActiveField::Replace => self.status_cache.push_str("   [editing replace]"),
                }
            }
        }

        self.status_cache
            .push_str("    (Enter=apply, Esc=close)");
    }
}

