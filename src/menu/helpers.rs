use muda::{MenuItem, CheckMenuItem, accelerator::Accelerator};

pub fn item(id: &str, label: &str) -> MenuItem {
    MenuItem::with_id(id, label, true, None)
}

pub fn item_with_accel(id: &str, label: &str, accelerator: Option<Accelerator>) -> MenuItem {
    MenuItem::with_id(id, label, true, accelerator)
}

pub fn check(id: &str, label: &str, checked: bool) -> CheckMenuItem {
    CheckMenuItem::with_id(id, label, true, checked, None)
}
