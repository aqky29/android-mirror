#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod adb;
mod win;
mod brands {
    include!(concat!(env!("OUT_DIR"), "/brands_gen.rs"));
}

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use adb::{AndroidDevice, DeviceMetrics, MirrorOptions};
use rune::color::hex;
use rune::tok::type_;
use rune::{
    App, AppSpec, Color, Cx, El, ScrollSpec, TextEdit, Tray, TrayEvent, TrayIcon, TrayItem,
    TraySpec, Tx, Waker, col, el, icon, row, text,
};
use rune_kit::{
    Button, ButtonVariant, Divider, Kit, SegmentedControl, Switch, TextInput, root,
};

static ACTIVE_MIRROR_PID: Mutex<Option<u32>> = Mutex::new(None);

fn set_active_mirror_pid(pid: Option<u32>) {
    if let Ok(mut guard) = ACTIVE_MIRROR_PID.lock() {
        *guard = pid;
    }
}

fn kill_active_mirror() {
    let pid = ACTIVE_MIRROR_PID.lock().ok().and_then(|mut g| g.take());
    if let Some(pid) = pid {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(0x0800_0000)
                .status();
        }
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();
        }
    }
}

enum BgMsg {
    DevicesUpdated(Vec<AndroidDevice>),
    MetricsUpdated(Option<DeviceMetrics>),
    ScrcpyInstalled(Result<PathBuf, String>),
    WifiSwitched(Result<String, String>),
    WifiConnected(Result<String, String>),
    DeviceWoken(Result<(), String>),
}

struct State {
    adb_path: Option<PathBuf>,
    scrcpy_path: Option<PathBuf>,
    devices: Vec<AndroidDevice>,
    selected_serial: Option<String>,
    active_serial: Arc<Mutex<Option<String>>>,
    metrics: Option<DeviceMetrics>,
    installing_scrcpy: bool,
    busy_action: bool,
    status_msg: String,
    status_is_error: bool,
    quality: usize,
    turn_screen_off: bool,
    stay_awake: bool,
    always_on_top: bool,
    forward_audio: bool,
    rounded_window: bool,
    show_wifi_drawer: bool,
    show_lockscreen_tips: bool,
    wifi_addr: TextEdit,
    mirror_child: Option<Child>,
    rx: Option<Receiver<BgMsg>>,
    tx: Option<Sender<BgMsg>>,
    waker: Option<Waker>,
}

fn tips_flag_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return base.join("AndroidMirror").join("tips_seen.flag");
    }
    #[cfg(target_os = "macos")]
    {
        let base = std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .unwrap_or_else(|| PathBuf::from("."));
        return base.join("AndroidMirror").join("tips_seen.flag");
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
            })
            .unwrap_or_else(|| PathBuf::from("."));
        return base.join("android-mirror").join("tips_seen.flag");
    }
    #[allow(unreachable_code)]
    PathBuf::from("tips_seen.flag")
}

fn is_first_run() -> bool {
    !tips_flag_path().is_file()
}

fn mark_tips_seen() {
    let path = tips_flag_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, "1");
}

impl Default for State {
    fn default() -> Self {
        let adb_path = adb::find_adb();
        let scrcpy_path = adb::find_scrcpy();
        let status_msg = if scrcpy_path.is_none() {
            "scrcpy belum terpasang. Klik 'Install scrcpy' untuk setup otomatis.".to_string()
        } else if adb_path.is_none() {
            "ADB tidak ditemukan. Install scrcpy akan menyertakan ADB.".to_string()
        } else {
            "Siap. Hubungkan HP Android lewat USB atau Wi-Fi.".to_string()
        };

        Self {
            adb_path,
            scrcpy_path,
            devices: Vec::new(),
            selected_serial: None,
            active_serial: Arc::new(Mutex::new(None)),
            metrics: None,
            installing_scrcpy: false,
            busy_action: false,
            status_msg,
            status_is_error: false,
            quality: 1,
            turn_screen_off: true,
            stay_awake: true,
            always_on_top: false,
            forward_audio: true,
            rounded_window: false,
            show_wifi_drawer: false,
            show_lockscreen_tips: is_first_run(),
            wifi_addr: TextEdit::default(),
            mirror_child: None,
            rx: None,
            tx: None,
            waker: None,
        }
    }
}

impl State {
    fn refresh_devices(&self) {
        let Some(adb) = self.adb_path.clone() else { return };
        let Some(tx) = self.tx.clone() else { return };
        let Some(waker) = self.waker.clone() else { return };
        thread::spawn(move || {
            let devs = adb::list_devices(&adb);
            let _ = tx.send(BgMsg::DevicesUpdated(devs));
            waker();
        });
    }

    fn start_install_scrcpy(&mut self) {
        if self.installing_scrcpy {
            return;
        }
        self.installing_scrcpy = true;
        self.status_is_error = false;
        self.status_msg = "Mengunduh & mengekstrak scrcpy terbaru...".to_string();

        let Some(tx) = self.tx.clone() else { return };
        let Some(waker) = self.waker.clone() else { return };
        thread::spawn(move || {
            let res = adb::install_scrcpy();
            let _ = tx.send(BgMsg::ScrcpyInstalled(res));
            waker();
        });
    }

    fn switch_selected_to_wifi(&mut self) {
        if self.busy_action {
            return;
        }
        let Some(adb) = self.adb_path.clone() else {
            self.set_error("ADB belum tersedia.");
            return;
        };
        let Some(serial) = self.selected_serial.clone() else {
            self.set_error("Pilih perangkat USB terlebih dahulu.");
            return;
        };
        let is_wifi = self
            .devices
            .iter()
            .find(|d| d.serial == serial)
            .is_some_and(|d| d.is_wifi);
        if is_wifi {
            self.set_error("Perangkat ini sudah terhubung lewat Wi-Fi.");
            return;
        }

        self.busy_action = true;
        self.status_is_error = false;
        self.status_msg = "Mengaktifkan mode Wi-Fi (tcpip 5555)...".to_string();

        let Some(tx) = self.tx.clone() else { return };
        let Some(waker) = self.waker.clone() else { return };
        thread::spawn(move || {
            let res = adb::switch_to_wifi(&adb, &serial);
            let _ = tx.send(BgMsg::WifiSwitched(res));
            waker();
        });
    }

    fn connect_manual_wifi(&mut self) {
        if self.busy_action {
            return;
        }
        let Some(adb) = self.adb_path.clone() else {
            self.set_error("ADB belum tersedia.");
            return;
        };
        let addr = self.wifi_addr.text.clone();
        if addr.trim().is_empty() {
            self.set_error("Isi alamat IP:Port HP terlebih dahulu.");
            return;
        }

        self.busy_action = true;
        self.status_is_error = false;
        self.status_msg = format!("Menghubungkan ke {}...", addr.trim());

        let Some(tx) = self.tx.clone() else { return };
        let Some(waker) = self.waker.clone() else { return };
        thread::spawn(move || {
            let res = adb::connect_wifi(&adb, &addr);
            let _ = tx.send(BgMsg::WifiConnected(res));
            waker();
        });
    }

    fn start_or_stop_mirror(&mut self) {
        if let Some(mut child) = self.mirror_child.take() {
            set_active_mirror_pid(None);
            let _ = child.kill();
            let _ = child.wait();
            self.status_is_error = false;
            self.status_msg = "Mirroring dihentikan.".to_string();
            return;
        }

        let Some(scrcpy) = self.scrcpy_path.clone() else {
            self.set_error("scrcpy belum terpasang. Klik 'Install scrcpy' dulu.");
            return;
        };

        let Some(serial) = self.selected_serial.clone() else {
            self.set_error("Tidak ada HP yang dipilih.");
            return;
        };

        let dev_status = self
            .devices
            .iter()
            .find(|d| d.serial == serial)
            .map(|d| d.status.as_str())
            .unwrap_or("");

        if dev_status != "device" {
            self.set_error("HP belum siap. Izinkan USB Debugging di layar HP.");
            return;
        }

        let window_title = self
            .devices
            .iter()
            .find(|d| d.serial == serial)
            .map(|d| {
                if d.market_name.is_empty() {
                    d.model.clone()
                } else {
                    format!("{} ({})", d.market_name, d.model)
                }
            })
            .unwrap_or_else(|| "Android Mirror".to_string());

        let opts = MirrorOptions {
            serial: serial.clone(),
            window_title,
            quality: self.quality,
            turn_screen_off: self.turn_screen_off,
            stay_awake: self.stay_awake,
            always_on_top: self.always_on_top,
            forward_audio: self.forward_audio,
            rounded_window: self.rounded_window,
        };

        match adb::start_mirror(&scrcpy, self.adb_path.as_deref(), &opts) {
            Ok(child) => {
                set_active_mirror_pid(Some(child.id()));
                self.mirror_child = Some(child);
                self.status_is_error = false;
                let name = self
                    .devices
                    .iter()
                    .find(|d| d.serial == serial)
                    .map(|d| {
                        if d.market_name.is_empty() {
                            d.model.as_str()
                        } else {
                            d.market_name.as_str()
                        }
                    })
                    .unwrap_or(serial.as_str());
                self.status_msg = format!("Mirroring aktif • {name}");
            }
            Err(e) => self.set_error(e),
        }
    }

    fn wake_selected_device(&mut self) {
        let Some(adb) = self.adb_path.clone() else {
            self.set_error("ADB belum tersedia.");
            return;
        };
        let Some(serial) = self.selected_serial.clone() else {
            self.set_error("Tidak ada HP yang dipilih.");
            return;
        };
        self.status_is_error = false;
        self.status_msg = "Mengirim sinyal bangun (WAKEUP) ke HP...".to_string();
        let Some(tx) = self.tx.clone() else { return };
        let Some(waker) = self.waker.clone() else { return };
        thread::spawn(move || {
            let res = adb::wake_device(&adb, &serial);
            let _ = tx.send(BgMsg::DeviceWoken(res));
            waker();
        });
    }

    fn set_error(&mut self, msg: impl Into<String>) {
        self.status_is_error = true;
        self.status_msg = msg.into();
    }
}

fn brand_accent(brand: &str) -> Color {
    match brand.trim().to_ascii_lowercase().as_str() {
        "realme" => hex("#ffd644").unwrap(),
        "samsung" => hex("#60a5fa").unwrap(),
        "xiaomi" | "poco" | "redmi" => hex("#ff8533").unwrap(),
        "oppo" | "android" => hex("#34d399").unwrap(),
        "vivo" => hex("#38bdf8").unwrap(),
        "oneplus" | "asus" | "huawei" => hex("#f43f5e").unwrap(),
        "google" | "pixel" => hex("#a5b4fc").unwrap(),
        _ => hex("#9ca3af").unwrap(),
    }
}

fn setup(s: &mut State, waker: Waker) {
    win::center_own_window_in_work_area();
    let (tx, rx) = mpsc::channel();
    s.tx = Some(tx.clone());
    s.rx = Some(rx);
    s.waker = Some(waker.clone());

    let initial_adb = s.adb_path.clone();
    let active_serial = s.active_serial.clone();
    thread::spawn(move || {
        let mut last_devices: Option<Vec<AndroidDevice>> = None;
        let mut last_metrics: Option<Option<DeviceMetrics>> = None;
        loop {
            let adb_now = initial_adb.clone().or_else(adb::find_adb);
            if let Some(adb) = adb_now {
                let devs = adb::list_devices(&adb);
                if last_devices.as_ref() != Some(&devs) {
                    last_devices = Some(devs.clone());
                    let _ = tx.send(BgMsg::DevicesUpdated(devs.clone()));
                    waker();
                }

                let target_serial = {
                    let guard = active_serial.lock().ok().and_then(|g| g.clone());
                    guard
                        .filter(|sel| devs.iter().any(|d| &d.serial == sel && d.status == "device"))
                        .or_else(|| {
                            devs.iter()
                                .find(|d| d.status == "device")
                                .map(|d| d.serial.clone())
                        })
                };

                let next_metrics = target_serial.map(|ser| adb::fetch_device_metrics(&adb, &ser));
                if last_metrics.as_ref() != Some(&next_metrics) {
                    last_metrics = Some(next_metrics.clone());
                    let _ = tx.send(BgMsg::MetricsUpdated(next_metrics));
                    waker();
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
}

fn poll(s: &mut State, _now: f64, _tx: &mut Tx) -> bool {
    let mut changed = false;

    if let Some(child) = &mut s.mirror_child {
        if let Ok(Some(_status)) = child.try_wait() {
            s.mirror_child = None;
            set_active_mirror_pid(None);
            s.status_is_error = false;
            s.status_msg = "Jendela mirroring ditutup.".to_string();
            changed = true;
        }
    }

    let msgs: Vec<BgMsg> = s
        .rx
        .as_ref()
        .map(|rx| rx.try_iter().collect())
        .unwrap_or_default();

    for msg in msgs {
        changed = true;
        match msg {
            BgMsg::DevicesUpdated(devs) => {
                let still_exists = s
                    .selected_serial
                    .as_ref()
                    .is_some_and(|sel| devs.iter().any(|d| &d.serial == sel));
                if !still_exists {
                    s.selected_serial = devs
                        .iter()
                        .find(|d| d.status == "device")
                        .or_else(|| devs.first())
                        .map(|d| d.serial.clone());
                }
                if let Ok(mut g) = s.active_serial.lock() {
                    *g = s.selected_serial.clone();
                }
                if !devs.iter().any(|d| d.status == "device") {
                    s.metrics = None;
                }
                s.devices = devs;
            }
            BgMsg::MetricsUpdated(m) => {
                s.metrics = m;
            }
            BgMsg::ScrcpyInstalled(res) => {
                s.installing_scrcpy = false;
                match res {
                    Ok(path) => {
                        s.scrcpy_path = Some(path);
                        if s.adb_path.is_none() {
                            s.adb_path = adb::find_adb();
                        }
                        s.status_is_error = false;
                        s.status_msg = "scrcpy siap! Hubungkan HP Android kamu.".to_string();
                        s.refresh_devices();
                    }
                    Err(e) => s.set_error(e),
                }
            }
            BgMsg::WifiSwitched(res) => {
                s.busy_action = false;
                match res {
                    Ok(addr) => {
                        s.status_is_error = false;
                        s.status_msg = format!("Terhubung Wi-Fi ({addr}). Kabel boleh dicabut!");
                        s.selected_serial = Some(addr);
                        s.refresh_devices();
                    }
                    Err(e) => s.set_error(e),
                }
            }
            BgMsg::WifiConnected(res) => {
                s.busy_action = false;
                match res {
                    Ok(addr) => {
                        s.status_is_error = false;
                        s.status_msg = format!("Terhubung ke {addr} via Wi-Fi.");
                        s.selected_serial = Some(addr);
                        s.refresh_devices();
                    }
                    Err(e) => s.set_error(e),
                }
            }
            BgMsg::DeviceWoken(res) => match res {
                Ok(()) => {
                    s.status_is_error = false;
                    s.status_msg = "Sinyal bangun dikirim ke HP.".to_string();
                }
                Err(e) => s.set_error(e),
            },
        }
    }

    changed
}

fn dot_pill(label: &str, ok: bool, k: &Kit) -> El {
    let dot_color = if ok {
        hex("#34d399").unwrap()
    } else {
        k.c.text_faint
    };
    row()
        .items_center()
        .gap(6.0)
        .px(9.0).py(4.0)
        .radius(999.0)
        .fill(k.c.layer_01)
        .border(1.0, k.c.border_weak)
        .child(el().size(6.0, 6.0).radius(3.0).fill(dot_color))
        .child(text(label).font(k.ty.small).color(k.c.text_muted))
}

fn setting_row(
    title: &str,
    subtitle: Option<&str>,
    checked: bool,
    k: &Kit,
    on_toggle: impl Fn(&mut State) + 'static,
) -> El {
    let mut left = col()
        .gap(2.0)
        .child(text(title).font(k.ty.ui).color(k.c.text_base));
    if let Some(sub) = subtitle {
        left = left.child(text(sub).font(k.ty.small).color(k.c.text_faint));
    }

    row()
        .w_full()
        .justify_between()
        .items_center()
        .px(14.0).py(10.0)
        .child(left)
        .child(Switch::new(checked).on_toggle(on_toggle).build(k))
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let layer: Vec<El> = Vec::new();

    // Compact Header
    let mut status_pills = row()
        .gap(6.0)
        .items_center()
        .child(dot_pill("ADB", s.adb_path.is_some(), &k))
        .child(dot_pill("scrcpy", s.scrcpy_path.is_some(), &k));

    if s.scrcpy_path.is_none() {
        let lbl = if s.installing_scrcpy {
            "Installing..."
        } else {
            "Install scrcpy"
        };
        status_pills = status_pills.child(
            Button::new(lbl)
                .variant(ButtonVariant::Contrast)
                .on_press(|s: &mut State| s.start_install_scrcpy())
                .build(&k),
        );
    }

    let header = row()
        .w_full()
        .justify_between()
        .items_center()
        .child(
            col()
                .gap(2.0)
                .child(text("Android Mirror").font(type_::TITLE).color(k.c.text_base))
                .child(
                    text("Simple setup • USB & Wi-Fi")
                        .font(k.ty.small)
                        .color(k.c.text_faint),
                ),
        )
        .child(status_pills);

    // Section 1: Perangkat
    let mut device_group = col()
        .w_full()
        .radius(14.0)
        .fill(k.c.layer_01)
        .border(1.0, k.c.border_weak);

    if s.devices.is_empty() {
        device_group = device_group.child(
            col()
                .w_full()
                .gap(4.0)
                .padding(14.0)
                .child(
                    text("Belum ada HP terdeteksi")
                        .font(k.ty.ui_medium)
                        .color(k.c.text_base),
                )
                .child(
                    text("1. Aktifkan Opsi Pengembang & USB Debugging di HP.")
                        .font(k.ty.small)
                        .color(k.c.text_muted),
                )
                .child(
                    text("2. Colok kabel USB ke PC & ketuk 'Izinkan' di layar HP.")
                        .font(k.ty.small)
                        .color(k.c.text_muted),
                ),
        );
    } else {
        for (idx, dev) in s.devices.iter().enumerate() {
            if idx > 0 {
                device_group = device_group.child(Divider::horizontal().build(&k));
            }
            let selected = s.selected_serial.as_deref() == Some(dev.serial.as_str());
            let serial_for_pick = dev.serial.clone();
            let conn_kind = if dev.is_wifi { "Wi-Fi" } else { "USB" };
            let accent = brand_accent(&dev.brand);
            let brand_icon = brands::icon_for_brand(&dev.brand);

            let display_name = if dev.market_name.is_empty() {
                dev.model.clone()
            } else {
                format!("{} ({})", dev.market_name, dev.model)
            };

            let status_color = if dev.status == "device" {
                hex("#34d399").unwrap()
            } else {
                k.c.text_muted
            };
            let status_label = if dev.status == "device" {
                "Siap"
            } else {
                dev.status.as_str()
            };

            let short_serial = if dev.serial.len() > 34 {
                format!("{}...", &dev.serial[..34])
            } else {
                dev.serial.clone()
            };

            let dev_row = el()
                .key(format!("dev-{}", dev.serial))
                .w_full()
                .px(14.0).py(12.0)
                .fill(if selected {
                    k.c.layer_02
                } else {
                    Color::TRANSPARENT
                })
                .on_press(move |s: &mut State| {
                    s.selected_serial = Some(serial_for_pick.clone());
                })
                .child(
                    row()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .gap(10.0)
                        .child(
                            row()
                                .items_center()
                                .gap(12.0)
                                .grow(1.0)
                                .min_w(0.0)
                                .child(
                                    el().size(52.0, 38.0)
                                        .radius(10.0)
                                        .fill(accent.alpha(0.14))
                                        .border(1.0, accent.alpha(0.32))
                                        .center()
                                        .child(icon(brand_icon, 34.0).color(accent)),
                                )
                                .child(
                                    col()
                                        .gap(2.0)
                                        .min_w(0.0)
                                        .child(
                                            text(display_name)
                                                .font(k.ty.ui_medium)
                                                .color(k.c.text_base),
                                        )
                                        .child(
                                            text(format!("{conn_kind} • {short_serial}"))
                                                .font(k.ty.small)
                                                .color(k.c.text_faint),
                                        ),
                                ),
                        )
                        .child(
                            el().px(9.0).py(3.0)
                                .radius(999.0)
                                .fill(status_color.alpha(0.12))
                                .child(text(status_label).font(k.ty.small).color(status_color)),
                        ),
                );

            device_group = device_group.child(dev_row);
        }
    }

    // Toolbar inside device group
    device_group = device_group
        .child(Divider::horizontal().build(&k))
        .child(
            row()
                .w_full()
                .justify_between()
                .items_center()
                .px(12.0).py(8.0)
                .child(
                    row()
                        .gap(8.0)
                        .child(
                            Button::new("Pindah ke Wi-Fi")
                                .variant(ButtonVariant::Outline)
                                .on_press(|s: &mut State| s.switch_selected_to_wifi())
                                .build(&k),
                        )
                        .child(
                            Button::new("Bangunkan HP")
                                .variant(ButtonVariant::Outline)
                                .on_press(|s: &mut State| s.wake_selected_device())
                                .build(&k),
                        )
                        .child(
                            Button::new(if s.show_wifi_drawer {
                                "Tutup Wi-Fi Manual"
                            } else {
                                "Manual IP"
                            })
                            .variant(ButtonVariant::Ghost)
                            .on_press(|s: &mut State| s.show_wifi_drawer = !s.show_wifi_drawer)
                            .build(&k),
                        ),
                )
                .child(
                    Button::new("Refresh")
                        .variant(ButtonVariant::Ghost)
                        .on_press(|s: &mut State| s.refresh_devices())
                        .build(&k),
                ),
        );

    if s.devices.iter().any(|d| d.status == "device") {
        let m = s.metrics.clone().unwrap_or_default();
        let cpu_str = m
            .cpu_pct
            .map(|v| format!("{v}%"))
            .unwrap_or_else(|| "...".to_string());
        let gpu_str = m
            .gpu_pct
            .map(|v| format!("{v}%"))
            .unwrap_or_else(|| "N/A".to_string());
        let (temp_str, temp_color) = match m.battery_temp_c {
            Some(t) => {
                let bat = m
                    .battery_pct
                    .map(|p| format!(" • {p}%"))
                    .unwrap_or_default();
                let color = if t >= 42.0 {
                    k.c.danger.fg
                } else if t >= 38.0 {
                    k.c.warning.fg
                } else {
                    k.c.success.fg
                };
                (format!("{t:.1}°C{bat}"), color)
            }
            None => ("...".to_string(), k.c.text_muted),
        };

        let metric_cell = |label: &str, val: String, val_color: Color| {
            col()
                .grow(1.0)
                .gap(2.0)
                .px(10.0)
                .py(7.0)
                .clip_rounded(6.0)
                .fill(k.c.bg_base)
                .border(1.0, k.c.border_weak)
                .child(text(label).font(k.ty.small).color(k.c.text_faint))
                .child(text(val).font(k.ty.ui_medium).color(val_color))
        };

        device_group = device_group
            .child(Divider::horizontal().build(&k))
            .child(
                row()
                    .w_full()
                    .gap(8.0)
                    .px(12.0)
                    .py(10.0)
                    .child(metric_cell("CPU Usage", cpu_str, k.c.text_base))
                    .child(metric_cell("GPU Usage", gpu_str, k.c.text_base))
                    .child(metric_cell("Suhu Baterai", temp_str, temp_color)),
            );
    }

    if s.show_wifi_drawer {
        device_group = device_group
            .child(Divider::horizontal().build(&k))
            .child(
                col()
                    .w_full()
                    .gap(8.0)
                    .padding(12.0)
                    .fill(k.c.bg_base)
                    .child(
                        row()
                            .w_full()
                            .gap(8.0)
                            .items_center()
                            .child(
                                el().grow(1.0).child(
                                    TextInput::new(&s.wifi_addr)
                                        .placeholder("Contoh: 192.168.1.15:5555")
                                        .on_edit(|s: &mut State| &mut s.wifi_addr)
                                        .build(&k),
                                ),
                            )
                            .child(
                                Button::new("Connect")
                                    .variant(ButtonVariant::Neutral)
                                    .on_press(|s: &mut State| s.connect_manual_wifi())
                                    .build(&k),
                            ),
                    )
                    .child(
                        col()
                            .gap(3.0)
                            .child(
                                text("Via USB dulu: Colok USB lalu klik 'Pindah ke Wi-Fi'.")
                                    .font(k.ty.small)
                                    .color(k.c.text_muted),
                            )
                            .child(
                                text("Tanpa kabel (Android 11+): Opsi Pengembang > Wireless Debugging > isi IP & Port di atas.")
                                    .font(k.ty.small)
                                    .color(k.c.text_muted),
                            ),
                    ),
            );
    }

    let device_section = col()
        .w_full()
        .gap(6.0)
        .child(
            row()
                .w_full()
                .justify_between()
                .px(2.0).py(0.0)
                .child(text("PERANGKAT").font(k.ty.small).color(k.c.text_faint))
                .child(
                    text(format!("{} terdeteksi", s.devices.len()))
                        .font(k.ty.small)
                        .color(k.c.text_faint),
                ),
        )
        .child(device_group);

    // Section 2: Pengaturan
    let quality_meta = match s.quality {
        0 => "720p • 30 fps",
        1 => "1080p • 60 fps",
        _ => "Native • 60 fps",
    };

    let quality_ctrl = SegmentedControl::new(&["Ringan", "Seimbang", "Tajam"], Some(s.quality))
        .label("Kualitas")
        .fit()
        .on_pick(|s: &mut State, idx, _| s.quality = idx)
        .build(&k);

    let mut settings_group = col()
        .w_full()
        .radius(14.0)
        .fill(k.c.layer_01)
        .border(1.0, k.c.border_weak)
        .child(el().w_full().px(12.0).py(10.0).child(quality_ctrl))
        .child(Divider::horizontal().build(&k))
        .child(setting_row(
            "Matikan layar HP",
            Some("Layar fisik gelap, tetap bisa dikontrol dari PC"),
            s.turn_screen_off,
            &k,
            |s| s.turn_screen_off = !s.turn_screen_off,
        ))
        .child(Divider::horizontal().build(&k))
        .child(setting_row(
            "Jaga HP tetap menyala",
            None,
            s.stay_awake,
            &k,
            |s| s.stay_awake = !s.stay_awake,
        ))
        .child(Divider::horizontal().build(&k))
        .child(setting_row(
            "Selalu di atas (Always on top)",
            None,
            s.always_on_top,
            &k,
            |s| s.always_on_top = !s.always_on_top,
        ))
        .child(Divider::horizontal().build(&k))
        .child(setting_row(
            "Teruskan audio HP ke PC",
            None,
            s.forward_audio,
            &k,
            |s| s.forward_audio = !s.forward_audio,
        ))
        .child(Divider::horizontal().build(&k))
        .child(setting_row(
            if cfg!(windows) {
                "Jendela mirror rounded (borderless)"
            } else {
                "Jendela mirror borderless"
            },
            Some(if cfg!(windows) {
                "Sudut melengkung dengan bar atas tipis (drag, minimize, close)"
            } else {
                "Tampilan bersih tanpa bar judul jendela (--window-borderless)"
            }),
            s.rounded_window,
            &k,
            |s| s.rounded_window = !s.rounded_window,
        ));

    if s.show_lockscreen_tips {
        settings_group = settings_group
            .child(Divider::horizontal().build(&k))
            .child(
                col()
                    .w_full()
                    .gap(6.0)
                    .padding(12.0)
                    .fill(k.c.bg_base)
                    .child(
                        row()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(
                                text("Tips Lockscreen & Layar Mati")
                                    .font(k.ty.ui_medium)
                                    .color(k.c.text_base),
                            )
                            .child(
                                Button::new("Mengerti, Tutup")
                                    .variant(ButtonVariant::Outline)
                                    .on_press(|s: &mut State| {
                                        s.show_lockscreen_tips = false;
                                        mark_tips_seen();
                                    })
                                    .build(&k),
                            ),
                    )
                    .child(
                        text("• Layar hitam saat Lockscreen: proteksi sistem Android (FLAG_SECURE). Tekan Spasi > ketik PIN HP di keyboard PC > Enter untuk buka kunci.")
                            .font(k.ty.small)
                            .color(k.c.text_muted),
                    )
                    .child(
                        text("• Jangan tekan Power fisik HP: pakai 'Matikan layar HP' (atau Alt+O di jendela mirror) agar layar gelap tanpa mengunci HP.")
                            .font(k.ty.small)
                            .color(k.c.text_muted),
                    )
                    .child(
                        text("• realme/OPPO/Xiaomi: aktifkan 'Disable permission monitoring' atau 'USB Debugging (Security Settings)' di Opsi Pengembang.")
                            .font(k.ty.small)
                            .color(k.c.text_muted),
                    ),
            );
    }

    let settings_section = col()
        .w_full()
        .gap(6.0)
        .child(
            row()
                .w_full()
                .justify_between()
                .items_center()
                .px(2.0).py(0.0)
                .child(
                    row()
                        .gap(8.0)
                        .items_center()
                        .child(text("PENGATURAN").font(k.ty.small).color(k.c.text_faint))
                        .child(
                            Button::new(if s.show_lockscreen_tips {
                                "Tutup Tips"
                            } else {
                                "Tips Lockscreen"
                            })
                            .variant(ButtonVariant::Ghost)
                            .on_press(|s: &mut State| {
                                s.show_lockscreen_tips = !s.show_lockscreen_tips;
                                if !s.show_lockscreen_tips {
                                    mark_tips_seen();
                                }
                            })
                            .build(&k),
                        ),
                )
                .child(text(quality_meta).font(k.ty.small).color(k.c.text_faint)),
        )
        .child(settings_group);

    // Footer
    let is_running = s.mirror_child.is_some();
    let footer = row()
        .w_full()
        .justify_between()
        .items_center()
        .gap(12.0)
        .px(18.0).py(12.0)
        .fill(k.c.layer_01)
        .border(1.0, k.c.border_weak)
        .child(
            el().grow(1.0).min_w(0.0).child(
                text(&s.status_msg)
                    .font(k.ty.small)
                    .color(if s.status_is_error {
                        k.c.danger.fg
                    } else {
                        k.c.text_muted
                    }),
            ),
        )
        .child(
            Button::new(if is_running {
                "Stop Mirroring"
            } else {
                "Mulai Mirroring"
            })
            .variant(if is_running {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Contrast
            })
            .on_press(|s: &mut State| s.start_or_stop_mirror())
            .build(&k),
        );

    let body = col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .fill(k.c.bg_base)
        .child(
            col()
                .w_full()
                .items_center()
                .child(el().w_full().max_w(540.0).px(18.0).py(14.0).child(header)),
        )
        .child(Divider::horizontal().build(&k))
        .child(
            col().key("viewport")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec { stick_bottom: false })
                .child(
                    col()
                        .w_full()
                        .items_center()
                        .padding(18.0)
                        .child(
                            col()
                                .w_full()
                                .max_w(540.0)
                                .gap(16.0)
                                .child(device_section)
                                .child(settings_section),
                        ),
                ),
        )
        .child(footer);

    root(body, layer)
}

impl Drop for State {
    fn drop(&mut self) {
        kill_active_mirror();
        if let Some(mut child) = self.mirror_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn make_tray_icon() -> TrayIcon {
    let size = 16usize;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let in_phone = (3..=12).contains(&x) && (1..=14).contains(&y);
            let in_screen = (4..=11).contains(&x) && (3..=11).contains(&y);
            let is_dot = (7..=8).contains(&x) && y == 13;
            if in_screen {
                // Indigo accent screen
                rgba[idx] = 99;
                rgba[idx + 1] = 102;
                rgba[idx + 2] = 241;
                rgba[idx + 3] = 255;
            } else if is_dot {
                rgba[idx] = 244;
                rgba[idx + 1] = 244;
                rgba[idx + 2] = 246;
                rgba[idx + 3] = 255;
            } else if in_phone {
                // Phone frame
                rgba[idx] = 228;
                rgba[idx + 1] = 228;
                rgba[idx + 2] = 231;
                rgba[idx + 3] = 255;
            }
        }
    }
    TrayIcon::rgba(size as u32, size as u32, rgba)
}

fn main() {
    let spec = AppSpec::<State> {
        min_size: Some((440.0, 480.0)),
        setup: Some(setup),
        poll: Some(poll),
        scenes: &[("connected-realme", |s| {
            s.adb_path = Some(PathBuf::from("adb"));
            s.scrcpy_path = Some(PathBuf::from("scrcpy"));
            s.devices = vec![AndroidDevice {
                serial: "adb-EXAMPLE123._adb-tls-connect._tcp".to_string(),
                status: "device".to_string(),
                model: "RMX3630".to_string(),
                brand: "realme".to_string(),
                market_name: "realme 10 Pro".to_string(),
                is_wifi: true,
            }];
            s.selected_serial = Some(s.devices[0].serial.clone());
            s.metrics = Some(DeviceMetrics {
                cpu_pct: Some(24),
                gpu_pct: Some(18),
                battery_temp_c: Some(34.2),
                battery_pct: Some(78),
            });
            s.status_msg = "Siap mirroring • realme 10 Pro".to_string();
        })],
        ..AppSpec::new("Android Mirror", (480.0, 590.0), State::default, view)
    };

    let app = App::new(spec).keep_running();
    let handle = app.handle();

    let _tray = if !rune::is_headless() {
        let tray_spec = TraySpec::new("android-mirror", "Android Mirror", make_tray_icon())
            .tooltip("Android Mirror — Klik untuk membuka")
            .menu(vec![
                TrayItem::action("show", "Buka Android Mirror"),
                TrayItem::Separator,
                TrayItem::action("quit", "Keluar"),
            ]);
        Tray::new(tray_spec, move |ev| match ev {
            TrayEvent::Activate { .. } => handle.open(),
            TrayEvent::Menu(id) if id == "show" => handle.open(),
            TrayEvent::Menu(id) if id == "quit" => {
                kill_active_mirror();
                handle.quit();
            }
            _ => {}
        })
        .ok()
    } else {
        None
    };

    app.run();
}
