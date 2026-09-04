//! Validation command - checks packaged addon layout and required artifacts

use crate::bundle_common::validate_required_paths;
use crate::platform::{
    MACOS_CEF_APP_PATH, MACOS_CEF_FRAMEWORKS, MACOS_EXTENSION_FRAMEWORK,
    MACOS_EXTENSION_INSTALL_NAME, MACOS_HELPERS, MACOS_UNIVERSAL_TARGET, PLATFORM_SPECS,
};
use plist::{Dictionary, Value};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::process::Command;

fn load_bundle_plist(path: &Path) -> Result<Dictionary, Box<dyn std::error::Error>> {
    let value = Value::from_file(path)?;
    value
        .into_dictionary()
        .ok_or_else(|| format!("{} is not a plist dictionary", path.display()).into())
}

fn required_plist_string<'a>(
    plist: &'a Dictionary,
    key: &str,
    path: &Path,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    plist
        .get(key)
        .and_then(Value::as_string)
        .ok_or_else(|| format!("{} is missing string key {}", path.display(), key).into())
}

fn validate_bundle_executable(
    bundle_path: &Path,
    plist_path: &Path,
    executable_dir: &Path,
    package_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let plist = load_bundle_plist(plist_path)?;
    let actual_package_type = required_plist_string(&plist, "CFBundlePackageType", plist_path)?;
    if actual_package_type != package_type {
        return Err(format!(
            "{} declares CFBundlePackageType={}, expected {}",
            plist_path.display(),
            actual_package_type,
            package_type
        )
        .into());
    }

    let executable = required_plist_string(&plist, "CFBundleExecutable", plist_path)?;
    let executable_path = bundle_path.join(executable_dir).join(executable);
    if !executable_path.is_file() {
        return Err(format!(
            "{} declares missing executable {}",
            plist_path.display(),
            executable_path.display()
        )
        .into());
    }
    Ok(())
}

fn validate_framework(
    framework_path: &Path,
    require_godot_export_metadata: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let plist_path = framework_path.join("Resources/Info.plist");
    validate_bundle_executable(framework_path, &plist_path, Path::new(""), "FMWK")?;

    if require_godot_export_metadata {
        let plist = load_bundle_plist(&plist_path)?;
        let supports_macos = plist
            .get("CFBundleSupportedPlatforms")
            .and_then(Value::as_array)
            .is_some_and(|platforms| {
                platforms
                    .iter()
                    .any(|platform| platform.as_string() == Some("MacOSX"))
            });
        if !supports_macos {
            return Err(format!(
                "{} must include MacOSX in CFBundleSupportedPlatforms",
                plist_path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn validate_app(app_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    validate_bundle_executable(
        app_path,
        &app_path.join("Contents/Info.plist"),
        Path::new("Contents/MacOS"),
        "APPL",
    )
}

fn validate_macos_bundle_structure(platform_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let extension_framework = platform_dir.join(MACOS_EXTENSION_FRAMEWORK);
    validate_framework(&extension_framework, true)?;

    let cef_app = extension_framework.join(MACOS_CEF_APP_PATH);
    validate_app(&cef_app)?;
    let nested_frameworks_dir = cef_app.join("Contents/Frameworks");

    for framework in MACOS_CEF_FRAMEWORKS {
        validate_framework(&nested_frameworks_dir.join(framework), false)?;
    }

    for helper in MACOS_HELPERS {
        validate_app(&nested_frameworks_dir.join(helper).with_extension("app"))?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn verify_code_signature(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict", "--verbose=2"])
        .arg(path)
        .output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "invalid code signature for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

#[cfg(target_os = "macos")]
fn validate_install_name(framework_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let library_path = framework_path.join("libgdcef.dylib");
    let output = Command::new("otool")
        .arg("-D")
        .arg(&library_path)
        .output()?;
    if !output.status.success() {
        return Err(format!("otool failed for {}", library_path.display()).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout
        .lines()
        .any(|line| line.trim() == MACOS_EXTENSION_INSTALL_NAME)
    {
        return Ok(());
    }

    Err(format!(
        "{} does not use relocatable install name {}",
        library_path.display(),
        MACOS_EXTENSION_INSTALL_NAME
    )
    .into())
}

fn validate_macos(platform_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    validate_macos_bundle_structure(platform_dir)?;

    #[cfg(target_os = "macos")]
    {
        let extension_framework = platform_dir.join(MACOS_EXTENSION_FRAMEWORK);
        validate_install_name(&extension_framework)?;
        verify_code_signature(&extension_framework)?;
        verify_code_signature(&extension_framework.join(MACOS_CEF_APP_PATH))?;
    }
    Ok(())
}

pub fn run(addon_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = addon_dir.join("bin");
    if !bin_dir.exists() {
        return Err(format!(
            "Addon directory '{}' does not contain a bin/ directory",
            addon_dir.display()
        )
        .into());
    }

    let mut validated = 0usize;
    for platform in PLATFORM_SPECS {
        let platform_dir = bin_dir.join(platform.target);
        if !platform_dir.exists() {
            println!("Skipping {} (not present)", platform.target);
            continue;
        }

        validate_required_paths(
            &platform_dir,
            platform.required_files,
            platform.required_dirs,
        )?;
        if platform.target == MACOS_UNIVERSAL_TARGET {
            validate_macos(&platform_dir)?;
        }
        println!("Validated {}", platform.target);
        validated += 1;
    }

    if validated == 0 {
        return Err("No platform directories found under addon bin/".into());
    }

    println!(
        "Validation complete: {} platform(s) checked in {}",
        validated,
        addon_dir.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            let path = std::env::temp_dir()
                .join(format!("godot-cef-{name}-{}-{nonce}", std::process::id()));
            assert!(fs::create_dir_all(&path).is_ok());
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.0) {
                eprintln!(
                    "failed to remove test directory {}: {error}",
                    self.0.display()
                );
            }
        }
    }

    fn write_framework_plist(
        framework_path: &Path,
        executable: &str,
        supported_platforms: Option<Vec<&str>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resources_path = framework_path.join("Resources");
        fs::create_dir_all(&resources_path)?;

        let mut plist = Dictionary::new();
        plist.insert(
            "CFBundleExecutable".to_string(),
            Value::String(executable.to_string()),
        );
        plist.insert(
            "CFBundlePackageType".to_string(),
            Value::String("FMWK".to_string()),
        );
        if let Some(platforms) = supported_platforms {
            plist.insert(
                "CFBundleSupportedPlatforms".to_string(),
                Value::Array(
                    platforms
                        .into_iter()
                        .map(|platform| Value::String(platform.to_string()))
                        .collect(),
                ),
            );
        }
        plist::to_file_xml(resources_path.join("Info.plist"), &plist)?;
        Ok(())
    }

    #[test]
    fn framework_validation_rejects_missing_supported_platform()
    -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TestDir::new("missing-platform");
        let framework_path = test_dir.0.join("Godot CEF.framework");
        write_framework_plist(&framework_path, "libgdcef.dylib", None)?;
        fs::write(framework_path.join("libgdcef.dylib"), [])?;

        let Err(error) = validate_framework(&framework_path, true) else {
            return Err("Godot-incompatible framework plist should fail".into());
        };
        if !error.to_string().contains("CFBundleSupportedPlatforms") {
            return Err(format!("unexpected validation error: {error}").into());
        }
        Ok(())
    }

    #[test]
    fn framework_validation_rejects_missing_declared_executable()
    -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TestDir::new("missing-executable");
        let framework_path = test_dir.0.join("Godot CEF.framework");
        write_framework_plist(&framework_path, "Godot CEF", Some(vec!["MacOSX"]))?;

        let Err(error) = validate_framework(&framework_path, true) else {
            return Err("framework with a missing executable should fail".into());
        };
        if !error.to_string().contains("declares missing executable") {
            return Err(format!("unexpected validation error: {error}").into());
        }
        Ok(())
    }
}
