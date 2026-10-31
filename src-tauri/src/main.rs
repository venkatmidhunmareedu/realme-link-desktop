#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod mock;
mod session;
mod transport;

use buds_protocol::{model::Catalog, parse::KeyBinding};
use session::{DeviceState, Session};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

#[derive(Default)]
struct App {
    session: Mutex<Option<Arc<Session>>>,
}

impl App {
    /// Replace the current session, closing the old one first.
    fn swap(&self, new: Option<Arc<Session>>) {
        if let Some(old) = std::mem::replace(&mut *self.session.lock().unwrap(), new) {
            old.close();
        }
    }

    fn cur(&self) -> Result<Arc<Session>, String> {
        self.session.lock().unwrap().clone().ok_or_else(|| "not connected".into())
    }
}

fn init_state(catalog: &Catalog, name: &str, address: &str, mock: bool) -> (DeviceState, Option<buds_protocol::model::ModelConfig>) {
    let found = catalog.find(name);
    let st = DeviceState {
        mock,
        name: name.into(),
        address: address.into(),
        model: found.map(|(k, _)| k.to_string()),
        view_items: found.map(|(_, c)| c.view_items.clone()).unwrap_or_default(),
        product: found.map(|(_, c)| c.product.clone()).unwrap_or_default(),
        ..Default::default()
    };
    (st, found.map(|(_, c)| c.clone()))
}

#[tauri::command]
async fn scan() -> Result<Vec<transport::Paired>, String> {
    transport::paired().await
}

/// Every model the app knows how to drive (also used to pick a mock).
#[tauri::command]
fn models() -> Vec<String> {
    Catalog::bundled().0.into_keys().collect()
}

#[tauri::command]
async fn connect(app: AppHandle, st: State<'_, App>, name: String, address: String) -> Result<DeviceState, String> {
    let catalog = Catalog::bundled();
    let (init, cfg) = init_state(&catalog, &name, &address, false);
    let cfg = cfg.ok_or_else(|| format!("\"{name}\" is not a supported model"))?;
    let link = transport::connect_bluez(&address, cfg.spp_uuid()).await?;
    let s = Session::start(app, link, init, Some(&cfg));
    let snapshot = s.state.lock().unwrap().clone();
    st.swap(Some(s));
    Ok(snapshot)
}

#[tauri::command]
async fn connect_mock(app: AppHandle, st: State<'_, App>, model: String) -> Result<DeviceState, String> {
    let catalog = Catalog::bundled();
    let (init, cfg) = init_state(&catalog, &model, "00:00:00:00:00:00", true);
    let s = Session::start(app, mock::spawn(), init, cfg.as_ref());
    let snapshot = s.state.lock().unwrap().clone();
    st.swap(Some(s));
    Ok(snapshot)
}

#[tauri::command]
fn disconnect(st: State<'_, App>) {
    st.swap(None);
}

#[tauri::command]
async fn set_noise(st: State<'_, App>, value: u8) -> Result<(), String> {
    Ok(st.cur()?.set_noise(value).await)
}

#[tauri::command]
async fn set_feature(st: State<'_, App>, id: u8, on: bool) -> Result<(), String> {
    Ok(st.cur()?.set_feature(id, on).await)
}

#[tauri::command]
async fn set_eq(st: State<'_, App>, id: u8) -> Result<(), String> {
    Ok(st.cur()?.set_eq(id).await)
}

#[tauri::command]
async fn set_high_volume(st: State<'_, App>, on: bool) -> Result<(), String> {
    Ok(st.cur()?.set_high_volume(on).await)
}

#[tauri::command]
async fn set_keymap(st: State<'_, App>, keys: Vec<KeyBinding>) -> Result<(), String> {
    Ok(st.cur()?.set_keymap(keys).await)
}

#[tauri::command]
async fn find(st: State<'_, App>, on: bool) -> Result<(), String> {
    Ok(st.cur()?.find(on).await)
}

fn main() {
    tauri::Builder::default()
        .manage(App::default())
        .invoke_handler(tauri::generate_handler![
            scan, models, connect, connect_mock, disconnect, set_noise, set_feature, set_eq, set_high_volume, set_keymap, find
        ])
        .run(tauri::generate_context!())
        .expect("error while running Realme Link");
}
