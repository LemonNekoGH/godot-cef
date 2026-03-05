# IPC 检查器（IPC Inspector）

`CefIpcInspector` 是一个内置开发调试覆盖层，用于查看某个 `CefTexture` 的 Godot 与 CEF 渲染进程之间的 IPC 消息。

它可以帮助你快速确认：
- 双向 IPC 是否正常流动
- 当前使用的是哪条通道（`text` / `binary` / `data`）
- 消息预览内容和字节大小
- 消息时序和到达顺序

## 可用性

出于安全考虑，IPC 检查器仅在以下场景启用：
- Godot 为调试构建（`OS.is_debug_build() == true`），或
- 从编辑器运行（`Engine.is_editor_hint() == true`）

在发布版中，检查器 UI 不会初始化。

## 快速开始

无需写脚本。

1. 在场景中同时添加 `CefTexture` 和 `CefIpcInspector`。
2. 在编辑器里选中 `CefIpcInspector` 节点。
3. 在 Inspector 面板中，把 `CefTexture` 节点拖到 `target_cef_texture` 属性上。
4. 以编辑器运行或 Debug 模式运行场景。
5. 点击右下角 `IPC Inspector` 按钮展开面板。

绑定完成后会立即开始监听 IPC 消息。

### 可选：用脚本设置目标

```gdscript
@onready var browser: CefTexture = $CefTexture
@onready var inspector: CefIpcInspector = $CefIpcInspector

func _ready():
    inspector.target_cef_texture = browser
```

## 生成测试消息

```gdscript
func send_test_messages():
    browser.send_ipc_message("hello from godot")
    browser.send_ipc_binary_message(PackedByteArray([0xDE, 0xAD, 0xBE, 0xEF]))
    browser.send_ipc_data(["score", 100, true])
```

网页 JavaScript 侧：

```javascript
window.sendIpcMessage("hello from js");
window.sendIpcBinaryMessage(new Uint8Array([1, 2, 3, 4]).buffer);
window.sendIpcData(["inventory", "potion", 3]);
```

## 面板功能

- `All / Incoming / Outgoing` 按方向筛选消息
- `Clear` 清空当前历史
- `Show more / Show less` 展开或折叠长消息
- 历史最多保留 `500` 条（超出后最旧消息会被丢弃）

## `debug_ipc_message` 事件结构

检查器内部监听 `CefTexture.debug_ipc_message(event: Variant)`，其中 `event` 为 `Dictionary`：

| 字段 | 类型 | 说明 |
|------|------|------|
| `direction` | `String` | `to_renderer` 或 `to_godot` |
| `lane` | `String` | `text`、`binary`、`data` |
| `body` | `String` | 消息预览（`binary` 通道以十六进制预览显示） |
| `timestamp_unix_ms` | `int` | Unix 毫秒时间戳 |
| `body_size_bytes` | `int` | 原始负载字节数 |

## 演示视频

这一节建议同时放短视频和静态预览图。

- 视频链接模板：`[IPC Inspector 演示](https://example.com/replace-with-your-video-link)`
- 建议时长：`30-60` 秒
- 建议镜头流程：
  1. 打开检查器面板
  2. 分别触发 `text` / `binary` / `data` 三种消息
  3. 切换方向筛选
  4. 展开长消息并点击 `Clear`

如果视频要随文档一起托管，可放在 `docs/public/videos/` 下，并使用：

```html
<video controls preload="metadata" style="width: 100%;">
  <source src="/videos/cef-ipc-inspector-demo.mp4" type="video/mp4" />
</video>
```

## 常见问题

- 面板始终不出现：
  - 确认当前是 Debug 构建或编辑器运行模式。
- 显示 `Assign target_cef_texture to a CefTexture node.`：
  - 需要把 `target_cef_texture` 指向实际浏览器节点。
- 没有任何消息：
  - 确认消息确实被发送，并检查当前筛选条件。
- 大消息缺失：
  - 超过安全上限的 IPC Data 负载会被丢弃并输出日志。
