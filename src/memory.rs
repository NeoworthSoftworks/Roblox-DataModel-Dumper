use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READWRITE, PAGE_READONLY};
use winapi::um::winnt::HANDLE;
use std::ptr::null_mut;
use std::io;

pub struct Memory {
    process_handle: HANDLE,
    module_base: usize,
}

impl Memory {
    pub fn new(process_handle: HANDLE, module_base: usize) -> Self {
        Memory { process_handle, module_base }
    }

    pub fn string_to_pattern(&self, pattern: &str) -> Vec<u8> {
        pattern.as_bytes().to_vec()
    }

    pub fn read<T: Copy>(&self, address: usize) -> io::Result<T> {
        let mut buffer: T = unsafe { std::mem::zeroed() };
        let result = unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const _,
                &mut buffer as *mut _ as *mut _,
                std::mem::size_of::<T>(),
                null_mut(),
            )
        };
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(buffer)
        }
    }

    pub fn aob_scan_all(&self, pattern: &str, return_multiple: bool, stop_at_value: usize) -> Vec<usize> {
        let byte_pattern = self.string_to_pattern(pattern);
        let pattern_size = byte_pattern.len();
        if pattern_size == 0 {
            return Vec::new();
        }

        let first_byte = byte_pattern[0];
        let mut results = Vec::new();
        let mut address = 0;

        loop {
            let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
            let query_result = unsafe {
                VirtualQueryEx(
                    self.process_handle,
                    address as *const _,
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };

            if query_result == 0 {
                break;
            }

            let region_size = mbi.RegionSize;
            if mbi.State != MEM_COMMIT || 
                !(mbi.Protect == PAGE_EXECUTE_READ ||
                  mbi.Protect == PAGE_EXECUTE_READWRITE ||
                  mbi.Protect == PAGE_READWRITE ||
                  mbi.Protect == PAGE_READONLY) 
            {
                address = mbi.BaseAddress as usize + region_size;
                continue;
            }

            let mut buffer = vec![0u8; region_size];
            let mut bytes_read = 0;

            let read_success = unsafe {
                ReadProcessMemory(
                    self.process_handle,
                    mbi.BaseAddress,
                    buffer.as_mut_ptr() as *mut _,
                    region_size,
                    &mut bytes_read,
                )
            };

            if read_success == 0 || bytes_read < pattern_size {
                address = mbi.BaseAddress as usize + region_size;
                continue;
            }

            for i in 0..=(bytes_read - pattern_size) {
                if buffer[i] != first_byte {
                    continue;
                }

                let mut found = true;
                for j in 1..pattern_size {
                    if byte_pattern[j] != 0x00 && byte_pattern[j] != buffer[i + j] {
                        found = false;
                        break;
                    }
                }

                if found {
                    let found_address = mbi.BaseAddress as usize + i;
                    results.push(found_address);
                    
                    if !return_multiple || (stop_at_value > 0 && results.len() >= stop_at_value) {
                        return results;
                    }
                }
            }

            address = mbi.BaseAddress as usize + region_size;
        }

        results
    }
}