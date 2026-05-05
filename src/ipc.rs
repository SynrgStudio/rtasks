use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::io::FromRawHandle;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, WaitNamedPipeW,
};
use windows::core::w;

const PIPE_NAME: windows::core::PCWSTR = w!(r"\\.\pipe\rtasks");
const PIPE_PATH: &str = r"\\.\pipe\rtasks";
const PIPE_BUFFER_SIZE: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    QuickAdd,
    Panel,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcRequest {
    pub cmd: IpcCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcResponse {
    Ok { message: String },
    Error { message: String },
}

pub fn start_named_pipe_server(sender: Sender<IpcCommand>) -> JoinHandle<()> {
    thread::spawn(move || run_named_pipe_server(sender))
}

pub fn send_command(command: IpcCommand, timeout: Duration) -> Result<IpcResponse> {
    let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    let pipe_available = unsafe { WaitNamedPipeW(PIPE_NAME, timeout_ms) };
    if !pipe_available.as_bool() {
        let last_error = unsafe { GetLastError() };
        anyhow::bail!(
            "rtasks daemon is not reachable on \\.\\pipe\\rtasks within {}ms: {last_error:?}",
            timeout.as_millis()
        );
    }

    let pipe = OpenOptions::new().read(true).write(true).open(PIPE_PATH)?;
    let mut reader = BufReader::new(pipe);
    let request = IpcRequest { cmd: command }.to_json_line()?;
    reader.get_mut().write_all(request.as_bytes())?;
    reader.get_mut().flush()?;

    let mut response_line = String::new();
    reader.read_line(&mut response_line)?;
    if response_line.trim().is_empty() {
        anyhow::bail!("empty response from rtasks daemon");
    }

    IpcResponse::from_json_line(&response_line)
}

fn run_named_pipe_server(sender: Sender<IpcCommand>) {
    loop {
        match create_pipe_file() {
            Ok(pipe) => {
                if !handle_client(pipe, &sender) {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn create_pipe_file() -> Result<File> {
    let handle = unsafe {
        CreateNamedPipeW(
            PIPE_NAME,
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            PIPE_BUFFER_SIZE,
            PIPE_BUFFER_SIZE,
            0,
            None,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        anyhow::bail!("CreateNamedPipeW failed");
    }

    let connected = unsafe { ConnectNamedPipe(handle, None) };
    if connected.is_err() {
        let last_error = unsafe { GetLastError() };
        if last_error != ERROR_PIPE_CONNECTED {
            let _ = unsafe { CloseHandle(handle) };
            anyhow::bail!("ConnectNamedPipe failed: {last_error:?}");
        }
    }

    Ok(unsafe { File::from_raw_handle(handle.0) })
}

fn handle_client(pipe: File, sender: &Sender<IpcCommand>) -> bool {
    let mut reader = BufReader::new(pipe);
    let mut line = String::new();
    let response = match reader.read_line(&mut line) {
        Ok(0) => IpcResponse::error("empty IPC request"),
        Ok(_) => match IpcRequest::from_json_line(&line) {
            Ok(request) => match sender.send(request.cmd) {
                Ok(()) => IpcResponse::ok(accepted_message(request.cmd)),
                Err(error) => IpcResponse::error(format!("daemon dispatcher unavailable: {error}")),
            },
            Err(error) => IpcResponse::error(error.to_string()),
        },
        Err(error) => IpcResponse::error(format!("failed to read IPC request: {error}")),
    };

    let should_continue =
        !matches!(response, IpcResponse::Ok { ref message } if message.contains("shutdown"));

    if let Ok(line) = response.to_json_line() {
        let _ = reader.get_mut().write_all(line.as_bytes());
        let _ = reader.get_mut().flush();
    }

    should_continue
}

fn accepted_message(command: IpcCommand) -> &'static str {
    match command {
        IpcCommand::QuickAdd => "quick add accepted",
        IpcCommand::Panel => "panel accepted",
        IpcCommand::Shutdown => "shutdown accepted",
    }
}

impl IpcRequest {
    pub fn to_json_line(&self) -> Result<String> {
        let mut line = serde_json::to_string(self).context("failed to serialize IPC request")?;
        line.push('\n');
        Ok(line)
    }

    pub fn from_json_line(line: &str) -> Result<Self> {
        serde_json::from_str(line.trim()).context("failed to parse IPC request")
    }
}

impl IpcResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self::Ok {
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    pub fn to_json_line(&self) -> Result<String> {
        let mut line = serde_json::to_string(self).context("failed to serialize IPC response")?;
        line.push('\n');
        Ok(line)
    }

    pub fn from_json_line(line: &str) -> Result<Self> {
        serde_json::from_str(line.trim()).context("failed to parse IPC response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_json_matches_protocol() {
        assert_eq!(
            (IpcRequest {
                cmd: IpcCommand::QuickAdd
            })
            .to_json_line()
            .expect("serialize"),
            "{\"cmd\":\"quick_add\"}\n"
        );
        assert_eq!(
            (IpcRequest {
                cmd: IpcCommand::Panel
            })
            .to_json_line()
            .expect("serialize"),
            "{\"cmd\":\"panel\"}\n"
        );
        assert_eq!(
            (IpcRequest {
                cmd: IpcCommand::Shutdown
            })
            .to_json_line()
            .expect("serialize"),
            "{\"cmd\":\"shutdown\"}\n"
        );
    }
}
