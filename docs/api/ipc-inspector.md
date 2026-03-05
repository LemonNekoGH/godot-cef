# IPC Inspector

`CefIpcInspector` is a built-in developer overlay for inspecting IPC traffic between Godot and CEF renderer processes for a specific `CefTexture`.

It helps you verify:
- Whether IPC is flowing in both directions
- Which lane is used (`text`, `binary`, `data`)
- The actual payload preview and byte size
- Message order and timing

## Availability

For safety, IPC inspector is only enabled when:
- Godot runs in debug mode (`OS.is_debug_build() == true`), or
- Running from the editor (`Engine.is_editor_hint() == true`)

In release builds, the inspector UI is not initialized.

## Quick Start

No scripting is required.

1. Add both `CefTexture` and `CefIpcInspector` to your scene.
2. Select the `CefIpcInspector` node in the editor.
3. In the Inspector panel, drag your `CefTexture` node into `target_cef_texture`.
4. Run the scene in editor/debug mode.
5. Click `IPC Inspector` in the bottom-right corner to open the panel.

The inspector will start listening immediately after the target is assigned.

### Optional: Assign Target in Script

```gdscript
@onready var browser: CefTexture = $CefTexture
@onready var inspector: CefIpcInspector = $CefIpcInspector

func _ready():
    inspector.target_cef_texture = browser
```

## Generate Test Messages

```gdscript
func send_test_messages():
    browser.send_ipc_message("hello from godot")
    browser.send_ipc_binary_message(PackedByteArray([0xDE, 0xAD, 0xBE, 0xEF]))
    browser.send_ipc_data(["score", 100, true])
```

From JavaScript:

```javascript
window.sendIpcMessage("hello from js");
window.sendIpcBinaryMessage(new Uint8Array([1, 2, 3, 4]).buffer);
window.sendIpcData(["inventory", "potion", 3]);
```

## Panel Features

- `All / Incoming / Outgoing` filter by message direction
- `Clear` resets the current history
- `Show more / Show less` expands long payloads
- Maximum history is `500` messages (oldest entries are dropped first)

## `debug_ipc_message` Payload

The inspector internally listens to `CefTexture.debug_ipc_message(event: Variant)` where `event` is a `Dictionary`:

| Key | Type | Description |
|-----|------|-------------|
| `direction` | `String` | `to_renderer` or `to_godot` |
| `lane` | `String` | `text`, `binary`, or `data` |
| `body` | `String` | Payload preview (`binary` is shown as hex preview) |
| `timestamp_unix_ms` | `int` | Unix timestamp in milliseconds |
| `body_size_bytes` | `int` | Original payload size in bytes |

## Demo Video

Use both a short video and a static preview image in this section.

- Video link template: `[IPC Inspector Demo](https://example.com/replace-with-your-video-link)`
- Recommended length: `30-60` seconds
- Suggested shots:
  1. Open panel
  2. Send one message per lane (`text` / `binary` / `data`)
  3. Switch direction filter
  4. Expand one long payload and press `Clear`

If you host the video inside docs, put it under `docs/public/videos/` and embed:

```html
<video controls preload="metadata" style="width: 100%;">
  <source src="/videos/cef-ipc-inspector-demo.mp4" type="video/mp4" />
</video>
```

## Troubleshooting

- Panel never appears:
  - Confirm you are in debug build or editor run mode.
- `Assign target_cef_texture to a CefTexture node.`:
  - Set `target_cef_texture` to the actual browser node.
- No messages shown:
  - Confirm messages are sent, and check the current direction filter.
- Missing large data messages:
  - Oversized IPC data payloads can be dropped by safety limits and logged.
