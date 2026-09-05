use crate::bundle_common::{
    AppInfoPlist, TARGET_ARM64, TARGET_X64, copy_directory, get_cef_dir_arm64, get_cef_dir_x64,
    get_target_dir, get_target_dir_for_target, run_cargo_for_macos_targets, run_lipo,
    sign_macos_code,
};
use crate::platform::{MACOS_CEF_FRAMEWORK_ARM64, MACOS_CEF_FRAMEWORK_X64, MACOS_HELPERS};
use std::fs;
use std::path::{Path, PathBuf};

const EXEC_PATH: &str = "Contents/MacOS";
const FRAMEWORKS_PATH: &str = "Contents/Frameworks";
const FRAMEWORK: &str = "Chromium Embedded Framework.framework";

fn sign_cef_framework(framework_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(framework_path.join("Libraries"))? {
        let library_path = entry?.path();
        if library_path
            .extension()
            .is_some_and(|extension| extension == "dylib")
        {
            sign_macos_code(&library_path)?;
        }
    }
    sign_macos_code(framework_path)
}

fn create_app(
    app_path: &Path,
    exec_name: &str,
    bin: &Path,
    is_helper: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_path = app_path.join(exec_name).with_extension("app");
    if app_path.exists() {
        fs::remove_dir_all(&app_path)?;
    }
    fs::create_dir_all(app_path.join(EXEC_PATH))?;
    let info_plist = AppInfoPlist::new(exec_name, is_helper);
    plist::to_file_xml(app_path.join("Contents/Info.plist"), &info_plist)?;
    fs::copy(bin, app_path.join(EXEC_PATH).join(exec_name))?;
    Ok(app_path)
}

fn bundle(
    target_dir: &Path,
    universal_helper: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let main_app_path = create_app(target_dir, "Godot CEF", universal_helper, false)?;

    let cef_path_arm64 = get_cef_dir_arm64()
        .ok_or("CEF ARM64 directory not found. Please set CEF_PATH_ARM64 environment variable.")?;
    let to_arm64 = main_app_path
        .join(FRAMEWORKS_PATH)
        .join(MACOS_CEF_FRAMEWORK_ARM64);
    copy_directory(&cef_path_arm64.join(FRAMEWORK), &to_arm64)?;
    println!("Copied: {MACOS_CEF_FRAMEWORK_ARM64}");

    let cef_path_x64 = get_cef_dir_x64()
        .ok_or("CEF X64 directory not found. Please set CEF_PATH_X64 environment variable.")?;
    let to_x64 = main_app_path
        .join(FRAMEWORKS_PATH)
        .join(MACOS_CEF_FRAMEWORK_X64);
    copy_directory(&cef_path_x64.join(FRAMEWORK), &to_x64)?;
    println!("Copied: {MACOS_CEF_FRAMEWORK_X64}");

    sign_cef_framework(&to_arm64)?;
    sign_cef_framework(&to_x64)?;

    for helper in MACOS_HELPERS {
        let helper_path = create_app(
            &main_app_path.join(FRAMEWORKS_PATH),
            helper,
            universal_helper,
            true,
        )?;
        sign_macos_code(&helper_path)?;
    }

    sign_macos_code(&main_app_path)?;

    println!("Created: {}", main_app_path.display());
    Ok(main_app_path)
}

pub fn build(
    release: bool,
    target_dir: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    run_cargo_for_macos_targets(&["build", "--bin", "gdcef_helper"], release)?;

    let target_dir_arm64 = get_target_dir_for_target(release, TARGET_ARM64, target_dir);
    let target_dir_x64 = get_target_dir_for_target(release, TARGET_X64, target_dir);
    let output_dir = get_target_dir(release, target_dir);

    let helper_arm64 = target_dir_arm64.join("gdcef_helper");
    let helper_x64 = target_dir_x64.join("gdcef_helper");
    let universal_helper = output_dir.join("gdcef_helper_universal");

    run_lipo(&helper_arm64, &helper_x64, &universal_helper)?;

    let app_path = bundle(&output_dir, &universal_helper)?;
    fs::remove_file(&universal_helper)?;
    Ok(app_path)
}
