use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const PORTABLE_MACRO_FILE: &str = "macro.e.mc";

const EMBEDDED_MACRO_MAGIC: &[u8] = b"MACRO_CENTER_EMBEDDED_MACRO_V1";

pub struct StartupMacroState {
    pub macro_json: Option<String>,
}

pub struct StartupMacro {
    pub macro_json: String,
    pub source: StartupMacroSource,
}

pub enum StartupMacroSource {
    Embedded,
    AdjacentFile(PathBuf),
}

#[tauri::command]
pub fn get_startup_macro(state: tauri::State<StartupMacroState>) -> Option<String> {
    state.macro_json.clone()
}

#[tauri::command]
pub fn export_standalone_macro(
    project_json: String,
    destination_path: String,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = (project_json, destination_path);
        return Err("Standalone macro export is only supported on Windows and Linux".to_string());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let destination = PathBuf::from(destination_path);
        let mut exe = current_base_exe_bytes().map_err(|e| e.to_string())?;
        append_embedded_macro(&mut exe, project_json.as_bytes());
        fs::write(&destination, exe).map_err(|e| e.to_string())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&destination)
                .map_err(|e| e.to_string())?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&destination, permissions).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

#[tauri::command]
pub fn export_portable_macro_bundle(
    project_json: String,
    destination_path: String,
) -> Result<(), String> {
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let app_name = exe_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("macro-center")
        .to_string();
    let app_bytes = current_base_exe_bytes().map_err(|e| e.to_string())?;
    let files = [
        ZipFileEntry {
            name: app_name.as_str(),
            data: app_bytes.as_slice(),
        },
        ZipFileEntry {
            name: PORTABLE_MACRO_FILE,
            data: project_json.as_bytes(),
        },
    ];

    write_stored_zip(Path::new(&destination_path), &files).map_err(|e| e.to_string())
}

pub fn load_startup_macro() -> Option<StartupMacro> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Ok(bytes) = fs::read(&exe_path)
            && let Some(macro_bytes) = embedded_macro_bytes(&bytes)
            && let Ok(macro_json) = String::from_utf8(macro_bytes.to_vec())
        {
            return Some(StartupMacro {
                macro_json,
                source: StartupMacroSource::Embedded,
            });
        }

        if let Some(directory) = exe_path.parent()
            && let Some((path, macro_json)) = adjacent_embedded_macro(directory)
        {
            return Some(StartupMacro {
                macro_json,
                source: StartupMacroSource::AdjacentFile(path),
            });
        }
    }

    None
}

fn adjacent_embedded_macro(directory: &Path) -> Option<(PathBuf, String)> {
    let entries = fs::read_dir(directory).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".e.mc") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            return Some((path, content));
        }
    }

    None
}

fn current_base_exe_bytes() -> io::Result<Vec<u8>> {
    let exe_path = std::env::current_exe()?;
    let bytes = fs::read(exe_path)?;
    Ok(strip_embedded_macro(bytes))
}

fn append_embedded_macro(bytes: &mut Vec<u8>, macro_json: &[u8]) {
    bytes.extend_from_slice(macro_json);
    bytes.extend_from_slice(EMBEDDED_MACRO_MAGIC);
    bytes.extend_from_slice(&(macro_json.len() as u64).to_le_bytes());
}

fn strip_embedded_macro(mut bytes: Vec<u8>) -> Vec<u8> {
    if let Some(start) = embedded_macro_start(&bytes) {
        bytes.truncate(start);
    }
    bytes
}

fn embedded_macro_bytes(bytes: &[u8]) -> Option<&[u8]> {
    let start = embedded_macro_start(bytes)?;
    let end = bytes.len() - EMBEDDED_MACRO_MAGIC.len() - std::mem::size_of::<u64>();
    Some(&bytes[start..end])
}

fn embedded_macro_start(bytes: &[u8]) -> Option<usize> {
    const LEN_SIZE: usize = std::mem::size_of::<u64>();
    if bytes.len() < EMBEDDED_MACRO_MAGIC.len() + LEN_SIZE {
        return None;
    }

    let len_offset = bytes.len() - LEN_SIZE;
    let mut len_bytes = [0u8; LEN_SIZE];
    len_bytes.copy_from_slice(&bytes[len_offset..]);
    let macro_len = u64::from_le_bytes(len_bytes) as usize;
    let magic_start = len_offset.checked_sub(EMBEDDED_MACRO_MAGIC.len())?;
    if &bytes[magic_start..len_offset] != EMBEDDED_MACRO_MAGIC {
        return None;
    }

    magic_start.checked_sub(macro_len)
}

struct ZipFileEntry<'a> {
    name: &'a str,
    data: &'a [u8],
}

struct CentralDirectoryEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
}

fn write_stored_zip(destination: &Path, files: &[ZipFileEntry<'_>]) -> io::Result<()> {
    let mut output = Vec::new();
    let mut central_entries = Vec::with_capacity(files.len());

    for file in files {
        let name_bytes = file.name.as_bytes();
        let size = u32::try_from(file.data.len()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "ZIP entries must be under 4 GiB",
            )
        })?;
        let name_len = u16::try_from(name_bytes.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "ZIP file name is too long")
        })?;
        let local_header_offset = u32::try_from(output.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP file is too large"))?;
        let crc32 = crc32(file.data);

        write_u32(&mut output, 0x0403_4b50)?;
        write_u16(&mut output, 20)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u32(&mut output, crc32)?;
        write_u32(&mut output, size)?;
        write_u32(&mut output, size)?;
        write_u16(&mut output, name_len)?;
        write_u16(&mut output, 0)?;
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(file.data);

        central_entries.push(CentralDirectoryEntry {
            name: file.name.to_string(),
            crc32,
            size,
            local_header_offset,
        });
    }

    let central_directory_offset = u32::try_from(output.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP file is too large"))?;

    for entry in &central_entries {
        let name_bytes = entry.name.as_bytes();
        let name_len = u16::try_from(name_bytes.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "ZIP file name is too long")
        })?;

        write_u32(&mut output, 0x0201_4b50)?;
        write_u16(&mut output, 20)?;
        write_u16(&mut output, 20)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u32(&mut output, entry.crc32)?;
        write_u32(&mut output, entry.size)?;
        write_u32(&mut output, entry.size)?;
        write_u16(&mut output, name_len)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u16(&mut output, 0)?;
        write_u32(&mut output, 0)?;
        write_u32(&mut output, entry.local_header_offset)?;
        output.extend_from_slice(name_bytes);
    }

    let central_directory_size = u32::try_from(output.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP file is too large"))?
        - central_directory_offset;
    let file_count = u16::try_from(central_entries.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "ZIP has too many entries"))?;

    write_u32(&mut output, 0x0605_4b50)?;
    write_u16(&mut output, 0)?;
    write_u16(&mut output, 0)?;
    write_u16(&mut output, file_count)?;
    write_u16(&mut output, file_count)?;
    write_u32(&mut output, central_directory_size)?;
    write_u32(&mut output, central_directory_offset)?;
    write_u16(&mut output, 0)?;

    fs::write(destination, output)
}

fn write_u16(output: &mut Vec<u8>, value: u16) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn write_u32(output: &mut Vec<u8>, value: u32) -> io::Result<()> {
    output.write_all(&value.to_le_bytes())
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
