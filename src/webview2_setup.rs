use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, copy};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use std::{ptr::null_mut};
use bindings::Windows::Win32::System::Registry::{HKEY, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, REG_SZ, RRF_RT_REG_SZ, RegGetValueA, RegOpenKeyExA, RegQueryValueExA};
use bindings::Windows::Win32::Foundation::MAX_PATH;


pub fn check_webview2_installed() -> bool {
    let rkey = r#"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"#;
    #[cfg(target_arch="i686")]
    let rkey = r#"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"#;

    let mut hkey = HKEY::default();

    let mut buf = [0; MAX_PATH as usize +1];
    let mut bsize = 0;

    // open skey
    let _ = unsafe{RegOpenKeyExA(HKEY_LOCAL_MACHINE, rkey, 0, KEY_QUERY_VALUE, &mut hkey)};

    // query svalue
    let skey = "pv";

    let _ = unsafe {RegQueryValueExA(hkey, skey, null_mut(), null_mut(), buf.as_mut_ptr() as _, &mut bsize)};
    // if we need to read the data oout, we need to read again
    // let status = unsafe {RegQueryValueExA(hkey, skey, null_mut(), null_mut(), buf.as_mut_ptr() as _, &mut bsize)};

    bsize > 0
}

// download file from https://go.microsoft.com/fwlink/p/?LinkId=2124703
// store the temp dir
pub fn download_webview2_boostrap(url: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
    let r_url = url.unwrap_or("https://go.microsoft.com/fwlink/p/?LinkId=2124703");

    let tmp_dir = env::temp_dir();
    let resp = reqwest::blocking::get(r_url)?;

    let d_path= {
        let f_name = resp.url().path_segments()
            .and_then(|segs| segs.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("webview2-stepup.exe");
        tmp_dir.as_path().join(f_name)
    };

    // create dest file
    let mut d_file = File::create(&d_path)?;
    // write bytes
    let content =  resp.bytes()?;

    copy(&mut Cursor::new(content), &mut d_file)?;

    Ok(d_path)
}

// install webview2 setup execute
// webview2setup.exe /silent /install
pub fn run_webview2_setup<P: AsRef<Path>>(target: P, silent: Option<bool>) -> Result<(), Box<dyn Error>> {
    // start exec the setup webview2
    let mut p = Command::new(target.as_ref());

    if silent.unwrap_or(true) {
        p.arg("/silent");
    }

    p.arg("/install");
    
    p.status()?;

    Ok(())
}

pub fn makesure_webview2<P: AsRef<Path>>(target: Option<P>, silent: Option<bool>) -> Result<(), Box<dyn Error>> {
    // if we installed just return
    if check_webview2_installed() {
        return Ok(());
    }

    // check if we exits
    match target {
        Some(p) if p.as_ref().exists() => {
            // go installed 
            return run_webview2_setup(p, silent);
        }
        _ => {}
    }

    // download and install
    // TODO: add url for donwload
    let p = download_webview2_boostrap(None)?;
    if !p.exists() {
        // donwload file not exits
        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "target file not exits")))
    }

   run_webview2_setup(p, silent)
} 

#[cfg(test)]
mod tests {
    use crate::webview2_setup::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }


    #[test]
    fn test_check_install() {
        assert!(check_webview2_installed())
    }

    #[test]
    fn test_makesure_webview2() {
        let _ = makesure_webview2::<PathBuf>(None, Some(false));
        assert!(check_webview2_installed())
    }

    #[test]
    fn test_webview_setup() {
        let p = download_webview2_boostrap(None);
        assert!(p.is_ok());
        let p = p.unwrap();
        // check target file exits
        assert!(&p.exists());

        // isntall 
        let r = run_webview2_setup(p, Some(false));
        assert!(r.is_ok());
    }
}