use std::ptr::null_mut;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPPROCESS, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::mem::zeroed;
use std::io;

pub fn get_process_id_by_name(process_name: &str) -> Option<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == null_mut() {
        return None;
    }

    let mut process_entry: winapi::um::tlhelp32::PROCESSENTRY32W = unsafe { zeroed() };
    process_entry.dwSize = std::mem::size_of::<winapi::um::tlhelp32::PROCESSENTRY32W>() as u32;

    if unsafe { Process32FirstW(snapshot, &mut process_entry) } == 0 {
        unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
        return None;
    }

    let target_name = OsStr::new(process_name).encode_wide().chain(Some(0)).collect::<Vec<_>>();

    loop {
        if target_name.as_slice() == &process_entry.szExeFile[..target_name.len()] {
            unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
            return Some(process_entry.th32ProcessID);
        }

        if unsafe { Process32NextW(snapshot, &mut process_entry) } == 0 {
            break;
        }
    }

    unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
    None
}

pub fn get_module_base_address(process_id: u32, module_name: &str) -> Option<usize> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process_id) };
    if snapshot == null_mut() {
        return None;
    }

    let mut module_entry: MODULEENTRY32W = unsafe { zeroed() };
    module_entry.dwSize = std::mem::size_of::<MODULEENTRY32W>() as u32;

    if unsafe { Module32FirstW(snapshot, &mut module_entry) } == 0 {
        unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
        return None;
    }

    let target_name = OsStr::new(module_name).encode_wide().chain(Some(0)).collect::<Vec<_>>();

    loop {
        if target_name.as_slice() == &module_entry.szModule[..target_name.len()] {
            unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
            return Some(module_entry.modBaseAddr as usize);
        }

        if unsafe { Module32NextW(snapshot, &mut module_entry) } == 0 {
            break;
        }
    }

    unsafe { winapi::um::handleapi::CloseHandle(snapshot) };
    None
}

pub fn open_process(process_id: u32) -> io::Result<HANDLE> {
    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_id) };
    if handle == null_mut() {
        Err(io::Error::last_os_error())
    } else {
        Ok(handle)
    }
}

pub fn close_handle(handle: HANDLE) {
    unsafe { winapi::um::handleapi::CloseHandle(handle) };
}