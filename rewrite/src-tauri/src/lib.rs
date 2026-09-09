use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, Runtime, WebviewWindow, WindowEvent};

pub mod codex_provider;
pub mod usage_contract;
pub mod usage_repository;

const MAIN_WINDOW: &str = "main";
const TRAY_ID: &str = "main-tray";
const EXIT_MENU_ID: &str = "exit";
const PANEL_WIDTH: i32 = 400;
const PANEL_HEIGHT: i32 = 228;
const INSTANCE_ADDRESS: &str = "127.0.0.1:47619";

type SharedController = Arc<Mutex<Controller>>;
type SharedCodexProvider = Arc<Mutex<codex_provider::CodexProvider>>;

fn claim_instance() -> std::io::Result<Option<TcpListener>> {
    match TcpListener::bind(INSTANCE_ADDRESS) {
        Ok(listener) => Ok(Some(listener)),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            TcpStream::connect(INSTANCE_ADDRESS)?.write_all(&[1])?;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LifecycleState {
    #[default]
    Hidden,
    Visible,
    Exiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayAction {
    Show,
    Hide,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelSize {
    width: i32,
    height: i32,
}

#[derive(Debug, Default)]
struct Controller {
    state: LifecycleState,
    focus_loss_anchor: Option<Point>,
    tray_press_bounds: Option<WorkArea>,
    just_shown: bool,
    last_position: Option<Point>,
}

impl Controller {
    fn tray_press(&mut self, bounds: WorkArea) {
        self.tray_press_bounds = Some(bounds);
    }

    fn cancel_tray_interaction(&mut self) {
        self.focus_loss_anchor = None;
        self.tray_press_bounds = None;
    }

    fn tray_click(&mut self) -> TrayAction {
        let focus_loss_was_this_click = self
            .tray_press_bounds
            .take()
            .is_some_and(|bounds| self.focus_loss_anchor.is_some_and(|p| bounds.contains(p)));
        match self.state {
            LifecycleState::Exiting => TrayAction::Ignore,
            LifecycleState::Visible => {
                self.state = LifecycleState::Hidden;
                self.focus_loss_anchor = None;
                self.just_shown = false;
                TrayAction::Hide
            }
            LifecycleState::Hidden => {
                self.focus_loss_anchor = None;
                if focus_loss_was_this_click {
                    TrayAction::Ignore
                } else {
                    self.state = LifecycleState::Visible;
                    self.just_shown = true;
                    TrayAction::Show
                }
            }
        }
    }

    fn focus_gained(&mut self) {
        self.just_shown = false;
    }

    fn focus_lost(&mut self, cursor: Option<Point>) -> bool {
        if self.state != LifecycleState::Visible {
            return false;
        }

        self.state = LifecycleState::Hidden;
        if self.just_shown {
            self.focus_loss_anchor = None;
        } else {
            self.focus_loss_anchor = cursor;
        }
        self.just_shown = false;
        true
    }

    fn close_requested(&mut self) -> bool {
        if self.state == LifecycleState::Exiting {
            return false;
        }
        self.state = LifecycleState::Hidden;
        self.focus_loss_anchor = None;
        self.tray_press_bounds = None;
        self.just_shown = false;
        true
    }

    fn activate_existing(&mut self) -> bool {
        if self.state == LifecycleState::Exiting {
            return false;
        }
        self.state = LifecycleState::Visible;
        self.focus_loss_anchor = None;
        self.tray_press_bounds = None;
        self.just_shown = true;
        true
    }

    fn exit(&mut self) -> bool {
        if self.state == LifecycleState::Exiting {
            return false;
        }
        self.state = LifecycleState::Exiting;
        self.focus_loss_anchor = None;
        self.tray_press_bounds = None;
        self.just_shown = false;
        true
    }

    fn remember_position(&mut self, position: Point) {
        self.last_position = Some(position);
    }

    fn last_position(&self) -> Option<Point> {
        self.last_position
    }
}

impl WorkArea {
    fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

fn rounded_point(position: PhysicalPosition<f64>) -> Point {
    Point {
        x: position.x.round() as i32,
        y: position.y.round() as i32,
    }
}

fn physical_rect(rect: tauri::Rect) -> Option<WorkArea> {
    match (rect.position, rect.size) {
        (tauri::Position::Physical(position), tauri::Size::Physical(size)) => Some(WorkArea {
            x: position.x,
            y: position.y,
            width: size.width as i32,
            height: size.height as i32,
        }),
        _ => None,
    }
}

fn panel_size(scale_factor: f64) -> PanelSize {
    PanelSize {
        width: (PANEL_WIDTH as f64 * scale_factor).round() as i32,
        height: (PANEL_HEIGHT as f64 * scale_factor).round() as i32,
    }
}

fn clamp_origin(origin: i32, start: i32, available: i32, panel: i32) -> i32 {
    let max = (start as i64 + available as i64 - panel as i64).max(start as i64);
    (origin as i64).clamp(start as i64, max) as i32
}

fn panel_position(anchor: Point, work_area: WorkArea, scale_factor: f64) -> Point {
    let size = panel_size(scale_factor);
    let desired = Point {
        x: anchor.x - size.width / 2,
        y: anchor.y - size.height,
    };
    clamp_panel_position(desired, work_area, scale_factor)
}

fn clamp_panel_position(origin: Point, work_area: WorkArea, scale_factor: f64) -> Point {
    let size = panel_size(scale_factor);
    Point {
        x: clamp_origin(origin.x, work_area.x, work_area.width, size.width),
        y: clamp_origin(origin.y, work_area.y, work_area.height, size.height),
    }
}

fn monitor_work_area(monitor: &tauri::Monitor) -> WorkArea {
    let area = monitor.work_area();
    WorkArea {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width as i32,
        height: area.size.height as i32,
    }
}

fn set_position_from_anchor<R: Runtime>(
    window: &WebviewWindow<R>,
    controller: &SharedController,
    anchor: Point,
) -> tauri::Result<()> {
    let monitor = window
        .monitor_from_point(anchor.x as f64, anchor.y as f64)?
        .or(window.current_monitor()?);
    if let Some(monitor) = monitor {
        let position = panel_position(anchor, monitor_work_area(&monitor), monitor.scale_factor());
        window.set_position(PhysicalPosition::new(position.x, position.y))?;
        controller.lock().unwrap().remember_position(position);
    }
    Ok(())
}

fn set_position_from_last_or_cursor<R: Runtime>(
    window: &WebviewWindow<R>,
    controller: &SharedController,
) -> tauri::Result<()> {
    let last_position = controller.lock().unwrap().last_position();
    if let Some(origin) = last_position {
        if let Some(monitor) = window
            .monitor_from_point(origin.x as f64, origin.y as f64)?
            .or(window.current_monitor()?)
        {
            let position =
                clamp_panel_position(origin, monitor_work_area(&monitor), monitor.scale_factor());
            window.set_position(PhysicalPosition::new(position.x, position.y))?;
            controller.lock().unwrap().remember_position(position);
        }
    } else if let Ok(cursor) = window.cursor_position() {
        set_position_from_anchor(window, controller, rounded_point(cursor))?;
    }
    Ok(())
}

fn show_window<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.show()?;
    window.set_focus()
}

fn handle_tray_click<R: Runtime>(
    app: &AppHandle<R>,
    controller: &SharedController,
    position: PhysicalPosition<f64>,
) {
    let point = rounded_point(position);
    let action = controller.lock().unwrap().tray_click();
    match action {
        TrayAction::Show => {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = set_position_from_anchor(&window, controller, point);
                if show_window(&window).is_err() {
                    let _ = window.hide();
                }
            }
        }
        TrayAction::Hide => {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.hide();
            }
        }
        TrayAction::Ignore => {}
    }
}

fn activate_existing<R: Runtime>(app: &AppHandle<R>, controller: &SharedController) {
    let should_activate = {
        let mut controller = controller.lock().unwrap();
        controller.activate_existing()
    };
    if !should_activate {
        return;
    }
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        if set_position_from_last_or_cursor(&window, controller)
            .and_then(|()| show_window(&window))
            .is_err()
        {
            let _ = window.hide();
        }
    }
}

fn exit_application<R: Runtime>(app: &AppHandle<R>, controller: &SharedController) {
    if controller.lock().unwrap().exit() {
        app.state::<SharedCodexProvider>()
            .lock()
            .unwrap()
            .shutdown();
        let _ = app.remove_tray_by_id(TRAY_ID);
        app.exit(0);
    }
}

#[tauri::command]
fn get_usage(
    app: AppHandle,
    provider: tauri::State<'_, SharedCodexProvider>,
) -> usage_contract::UsageSnapshot {
    let Ok(data_dir) = app.path().app_data_dir() else {
        let mut provider = provider.lock().unwrap();
        let mut repository = usage_repository::UsageRepository::default();
        return provider.usage(&mut repository);
    };
    let path = data_dir.join("usage-snapshots.json");
    let mut provider = provider.lock().unwrap();
    let mut repository = usage_repository::UsageRepository::load(&path);
    let snapshot = provider.usage(&mut repository);
    let _ = repository.save(&path);
    snapshot
}

fn open_auth_url(auth_url: &str) -> Result<(), String> {
    Command::new("explorer.exe")
        .arg(auth_url)
        .spawn()
        .map(|_| ())
        .map_err(|_| "Could not open the browser for Codex sign-in.".into())
}

#[tauri::command]
fn start_codex_login(provider: tauri::State<'_, SharedCodexProvider>) -> Result<(), String> {
    let auth_url = provider
        .lock()
        .unwrap()
        .start_login()
        .map_err(|error| error.to_string())?;
    if let Err(error) = open_auth_url(&auth_url) {
        let _ = provider.lock().unwrap().cancel_login();
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
fn poll_codex_login(
    provider: tauri::State<'_, SharedCodexProvider>,
) -> Result<codex_provider::LoginStatus, String> {
    provider
        .lock()
        .unwrap()
        .poll_login()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_codex_login(provider: tauri::State<'_, SharedCodexProvider>) -> Result<(), String> {
    provider
        .lock()
        .unwrap()
        .cancel_login()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn logout_codex(provider: tauri::State<'_, SharedCodexProvider>) -> Result<(), String> {
    provider
        .lock()
        .unwrap()
        .logout()
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let Some(instance_listener) = claim_instance().expect("failed to claim app instance") else {
        return;
    };
    let controller = Arc::new(Mutex::new(Controller::default()));

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(
            codex_provider::CodexProvider::default(),
        )))
        .setup({
            let controller = Arc::clone(&controller);
            move |app| {
                let window = app
                    .get_webview_window(MAIN_WINDOW)
                    .expect("main window is declared in tauri.conf.json");
                window.hide()?;

                {
                    let controller = Arc::clone(&controller);
                    let event_window = window.clone();
                    window.on_window_event(move |event| match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            if controller.lock().unwrap().close_requested() {
                                api.prevent_close();
                                let _ = event_window.hide();
                            }
                        }
                        WindowEvent::Focused(true) => controller.lock().unwrap().focus_gained(),
                        WindowEvent::Focused(false) => {
                            let cursor = event_window.cursor_position().ok().map(rounded_point);
                            if controller.lock().unwrap().focus_lost(cursor) {
                                let _ = event_window.hide();
                            }
                        }
                        _ => {}
                    });
                }

                let quit = MenuItem::with_id(app, EXIT_MENU_ID, "Sair", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&quit])?;
                let tray_controller = Arc::clone(&controller);
                let click_controller = Arc::clone(&controller);
                TrayIconBuilder::with_id(TRAY_ID)
                    .icon(
                        app.default_window_icon()
                            .cloned()
                            .expect("default tray icon"),
                    )
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(move |app, event| {
                        if event.id() == EXIT_MENU_ID {
                            exit_application(app, &tray_controller);
                        }
                    })
                    .on_tray_icon_event(move |tray, event| match event {
                        TrayIconEvent::Click {
                            position,
                            rect,
                            button: MouseButton::Left,
                            button_state,
                            ..
                        } => match button_state {
                            MouseButtonState::Down => {
                                if let Some(bounds) = physical_rect(rect) {
                                    click_controller.lock().unwrap().tray_press(bounds);
                                }
                            }
                            MouseButtonState::Up => {
                                handle_tray_click(tray.app_handle(), &click_controller, position);
                            }
                        },
                        TrayIconEvent::Click { .. } => {
                            click_controller.lock().unwrap().cancel_tray_interaction();
                        }
                        _ => {}
                    })
                    .build(app)?;

                let activation_app = app.handle().clone();
                let activation_controller = Arc::clone(&controller);
                thread::spawn(move || {
                    for connection in instance_listener.incoming() {
                        if connection.is_err() {
                            break;
                        }
                        let app = activation_app.clone();
                        let controller = Arc::clone(&activation_controller);
                        let _ = activation_app.run_on_main_thread(move || {
                            activate_existing(&app, &controller);
                        });
                    }
                });

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_usage,
            start_codex_login,
            poll_codex_login,
            cancel_codex_login,
            logout_codex
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contains(origin: Point, work_area: WorkArea, scale_factor: f64) -> bool {
        let size = panel_size(scale_factor);
        origin.x >= work_area.x
            && origin.y >= work_area.y
            && origin.x + size.width <= work_area.x + work_area.width
            && origin.y + size.height <= work_area.y + work_area.height
    }

    #[test]
    fn placement_contains_panel_at_requested_scales_and_edges() {
        let work_areas = [
            WorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040,
            },
            WorkArea {
                x: -1920,
                y: -120,
                width: 1920,
                height: 1080,
            },
        ];

        for scale_factor in [1.0, 1.25, 1.5] {
            for work_area in work_areas {
                for anchor in [
                    Point {
                        x: work_area.x,
                        y: work_area.y,
                    },
                    Point {
                        x: work_area.x + work_area.width,
                        y: work_area.y,
                    },
                    Point {
                        x: work_area.x,
                        y: work_area.y + work_area.height,
                    },
                    Point {
                        x: work_area.x + work_area.width,
                        y: work_area.y + work_area.height,
                    },
                ] {
                    assert!(contains(
                        panel_position(anchor, work_area, scale_factor),
                        work_area,
                        scale_factor
                    ));
                }
            }
        }
    }

    #[test]
    fn placement_respects_taskbar_and_target_monitor_changes() {
        let taskbar_area = WorkArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1040,
        };
        let secondary_area = WorkArea {
            x: -1600,
            y: 80,
            width: 1600,
            height: 820,
        };

        let first = panel_position(Point { x: 1900, y: 1030 }, taskbar_area, 1.25);
        let second = panel_position(Point { x: -10, y: 100 }, secondary_area, 1.5);

        assert!(contains(first, taskbar_area, 1.25));
        assert!(contains(second, secondary_area, 1.5));
        assert_ne!(first, second);
    }

    #[test]
    fn lifecycle_handles_hidden_startup_close_focus_and_exit() {
        let mut controller = Controller::default();

        assert_eq!(controller.state, LifecycleState::Hidden);
        assert_eq!(controller.tray_click(), TrayAction::Show);
        assert_eq!(controller.state, LifecycleState::Visible);
        assert!(controller.close_requested());
        assert_eq!(controller.state, LifecycleState::Hidden);
        assert_eq!(controller.tray_click(), TrayAction::Show);
        assert!(controller.exit());
        assert_eq!(controller.tray_click(), TrayAction::Ignore);
        assert!(!controller.close_requested());
    }

    #[test]
    fn focus_loss_before_tray_click_is_correlated_without_a_timer() {
        let mut controller = Controller::default();
        let point = Point { x: 500, y: 600 };

        assert_eq!(controller.tray_click(), TrayAction::Show);
        controller.focus_gained();
        assert!(controller.focus_lost(Some(point)));
        controller.tray_press(WorkArea {
            x: 490,
            y: 590,
            width: 40,
            height: 40,
        });
        assert_eq!(controller.tray_click(), TrayAction::Ignore);
        assert_eq!(controller.tray_click(), TrayAction::Show);
    }

    #[test]
    fn tray_click_before_focus_loss_keeps_the_panel_hidden() {
        let mut controller = Controller::default();
        let point = Point { x: 600, y: 700 };

        assert_eq!(controller.tray_click(), TrayAction::Show);
        controller.focus_gained();
        controller.tray_press(WorkArea {
            x: 590,
            y: 690,
            width: 40,
            height: 40,
        });
        assert_eq!(controller.tray_click(), TrayAction::Hide);
        assert!(!controller.focus_lost(Some(point)));
        assert_eq!(controller.state, LifecycleState::Hidden);
        assert_eq!(controller.tray_click(), TrayAction::Show);
    }

    #[test]
    fn unrelated_focus_loss_does_not_swallow_a_later_tray_click() {
        let mut controller = Controller::default();
        let focus_loss_point = Point { x: 100, y: 200 };

        assert_eq!(controller.tray_click(), TrayAction::Show);
        controller.focus_gained();
        assert!(controller.focus_lost(Some(focus_loss_point)));
        controller.tray_press(WorkArea {
            x: 880,
            y: 980,
            width: 40,
            height: 40,
        });
        assert_eq!(controller.tray_click(), TrayAction::Show);
    }

    #[test]
    fn right_click_focus_loss_does_not_swallow_the_next_left_click() {
        let mut controller = Controller::default();
        let point = Point { x: 900, y: 1000 };
        let tray_bounds = WorkArea {
            x: 880,
            y: 980,
            width: 40,
            height: 40,
        };

        assert_eq!(controller.tray_click(), TrayAction::Show);
        controller.focus_gained();
        assert!(controller.focus_lost(Some(point)));
        controller.cancel_tray_interaction();
        controller.tray_press(tray_bounds);

        assert_eq!(controller.tray_click(), TrayAction::Show);
    }

    #[test]
    fn focus_loss_after_show_hides_and_does_not_swallow_a_rapid_click() {
        let mut controller = Controller::default();
        let point = Point { x: 700, y: 800 };

        assert_eq!(controller.tray_click(), TrayAction::Show);
        assert!(controller.focus_lost(Some(point)));
        assert_eq!(controller.state, LifecycleState::Hidden);
        assert_eq!(controller.tray_click(), TrayAction::Show);
    }

    #[test]
    fn visible_click_hides_and_repeated_clicks_reuse_one_lifecycle() {
        let mut controller = Controller::default();

        assert_eq!(controller.tray_click(), TrayAction::Show);
        controller.focus_gained();
        assert_eq!(controller.tray_click(), TrayAction::Hide);
        assert_eq!(controller.tray_click(), TrayAction::Show);
        assert_eq!(controller.state, LifecycleState::Visible);
    }

    #[test]
    fn second_instance_signal_reaches_the_primary_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        TcpStream::connect(address)
            .unwrap()
            .write_all(&[1])
            .unwrap();
        assert!(listener.accept().is_ok());
    }
}
