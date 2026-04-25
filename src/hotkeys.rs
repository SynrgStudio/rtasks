use anyhow::{Context, Result};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    OpenQuickAdd,
    TogglePanel,
}

pub struct Hotkeys {
    _manager: GlobalHotKeyManager,
    quick_add: HotKey,
    panel: HotKey,
}

impl Hotkeys {
    pub fn register() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("failed to create hotkey manager")?;
        let quick_add = HotKey::new(Some(Modifiers::ALT), Code::Space);
        let panel = HotKey::new(Some(Modifiers::CONTROL), Code::Space);

        manager
            .register(quick_add)
            .context("failed to register Alt+Spacebar for Quick Add")?;
        manager
            .register(panel)
            .context("failed to register Ctrl+Spacebar for Panel")?;

        Ok(Self {
            _manager: manager,
            quick_add,
            panel,
        })
    }

    pub fn drain_actions(&self) -> Vec<HotkeyAction> {
        let receiver = GlobalHotKeyEvent::receiver();
        let mut actions = Vec::new();

        while let Ok(event) = receiver.try_recv() {
            if event.state != HotKeyState::Pressed {
                continue;
            }

            if event.id == self.quick_add.id() {
                actions.push(HotkeyAction::OpenQuickAdd);
            } else if event.id == self.panel.id() {
                actions.push(HotkeyAction::TogglePanel);
            }
        }

        actions
    }
}
