mod memory;
mod process;
mod utils;

use termcolor::{Color, ColorChoice, StandardStream};
use memory::Memory;
use process::{get_process_id_by_name, get_module_base_address};
use utils::{format_address, TermColors};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let colors = TermColors;
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    
    utils::show_credits(&mut stdout)?;
    
    colors.write_header(&mut stdout, "init", Color::Yellow)?;
    writeln!(&mut stdout, " Searching for Roblox process...")?;
    stdout.flush()?;

    let process_id = match get_process_id_by_name("RobloxPlayerBeta.exe") {
        Some(id) => id,
        None => {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);
            colors.write_header(&mut stderr, "error", Color::Red)?;
            writeln!(&mut stderr, " Failed to find Roblox process")?;
            return Ok(());
        }
    };

    colors.write_header(&mut stdout, "success", Color::Green)?;
    writeln!(&mut stdout, " Found Roblox process ID: {}", process_id)?;

    
    colors.write_header(&mut stdout, "init", Color::Yellow)?;
    writeln!(&mut stdout, " Acquiring process handle...")?;
    stdout.flush()?;

    let process_handle = match process::open_process(process_id) {
        Ok(handle) => handle,
        Err(e) => {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);
            colors.write_header(&mut stderr, "error", Color::Red)?;
            writeln!(&mut stderr, " Failed to open process: {}", e)?;
            return Ok(());
        }
    };

    colors.write_header(&mut stdout, "success", Color::Green)?;
    writeln!(&mut stdout, " Process handle acquired")?;

    
    colors.write_header(&mut stdout, "init", Color::Yellow)?;
    writeln!(&mut stdout, " Getting base address...")?;
    stdout.flush()?;

    let base_address = match get_module_base_address(process_id, "RobloxPlayerBeta.exe") {
        Some(addr) => addr,
        None => {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);
            colors.write_header(&mut stderr, "error", Color::Red)?;
            writeln!(&mut stderr, " Failed to get base address")?;
            return Ok(());
        }
    };

    colors.write_header(&mut stdout, "success", Color::Green)?;
    writeln!(&mut stdout, " Base address: {}", format_address(base_address))?;

    let mem = Memory::new(process_handle, base_address);

    
    colors.write_header(&mut stdout, "MEOW", Color::Magenta)?;
    writeln!(&mut stdout, " Starting scan...")?;
    stdout.flush()?;

    let total_start_time = std::time::Instant::now();
    let datamodel = mem.aob_scan_all("RenderJob(EarlyRendering;", false, 1);

    if !datamodel.is_empty() {
        const RENDERVIEW_OFFSET: usize = 0x1E8;
        
        
        let mut valid_dm = None;
        for dm_addr in &datamodel {
            if let Ok(_test_read) = mem.read::<[u8; 0x200]>(*dm_addr) {
                let render_view = mem.read::<usize>(*dm_addr + RENDERVIEW_OFFSET);
                match render_view {
                    Ok(rv_addr) if rv_addr > 0x0000000100000000 && rv_addr < 0x00007FFFFFFEFFFF => {
                        valid_dm = Some(*dm_addr);
                        break;
                    },
                    Err(e) => {
                        colors.write_header(&mut stdout, "warn", Color::Yellow)?;
                        writeln!(&mut stdout, " Invalid RenderView at {}: {}", 
                            format_address(*dm_addr + RENDERVIEW_OFFSET), e)?;
                        break;
                    },
                    _ => continue
                }
            }
        }

        let dm_addr = match valid_dm {
            Some(addr) => addr,
            None => {
                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                colors.write_header(&mut stderr, "error", Color::Red)?;
                writeln!(&mut stderr, " No valid DataModel addresses found")?;
                return Ok(());
            }
        };

        colors.write_header(&mut stdout, "found", Color::Green)?;
        writeln!(&mut stdout, " DataModel pattern at: {}", format_address(dm_addr))?;

        
        const FAKE_OFFSET: usize = 0x118;
        const REAL_OFFSET: usize = 0x1A8;

        let render_view = match mem.read::<usize>(dm_addr + RENDERVIEW_OFFSET) {
            Ok(addr) => addr,
            Err(e) => {
                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                colors.write_header(&mut stderr, "error", Color::Red)?;
                writeln!(&mut stderr, " Failed to read RenderView: {}", e)?;
                return Ok(());
            }
        };
        
        colors.write_header(&mut stdout, "read", Color::Cyan)?;
        writeln!(&mut stdout, " RenderView address: {}", format_address(render_view))?;

        let fake_data_model = match mem.read::<usize>(render_view + FAKE_OFFSET) {
            Ok(addr) => addr,
            Err(e) => {
                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                colors.write_header(&mut stderr, "error", Color::Red)?;
                writeln!(&mut stderr, " Failed to read FakeDataModel: {}", e)?;
                return Ok(());
            }
        };
        
        colors.write_header(&mut stdout, "read", Color::Cyan)?;
        writeln!(&mut stdout, " FakeDataModel address: {}", format_address(fake_data_model))?;

        let real_data_model = match mem.read::<usize>(fake_data_model + REAL_OFFSET) {
            Ok(addr) => addr,
            Err(e) => {
                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                colors.write_header(&mut stderr, "error", Color::Red)?;
                writeln!(&mut stderr, " Failed to read RealDataModel: {}", e)?;
                return Ok(());
            }
        };
        
        colors.write_header(&mut stdout, "read", Color::Cyan)?;
        writeln!(&mut stdout, " RealDataModel address: {}", format_address(real_data_model))?;

        
        let total_duration = total_start_time.elapsed();
        colors.write_header(&mut stdout, "success", Color::Green)?;
        writeln!(&mut stdout, " Scan completed successfully!")?;
        
        colors.write_header(&mut stdout, "time", Color::Yellow)?;
        writeln!(&mut stdout, " Operation took {:.3}ms ({:.3}s)", 
            total_duration.as_millis(), total_duration.as_secs_f64())?;
    } else {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        colors.write_header(&mut stderr, "error", Color::Red)?;
        writeln!(&mut stderr, " Failed to find DataModel pattern")?;
    }

    colors.write_header(&mut stdout, "MEOW", Color::Magenta)?;
    writeln!(&mut stdout, " Scan complete!")?;
    
    writeln!(&mut stdout, "\nPress Enter to exit...")?;
    let _ = std::io::stdin().read_line(&mut String::new());

    process::close_handle(process_handle);
    Ok(())
}