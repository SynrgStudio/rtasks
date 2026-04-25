use anyhow::{Context, Result};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

const OPEN_QUICK_ADD_ID: &str = "open-quick-add";
const OPEN_PANEL_ID: &str = "open-panel";
const QUIT_ID: &str = "quit";
const ICON_SIZE: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    OpenQuickAdd,
    OpenPanel,
    Quit,
}

pub struct AppTray {
    _tray_icon: TrayIcon,
}

impl AppTray {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();
        let open_quick_add = MenuItem::with_id(OPEN_QUICK_ADD_ID, "Open Quick Add", true, None);
        let open_panel = MenuItem::with_id(OPEN_PANEL_ID, "Open Panel", true, None);
        let quit = MenuItem::with_id(QUIT_ID, "Quit", true, None);

        menu.append(&open_quick_add)
            .context("failed to add Open Quick Add tray menu item")?;
        menu.append(&open_panel)
            .context("failed to add Open Panel tray menu item")?;
        menu.append(&quit)
            .context("failed to add Quit tray menu item")?;

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("RTasks")
            .with_icon(r_icon()?)
            .with_menu(Box::new(menu))
            .build()
            .context("failed to create tray icon")?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    pub fn drain_actions(&self) -> Vec<TrayAction> {
        let receiver = MenuEvent::receiver();
        let mut actions = Vec::new();

        while let Ok(event) = receiver.try_recv() {
            match event.id.as_ref() {
                OPEN_QUICK_ADD_ID => actions.push(TrayAction::OpenQuickAdd),
                OPEN_PANEL_ID => actions.push(TrayAction::OpenPanel),
                QUIT_ID => actions.push(TrayAction::Quit),
                _ => {}
            }
        }

        actions
    }
}

fn r_icon() -> Result<Icon> {
    let size = ICON_SIZE as usize;
    let mut rgba = vec![0_u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let index = (y * size + x) * 4;
            let in_glyph = is_r_pixel(x, y);
            let in_accent = y == 0 || x == 0 || x == size - 1 || y == size - 1;

            let (red, green, blue, alpha) = if in_glyph {
                (230, 230, 230, 255)
            } else if in_accent {
                (58, 63, 75, 255)
            } else {
                (40, 44, 52, 255)
            };

            rgba[index] = red;
            rgba[index + 1] = green;
            rgba[index + 2] = blue;
            rgba[index + 3] = alpha;
        }
    }

    Icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).context("failed to create tray icon image")
}

fn is_r_pixel(x: usize, y: usize) -> bool {
    let vertical = (8..=11).contains(&x) && (7..=25).contains(&y);
    let top = (8..=20).contains(&x) && (7..=10).contains(&y);
    let middle = (8..=20).contains(&x) && (15..=18).contains(&y);
    let right_bowl = (19..=22).contains(&x) && (10..=15).contains(&y);
    let diagonal = (17..=24).contains(&x) && (18..=25).contains(&y) && x >= y.saturating_sub(1);

    vertical || top || middle || right_bowl || diagonal
}
