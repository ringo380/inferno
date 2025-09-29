# Tauri v1 → v2 Migration Audit

**Date**: 2025-09-29
**Scope**: Complete functionality comparison between Tauri v1 and v2 implementations

---

## 📊 Command Comparison Matrix

| Command | Tauri v1 (src/tauri_app.rs) | Tauri v2 (dashboard/src-tauri) | Status | Notes |
|---------|------------------------------|--------------------------------|--------|-------|
| **Core Model Operations** |
| `get_models` | ✅ | ✅ | ✅ Parity | Both use ModelManager |
| `get_loaded_models` | ✅ | ✅ | ✅ Parity | - |
| `load_model` | ✅ | ✅ | ✅ Parity | v2 has better event emission |
| `unload_model` | ✅ | ✅ | ✅ Parity | - |
| `validate_model` | ✅ Basic | ✅ Enhanced | ⚠️ v2 Better | v2 uses BackendManager validation |
| **Inference Operations** |
| `infer` | ✅ | ✅ | ✅ Parity | v2 has better error handling |
| `infer_stream` | ❌ | ✅ | 🔴 v2 Only | Streaming inference |
| **System Information** |
| `get_system_info` | ✅ | ✅ | ✅ Parity | - |
| `get_metrics` | ✅ Basic | ✅ Enhanced | ⚠️ v2 Better | v2 has GlobalMetrics |
| `get_inferno_metrics` | ❌ | ✅ | 🔴 v2 Only | Enhanced metrics with CPU/GPU |
| `get_active_processes` | ❌ | ✅ | 🔴 v2 Only | Process monitoring |
| **File Operations** |
| `open_file_dialog` | ✅ Old API | ✅ New Plugin | ⚠️ v2 Better | v2 uses tauri-plugin-dialog |
| `upload_model` | ❌ | ✅ | 🔴 v2 Only | Model upload to local directory |
| **Settings Management** |
| `get_settings` | ❌ | ✅ | 🔴 v2 Only | Comprehensive settings |
| `set_settings` | ❌ | ✅ | 🔴 v2 Only | Persistent settings |
| **Activity Logging** |
| `get_recent_activities` | ❌ | ✅ | 🔴 v2 Only | Activity log system |
| `get_activity_stats` | ❌ | ✅ | 🔴 v2 Only | Activity statistics |
| `clear_activities` | ❌ | ✅ | 🔴 v2 Only | - |
| **Notifications** |
| `get_notifications` | ❌ | ✅ | 🔴 v2 Only | Notification system |
| `get_unread_notification_count` | ❌ | ✅ | 🔴 v2 Only | - |
| `mark_notification_as_read` | ❌ | ✅ | 🔴 v2 Only | - |
| `mark_all_notifications_as_read` | ❌ | ✅ | 🔴 v2 Only | - |
| `dismiss_notification` | ❌ | ✅ | 🔴 v2 Only | - |
| `clear_all_notifications` | ❌ | ✅ | 🔴 v2 Only | - |
| `create_notification` | ❌ | ✅ | 🔴 v2 Only | - |
| **Batch Job Management** |
| `get_batch_jobs` | ❌ | ✅ | 🔴 v2 Only | Complete batch system |
| `get_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `create_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `start_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `pause_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `cancel_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `delete_batch_job` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_batch_job_count` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_active_batch_job_count` | ❌ | ✅ | 🔴 v2 Only | - |
| **Security Management** |
| `create_api_key` | ❌ | ✅ | 🔴 v2 Only | Complete security system |
| `get_api_keys` | ❌ | ✅ | 🔴 v2 Only | - |
| `revoke_api_key` | ❌ | ✅ | 🔴 v2 Only | - |
| `delete_api_key` | ❌ | ✅ | 🔴 v2 Only | - |
| `validate_api_key` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_security_events` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_security_metrics` | ❌ | ✅ | 🔴 v2 Only | - |
| `clear_security_events` | ❌ | ✅ | 🔴 v2 Only | - |
| **Model Repository (HuggingFace)** |
| `search_external_models` | ❌ | ✅ | 🔴 v2 Only | Model discovery |
| `get_external_model_details` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_featured_models` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_trending_models` | ❌ | ✅ | 🔴 v2 Only | - |
| `start_model_download` | ❌ | ✅ | 🔴 v2 Only | Download manager |
| `get_download_progress` | ❌ | ✅ | 🔴 v2 Only | - |
| `get_all_downloads` | ❌ | ✅ | 🔴 v2 Only | - |
| `cancel_download` | ❌ | ✅ | 🔴 v2 Only | - |
| **macOS Integration** |
| `send_native_notification` | ✅ | ❌ | 🟡 v1 Only | Needs migration |
| `get_system_appearance` | ✅ | ❌ | 🟡 v1 Only | Needs migration |
| `set_window_vibrancy` | ✅ | ❌ | 🟡 v1 Only | Needs migration |
| `toggle_always_on_top` | ✅ | ❌ | 🟡 v1 Only | Needs migration |
| `minimize_to_tray` | ✅ | ❌ | 🟡 v1 Only | Needs migration |

**Total Commands**:
- Tauri v1: **9 core** + **5 macOS** = **14 commands**
- Tauri v2: **51 commands** (comprehensive)
- v2 Exclusive: **42 commands**
- v1 Exclusive: **5 macOS integration commands**

---

## 🏗️ Architecture Comparison

### Tauri v1 (src/tauri_app.rs)
**Lines of Code**: 256
**Dependencies**: Basic Inferno crate imports
**State Management**: Simple `AppState` with Mutex wrappers

```rust
pub struct AppState {
    backends: Mutex<HashMap<String, Backend>>,
    model_manager: Mutex<ModelManager>,
    metrics: Mutex<MetricsCollector>,
    config: Mutex<Config>,
}
```

**Strengths**:
- ✅ macOS-specific integration functions
- ✅ Native menu bar and system tray
- ✅ Window vibrancy and appearance detection

**Weaknesses**:
- ❌ No batch job system
- ❌ No security/API key management
- ❌ No notification system
- ❌ No activity logging
- ❌ No settings persistence
- ❌ No model repository integration
- ❌ No streaming inference
- ❌ Uses deprecated Tauri v1 APIs

### Tauri v2 (dashboard/src-tauri/src/main.rs)
**Lines of Code**: 1,305 (main.rs alone)
**Additional Modules**:
- `backend_manager.rs` (11,597 lines)
- `database.rs` (32,101 lines)
- `security.rs` (13,080 lines)
- `events.rs` (12,424 lines)
- `model_repository.rs` (16,918 lines)
- `activity_logger.rs` (7,779 lines)

**State Management**: Comprehensive AppState with Arc wrappers

```rust
pub struct AppState {
    pub system: Arc<Mutex<System>>,
    pub backend_manager: Arc<BackendManager>,
    pub metrics: Arc<Mutex<MetricsSnapshot>>,
    pub activity_logger: Arc<ActivityLogger>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub notifications: Arc<Mutex<Vec<Notification>>>,
    pub batch_jobs: Arc<Mutex<Vec<BatchJob>>>,
    pub security_manager: Arc<SecurityManager>,
    pub event_manager: Arc<Mutex<Option<EventManager>>>,
    pub database: Arc<DatabaseManager>,
    pub model_repository: Arc<ModelRepositoryService>,
    pub download_manager: Arc<ModelDownloadManager>,
}
```

**Strengths**:
- ✅ Complete feature parity with web dashboard
- ✅ SQLite database for persistence
- ✅ Comprehensive security system
- ✅ Event emission system
- ✅ Model repository (HuggingFace) integration
- ✅ Activity logging and auditing
- ✅ Batch job management
- ✅ Settings persistence
- ✅ Notification system
- ✅ Streaming inference
- ✅ Uses modern Tauri v2 APIs

**Weaknesses**:
- ❌ Missing macOS-specific integrations
- ❌ No system tray menu (basic implementation only)
- ❌ No window vibrancy
- ❌ No appearance detection

---

## 🎯 Migration Strategy

### Phase 1: Extract Tauri v1 macOS Features
**Files to migrate from**:
- `src/macos_integration.rs` (336 lines)
- `src/tauri_app.rs` (macOS command handlers)

**Key functions to preserve**:
```rust
// System tray with custom menu
create_system_tray() -> SystemTray
handle_system_tray_event(event)

// Native macOS menu
create_app_menu() -> Menu
handle_menu_event(window, menu_item_id)

// macOS-specific commands
send_native_notification(notification: MacOSNotification)
get_system_appearance() -> String // "light" | "dark"
set_window_vibrancy(window: Window, effect: String)
toggle_always_on_top(window: Window)
minimize_to_tray(window: Window)
```

### Phase 2: Update to Tauri v2 APIs
**API Changes Required**:
1. `tauri::api::notification::Notification` → `tauri-plugin-notification`
2. `SystemTray` → `TrayIcon` (different API)
3. `Menu` / `MenuItem` → New menu API
4. `Window` methods updated

### Phase 3: Integrate into Tauri v2 Codebase
**Target**: `dashboard/src-tauri/src/main.rs` + new module

**New file**: `dashboard/src-tauri/src/macos.rs`
- Port all macOS-specific functions
- Update to Tauri v2 APIs
- Integrate with existing AppState
- Add to invoke_handler

---

## 🔧 Technical Debt to Address

### 1. Deprecated Tauri v1 APIs
```rust
// OLD (Tauri v1)
use tauri::{CustomMenuItem, Menu, SystemTray, SystemTrayMenu};

// NEW (Tauri v2)
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder};
```

### 2. Plugin Migration
```rust
// OLD (Tauri v1)
FileDialogBuilder::new().pick_file().await

// NEW (Tauri v2)
app.dialog()
    .file()
    .add_filter("Model Files", &["gguf"])
    .blocking_pick_file()
```

### 3. State Management
- v1 uses `State<'_, AppState>`
- v2 uses `State<'_, AppState>` (same, but AppState is different)
- Need to ensure all v1 commands work with v2 AppState

---

## ✅ Action Items

### Immediate (Phase 1.2)
- [x] Create this audit document
- [ ] Create `dashboard/src-tauri/src/macos.rs`
- [ ] Port macOS integration functions to Tauri v2 APIs
- [ ] Update system tray implementation
- [ ] Update menu bar implementation

### Short-term (Phase 1.3)
- [ ] Test all migrated macOS features
- [ ] Add macOS commands to invoke_handler
- [ ] Update AppState if needed
- [ ] Remove old Tauri v1 files

### Long-term (Phase 1.4+)
- [ ] Delete `src/tauri_app.rs`
- [ ] Archive `src/macos_integration.rs`
- [ ] Update Cargo.toml dependencies
- [ ] Update build configuration

---

## 🎉 Conclusion

**Decision**: Tauri v2 implementation is significantly more complete and should be the primary target.

**Migration Path**:
1. Port 5 macOS-specific functions from v1 to v2
2. Update APIs for Tauri v2 compatibility
3. Integrate into existing v2 codebase
4. Delete Tauri v1 implementation

**Estimated Effort**: 1-2 days for macOS integration migration

**Risk**: Low - v2 is already fully functional, just missing macOS polish