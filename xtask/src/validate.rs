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
) -> Result<Dictionary, Box<dyn std::error::Error>> {
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
    Ok(plist)
}

fn validate_framework(framework_path: &Path) -> Result<Dictionary, Box<dyn std::error::Error>> {
    let plist_path = framework_path.join("Resources/Info.plist");
    validate_bundle_executable(framework_path, &plist_path, Path::new(""), "FMWK")
}

fn validate_godot_framework(framework_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let plist_path = framework_path.join("Resources/Info.plist");
    let plist = validate_framework(framework_path)?;

    for key in [
        "CFBundleIdentifier",
        "CFBundleInfoDictionaryVersion",
        "CFBundleName",
    ] {
        required_plist_string(&plist, key, &plist_path)?;
    }

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
    Ok(())
}

fn validate_app(app_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    validate_bundle_executable(
        app_path,
        &app_path.join("Contents/Info.plist"),
        Path::new("Contents/MacOS"),
        "APPL",
    )?;
    Ok(())
}

fn validate_macos_bundle_structure(platform_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let extension_framework = platform_dir.join(MACOS_EXTENSION_FRAMEWORK);
    validate_godot_framework(&extension_framework)?;

    let cef_app = extension_framework.join(MACOS_CEF_APP_PATH);
    validate_app(&cef_app)?;
    let nested_frameworks_dir = cef_app.join("Contents/Frameworks");

    for framework in MACOS_CEF_FRAMEWORKS {
        validate_framework(&nested_frameworks_dir.join(framework))?;
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
    for architecture in ["arm64", "x86_64"] {
        let output = Command::new("otool")
            .args(["-arch", architecture, "-D"])
            .arg(&library_path)
            .output()?;
        let has_relocatable_id = output.status.success()
            && String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.trim() == MACOS_EXTENSION_INSTALL_NAME);
        if !has_relocatable_id {
            return Err(format!(
                "{} does not use relocatable install name {} for {}",
                library_path.display(),
                MACOS_EXTENSION_INSTALL_NAME,
                architecture
            )
            .into());
        }
    }
    Ok(())
}

fn validate_macos(platform_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    validate_macos_bundle_structure(platform_dir)?;

    #[cfg(target_os = "macos")]
    {
        let extension_framework = platform_dir.join(MACOS_EXTENSION_FRAMEWORK);
        validate_install_name(&extension_framework)?;

        let cef_frameworks_dir = extension_framework
            .join(MACOS_CEF_APP_PATH)
            .join("Contents/Frameworks");
        for framework in MACOS_CEF_FRAMEWORKS {
            for entry in std::fs::read_dir(cef_frameworks_dir.join(framework).join("Libraries"))? {
                let library_path = entry?.path();
                if library_path
                    .extension()
                    .is_some_and(|extension| extension == "dylib")
                {
                    verify_code_signature(&library_path)?;
                }
            }
        }

        verify_code_signature(&extension_framework)?;
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_framework(
        framework_path: &Path,
        plist: &Dictionary,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resources_path = framework_path.join("Resources");
        fs::create_dir_all(&resources_path)?;
        plist::to_file_xml(resources_path.join("Info.plist"), plist)?;
        fs::write(framework_path.join("libgdcef.dylib"), [])?;
        Ok(())
    }

    #[test]
    fn godot_framework_validation_rejects_export_incompatible_plists()
    -> Result<(), Box<dyn std::error::Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let test_dir = std::env::temp_dir().join(format!(
            "godot-cef-framework-validation-{}-{nonce}",
            std::process::id()
        ));

        let mut valid_plist = Dictionary::new();
        for (key, value) in [
            ("CFBundleExecutable", "libgdcef.dylib"),
            ("CFBundleIdentifier", "me.delton.gdcef.libgdcef"),
            ("CFBundleInfoDictionaryVersion", "6.0"),
            ("CFBundleName", "gdcef"),
            ("CFBundlePackageType", "FMWK"),
        ] {
            valid_plist.insert(key.to_string(), Value::String(value.to_string()));
        }
        valid_plist.insert(
            "CFBundleSupportedPlatforms".to_string(),
            Value::Array(vec![Value::String("MacOSX".to_string())]),
        );

        let framework_path = test_dir.join("valid").join("Godot CEF.framework");
        write_framework(&framework_path, &valid_plist)?;
        validate_godot_framework(&framework_path)?;

        for missing_key in [
            "CFBundleExecutable",
            "CFBundleIdentifier",
            "CFBundleInfoDictionaryVersion",
            "CFBundleName",
            "CFBundlePackageType",
            "CFBundleSupportedPlatforms",
        ] {
            let framework_path = test_dir.join(missing_key).join("Godot CEF.framework");
            let mut plist = valid_plist.clone();
            plist.remove(missing_key);
            write_framework(&framework_path, &plist)?;
            if validate_godot_framework(&framework_path).is_ok() {
                return Err(format!("framework without {missing_key} passed validation").into());
            }
        }

        let framework_path = test_dir
            .join("wrong-executable")
            .join("Godot CEF.framework");
        let mut plist = valid_plist;
        plist.insert(
            "CFBundleExecutable".to_string(),
            Value::String("Godot CEF".to_string()),
        );
        write_framework(&framework_path, &plist)?;
        if validate_godot_framework(&framework_path).is_ok() {
            return Err("framework with a missing declared executable passed validation".into());
        }

        fs::remove_dir_all(test_dir)?;
        Ok(())
    }
}
