use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn move_files_to_current_dir(start_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    // Recursively walk through all subdirectories
    walk_directories(start_dir, dest_dir)?;

    Ok(())
}

fn walk_directories(dir: &Path, target_dir: &Path) -> io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // Move file to target directory
            move_file(&path, target_dir)?;
        } else if path.is_dir() {
            // Recursively process subdirectory
            walk_directories(&path, target_dir)?;
        }
    }

    Ok(())
}

fn move_file(file_path: &Path, target_dir: &Path) -> io::Result<()> {
    if let Some(file_name) = file_path.file_name() {
        let mut target_path = target_dir.join(file_name);

        // Handle name conflicts by adding a number suffix
        let mut counter = 1;
        while target_path.exists() {
            if let Some(stem) = file_path.file_stem() {
                if let Some(extension) = file_path.extension() {
                    let new_name = format!(
                        "{}_{}.{}",
                        stem.to_string_lossy(),
                        counter,
                        extension.to_string_lossy()
                    );
                    target_path = target_dir.join(new_name);
                } else {
                    let new_name = format!("{}_{}", stem.to_string_lossy(), counter);
                    target_path = target_dir.join(new_name);
                }
            }
            counter += 1;
        }

        println!(
            "Moving: {} -> {}",
            file_path.display(),
            target_path.display()
        );
        fs::rename(file_path, target_path)?;
    }

    Ok(())
}

fn pad_and_rename_files(dir_path: &Path) -> io::Result<()> {
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process files, not directories
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();

                if let Some(new_name) = generate_padded_name(&file_name_str) {
                    let new_path = path.parent().unwrap().join(&new_name);

                    // Check if the new name would conflict with an existing file
                    if new_path.exists() && new_path != path {
                        println!(
                            "Warning: Cannot rename '{}' to '{}' - target file already exists",
                            file_name_str, new_name
                        );
                        continue;
                    }

                    // Only rename if the name actually changed
                    if new_name != file_name_str {
                        println!("Renaming: '{}' -> '{}'", file_name_str, new_name);
                        fs::rename(&path, &new_path)?;
                    } else {
                        println!("Skipping: '{}' (already properly formatted)", file_name_str);
                    }
                } else {
                    println!(
                        "Skipping: '{}' (no dash found or invalid format)",
                        file_name_str
                    );
                }
            }
        }
    }

    Ok(())
}

fn generate_padded_name(file_name: &str) -> Option<String> {
    // Find the first dash
    if let Some(dash_pos) = file_name.find('-') {
        let prefix = &file_name[..dash_pos];
        let suffix = &file_name[dash_pos..]; // Include the dash and everything after

        // Check if prefix is numeric (or can be treated as a number)
        if let Ok(num) = prefix.parse::<u32>() {
            // Pad with zeros to make it 3 characters
            let padded_prefix = format!("{:03}", num);
            Some(format!("{}{}", padded_prefix, suffix))
        } else {
            // If prefix is not numeric, pad it as a string to 3 characters with zeros
            if prefix.len() <= 3 {
                let padded_prefix = format!("{:0>3}", prefix);
                Some(format!("{}{}", padded_prefix, suffix))
            } else {
                // If prefix is longer than 3 characters, don't modify
                None
            }
        }
    } else {
        None
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let start_directory = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    if !start_directory.exists() {
        eprintln!(
            "Error: Directory '{}' does not exist",
            start_directory.display()
        );
        return Ok(());
    }
    if !start_directory.is_dir() {
        eprintln!("Error: '{}' is not a directory", start_directory.display());
        return Ok(());
    }

    let dest_directory = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        std::env::current_dir()?
    };

    if !dest_directory.exists() {
        eprintln!(
            "Error: Directory '{}' does not exist",
            dest_directory.display()
        );
        return Ok(());
    }
    if !dest_directory.is_dir() {
        eprintln!("Error: '{}' is not a directory", dest_directory.display());
        return Ok(());
    }

    println!(
        "Moving files from '{}' and its subdirectories to current directory...",
        start_directory.display()
    );

    match move_files_to_current_dir(&start_directory, &dest_directory) {
        Ok(()) => println!("Successfully moved all files!"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
    match pad_and_rename_files(&dest_directory) {
        Ok(()) => println!("Successfully renamed all files!"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }

    Ok(())
}
