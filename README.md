# BufferVault

A lightweight and secure clipboard history manager for Windows 10/11 that automatically intercepts every copy operation and keeps an encrypted history of all copied items. Users can recall this history at any time, browse through the entries, and select items to paste wherever they want.

## Features

- 🔒 **Encrypted Storage**: All clipboard data is encrypted using AES-256 encryption
- 📋 **Automatic Monitoring**: Automatically captures all copy operations
- 🔍 **Search Functionality**: Quickly find items in your clipboard history
- 🖥️ **User-Friendly GUI**: Easy-to-use interface for browsing and selecting items
- 💾 **Persistent Storage**: History is saved and available across sessions
- 🔔 **System Tray Integration**: Runs minimized in the system tray
- ⚡ **Lightweight**: Minimal resource usage
- 🎯 **Quick Access**: Double-click any item to copy it back to clipboard

## Installation

### Prerequisites

- Windows 10 or Windows 11
- Python 3.7 or higher

### Setup

1. Clone the repository:
```bash
git clone https://github.com/odaiko42/bufferVault.git
cd bufferVault
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

## Usage

### GUI Mode (Recommended)

Start BufferVault with the graphical interface:

```bash
python main.py --mode gui
```

Or simply:

```bash
python main.py
```

#### Using the GUI:

1. **Automatic Monitoring**: Once launched, BufferVault automatically monitors your clipboard
2. **View History**: All copied items appear in the main window
3. **Search**: Type in the search box to filter items
4. **Paste Items**: 
   - Double-click any item to copy it to clipboard
   - Or select an item and click "Paste Selected"
5. **View Full Content**: Click "View Full Content" to see the complete text of an item
6. **System Tray**: Close the window to minimize to system tray (monitoring continues)

### CLI Mode

Run BufferVault as a background service:

```bash
python main.py --mode cli
```

### View History

Display recent clipboard history in terminal:

```bash
python main.py --mode history
```

## Configuration

BufferVault creates a `config.json` file on first run with the following default settings:

```json
{
    "max_history_items": 1000,
    "auto_start": false,
    "storage_path": "clipboard_data",
    "encryption_enabled": true,
    "hotkey": "Ctrl+Shift+V",
    "max_item_size_mb": 10
}
```

### Configuration Options:

- **max_history_items**: Maximum number of items to keep in history
- **auto_start**: Start with Windows (not yet implemented)
- **storage_path**: Directory where clipboard data is stored
- **encryption_enabled**: Enable/disable encryption (recommended: true)
- **hotkey**: Keyboard shortcut to open BufferVault (not yet implemented)
- **max_item_size_mb**: Maximum size for a single clipboard item (in MB)

## Security

- All clipboard data is encrypted using **AES-256** encryption via the `cryptography` library (v42.0.4+)
- Encryption keys are derived using **PBKDF2HMAC** with 100,000 iterations
- Data is stored in the `clipboard_data` directory with `.vault` extension
- A unique salt is generated for each installation
- **Security Updates**: Uses cryptography 42.0.4 to address CVE vulnerabilities (NULL pointer dereference and Bleichenbacher timing oracle)

### Security Note:

The default implementation uses a machine-specific identifier for encryption. For enhanced security, you can modify the `encryption.py` file to use a custom password.

## File Structure

```
bufferVault/
├── main.py                 # Main entry point
├── gui.py                  # GUI implementation
├── clipboard_monitor.py    # Clipboard monitoring service
├── storage.py             # Storage and history management
├── encryption.py          # Encryption/decryption module
├── config.py              # Configuration management
├── requirements.txt       # Python dependencies
├── README.md             # This file
└── clipboard_data/       # Encrypted clipboard storage (created on first run)
```

## Dependencies

- **pyperclip**: Cross-platform clipboard access
- **cryptography**: Encryption/decryption functionality
- **PyQt5**: GUI framework

## Troubleshooting

### Common Issues:

1. **Clipboard not being monitored**:
   - Ensure BufferVault is running
   - Check that no other clipboard manager is interfering

2. **Items not appearing in history**:
   - Check that the copied text is not empty
   - Verify the item size is within the configured limit

3. **GUI not starting**:
   - Ensure PyQt5 is properly installed
   - Try reinstalling dependencies: `pip install -r requirements.txt --force-reinstall`

## Platform Support

Currently, BufferVault is designed for **Windows 10/11**. The core functionality (clipboard monitoring, encryption, storage) is cross-platform compatible, but the GUI and system integration are optimized for Windows.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

This project is open source. Please check the repository for license information.

## Roadmap

Future enhancements planned:
- [ ] Hotkey support for quick access
- [ ] Auto-start with Windows
- [ ] Image clipboard support
- [ ] File path clipboard support
- [ ] Export/import history
- [ ] Custom encryption password
- [ ] Cloud sync (optional)
- [ ] Dark mode theme

## Version

Current version: **1.0.0**

---

**Note**: Always ensure you have the latest version for the best experience and security updates.