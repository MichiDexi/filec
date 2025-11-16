use std::fs;
use std::env;
use std::path::Path;
use rayon::prelude::*;

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		panic!("Wrong usage");
	}

	let path = Path::new(&args[1]);

	if !path.exists() {
		panic!("Path does not exist: {:?}", path);
	}

	if path.is_file() {
		let size = match fs::metadata(path) {
			Ok(meta) => meta.len(),
			Err(e) => {
				eprintln!("Error reading file {:?}: {}", path, e);
					return Ok(());
			}
		};
		println!("File '{}' has {} ({} bytes)", &args[1], human_readable_size(size), size);
	}
	else {
		let size = match dir_size(path) {
			Ok(s) => s,
			Err(e) => {
				eprintln!("Error reading directory: {}", e);
				return Ok(());
			}
		};
		println!("Directory '{}' has {} ({} bytes)", &args[1], human_readable_size(size), size);
	}

	Ok(())
}

fn human_readable_size(bytes: u64) -> String {
	let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
	let mut size = bytes as f64;
	let mut unit = 0;

	while size >= 1024.0 && unit < units.len() - 1 {
		size /= 1024.0;
		unit += 1;
	}
	format!("{:.1} {}", size, units[unit])
}

fn dir_size(path: &Path) -> std::io::Result<u64> {
	let mut total: u64 = 0;

	let path_str = path.to_string_lossy();
	if path_str.starts_with("/proc") || path_str.starts_with("/sys") || path_str.starts_with("/dev") {
		return Ok(0);
	}
	
	let entries = match fs::read_dir(path) {
		Ok(e) => e,
		Err(_) => return Ok(0),  // skip directories you can't access
	};

	total += entries.par_bridge()   // <- turn iterator into parallel
		.filter_map(|entry| entry.ok())
		.map(|entry| {
		let ft = entry.file_type().ok();
		match ft {
			Some(ft) if ft.is_file() => entry.metadata().map(|m| m.len()).unwrap_or(0),
			Some(ft) if ft.is_dir() => dir_size(&entry.path()).unwrap_or(0),
			_ => 0,
		}
	})
	.sum::<u64>();
	Ok(total)
}
