use failure::Error;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

pub fn unzip(input_file: &PathBuf, output_dir: &PathBuf) -> Result<(), Error> {
    let zipfile_pointer = File::open(&input_file)?;
    let mut archive = zip::ZipArchive::new(zipfile_pointer)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.sanitized_name();

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(output_dir.join(&outpath))?;
        } else {
            if let Some(p) = output_dir.join(&outpath).parent() {
                if !p.exists() {
                    fs::create_dir_all(output_dir.join(&p))?;
                }
            }
            let next_path = output_dir.join(&outpath);
            let mut outfile = fs::File::create(next_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(output_dir.join(&outpath), fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}
