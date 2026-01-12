#!/bin/bash

function remove_quarantine() {
    if [ -e "$1" ]; then
        xattr -dr com.apple.quarantine "$1"
    else
        echo "$1 not found."
        exit 1
    fi
}

function add_executable_flag() {
    if [ -e "$1" ]; then
        chmod +x "$1"
    else
        echo "$1 not found."
        exit 1
    fi
}

remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/MacOS/Godot CEF"
remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper.app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper.app/Contents/MacOS/Godot CEF Helper"
remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Alerts).app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Alerts).app/Contents/MacOS/Godot CEF Helper (Alerts)"
remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (GPU).app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (GPU).app/Contents/MacOS/Godot CEF Helper (GPU)"
remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Plugin).app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Plugin).app/Contents/MacOS/Godot CEF Helper (Plugin)"
remove_quarantine "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Renderer).app"
add_executable_flag "./bin/universal-apple-darwin/Godot CEF.app/Contents/Frameworks/Godot CEF Helper (Renderer).app/Contents/MacOS/Godot CEF Helper (Renderer)"

remove_quarantine "./bin/universal-apple-darwin/Godot CEF.framework/libgdcef.dylib"
