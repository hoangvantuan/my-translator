use crate::audio::microphone::MicCapture;
use crate::audio::SystemAudioCapture;
use crate::commands::local_pipeline::LocalPipelineState;
use serde::Serialize;
use std::io::Write;
use std::process::Child;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{ipc::Channel, State};

/// State for tracking active audio captures
pub struct AudioState {
    pub system_audio: Mutex<SystemAudioCapture>,
    pub microphone: Mutex<MicCapture>,
    pub active_receiver: Mutex<Option<AudioForwarder>>,
}

/// Forwards audio from a receiver to a Tauri IPC channel
pub struct AudioForwarder {
    /// Handle to signal stop
    stop_flag: Arc<AtomicBool>,
}

impl AudioForwarder {
    fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

#[derive(Serialize, Clone)]
pub struct PermissionStatus {
    pub screen_recording: String,
    pub microphone: String,
}

fn start_receiver_for_source(
    source: &str,
    state: &AudioState,
) -> Result<mpsc::Receiver<Vec<u8>>, String> {
    match source {
        "system" => {
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            sys.start()
        }
        "microphone" => {
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            mic.start()
        }
        "both" => {
            let sys = state.system_audio.lock().map_err(|e| e.to_string())?;
            let sys_rx = sys.start()?;
            let mut mic = state.microphone.lock().map_err(|e| e.to_string())?;
            let mic_rx = mic.start()?;

            let (merged_tx, merged_rx) = mpsc::channel::<Vec<u8>>();
            let tx1 = merged_tx.clone();
            let tx2 = merged_tx;

            std::thread::spawn(move || {
                while let Ok(data) = sys_rx.recv() {
                    if tx1.send(data).is_err() {
                        break;
                    }
                }
            });

            std::thread::spawn(move || {
                while let Ok(data) = mic_rx.recv() {
                    if tx2.send(data).is_err() {
                        break;
                    }
                }
            });

            Ok(merged_rx)
        }
        _ => Err(format!("Unknown source: {}", source)),
    }
}

/// Start audio capture and forward data to the frontend via IPC channel
#[tauri::command]
pub fn start_capture(
    source: String,
    channel: Channel<Vec<u8>>,
    state: State<'_, AudioState>,
) -> Result<(), String> {
    // Stop any existing capture first
    stop_capture_inner(&state);

    let receiver = start_receiver_for_source(&source, &state)?;

    // Spawn a thread to forward audio data from receiver to IPC channel
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        let mut buffer: Vec<u8> = Vec::with_capacity(16000);
        let batch_interval = std::time::Duration::from_millis(100);
        let mut last_flush = std::time::Instant::now();

        loop {
            if stop_flag_clone.load(Ordering::SeqCst) {
                // Flush remaining buffer before exit
                if !buffer.is_empty() {
                    let _ = channel.send(buffer.clone());
                }
                break;
            }

            match receiver.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok(data) => {
                    buffer.extend_from_slice(&data);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if !buffer.is_empty() {
                        let _ = channel.send(buffer.clone());
                    }
                    break;
                }
            }

            // Flush buffer every 200ms
            if last_flush.elapsed() >= batch_interval && !buffer.is_empty() {
                if let Err(_e) = channel.send(buffer.clone()) {
                    break; // Channel closed
                }
                buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }
    });

    // Store the forwarder so we can stop it later
    let forwarder = AudioForwarder { stop_flag };
    let mut active = state.active_receiver.lock().map_err(|e| e.to_string())?;
    *active = Some(forwarder);

    Ok(())
}

/// Start audio capture and write batches directly to the local pipeline stdin.
#[tauri::command]
pub fn start_capture_to_pipeline(
    source: String,
    state: State<'_, AudioState>,
    pipeline_state: State<'_, LocalPipelineState>,
) -> Result<(), String> {
    stop_capture_inner(&state);

    let receiver = start_receiver_for_source(&source, &state)?;
    let process = pipeline_state.process.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    std::thread::spawn(move || {
        let mut buffer: Vec<u8> = Vec::with_capacity(16000);
        let batch_interval = std::time::Duration::from_millis(100);
        let mut last_flush = std::time::Instant::now();

        loop {
            if stop_flag_clone.load(Ordering::SeqCst) {
                if !buffer.is_empty() {
                    let _ = write_audio_to_pipeline(&process, &buffer);
                }
                break;
            }

            match receiver.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok(data) => {
                    buffer.extend_from_slice(&data);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    if !buffer.is_empty() {
                        let _ = write_audio_to_pipeline(&process, &buffer);
                    }
                    break;
                }
            }

            if last_flush.elapsed() >= batch_interval && !buffer.is_empty() {
                if write_audio_to_pipeline(&process, &buffer).is_err() {
                    break;
                }
                buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }
    });

    let forwarder = AudioForwarder { stop_flag };
    let mut active = state.active_receiver.lock().map_err(|e| e.to_string())?;
    *active = Some(forwarder);

    Ok(())
}

fn write_audio_to_pipeline(
    process: &Arc<Mutex<Option<Child>>>,
    data: &[u8],
) -> Result<(), String> {
    let mut proc = process.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *proc {
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(data).map_err(|e| e.to_string())?;
            stdin.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Stop audio capture
#[tauri::command]
pub fn stop_capture(state: State<'_, AudioState>) -> Result<(), String> {
    stop_capture_inner(&state);
    Ok(())
}

fn stop_capture_inner(state: &AudioState) {
    // Stop the forwarder
    if let Ok(mut active) = state.active_receiver.lock() {
        if let Some(forwarder) = active.take() {
            forwarder.stop();
        }
    }

    // Stop system audio
    if let Ok(sys) = state.system_audio.lock() {
        sys.stop();
    }

    // Stop microphone
    if let Ok(mut mic) = state.microphone.lock() {
        mic.stop();
    }
}

/// Check audio capture permissions
#[tauri::command]
pub fn check_permissions() -> PermissionStatus {
    // Note: Actual permission checking on macOS requires Objective-C interop
    // For now, we return "unknown" and permissions will be prompted on first use
    PermissionStatus {
        screen_recording: "unknown".to_string(),
        microphone: "unknown".to_string(),
    }
}

/// Open macOS Privacy & Security > Screen Recording settings
#[tauri::command]
pub fn open_privacy_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
            .map_err(|e| format!("Failed to open System Settings: {}", e))?;
    }
    Ok(())
}
