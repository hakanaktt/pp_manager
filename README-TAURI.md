# Process Priority Manager (Tauri Edition)

A modern Windows application built with Tauri, Vue.js, and TypeScript for managing process priorities and CPU affinity settings. This tool allows you to monitor specific processes and automatically apply custom CPU core assignments and priority levels.

## Features

- **Modern UI**: Built with Vue.js and TypeScript for a responsive, modern interface
- **Process Monitoring**: Continuously monitor target processes and apply settings when they start
- **CPU Affinity Control**: Select specific CPU cores for process execution
- **Priority Management**: Set process priority levels (Idle, Below Normal, Normal, Above Normal, High, Realtime)
- **Quick Presets**: Pre-configured settings for common scenarios (Performance Cores, Efficiency Cores, etc.)
- **System Tray Integration**: Minimize to system tray and control via tray menu
- **Real-time Logging**: Monitor all process management activities with detailed logs
- **Administrator Detection**: Automatic detection of administrator privileges with recommendations
- **Cross-platform Ready**: Built with Tauri for potential future cross-platform support

## Technology Stack

- **Backend**: Rust with Tauri framework
- **Frontend**: Vue.js 3 with TypeScript
- **Build System**: Vite
- **UI**: Custom CSS with modern design principles

## Requirements

- Windows 10/11
- Administrator privileges (recommended for full functionality)
- Node.js 18+ (for development)
- Rust toolchain (for building from source)

## Installation

### Pre-built Releases
Download the latest release from the [Releases](https://github.com/your-repo/pp-manager/releases) page.

### Building from Source

1. **Install Prerequisites**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Node.js (18+)
   # Download from https://nodejs.org/
   ```

2. **Clone and Build**:
   ```bash
   git clone https://github.com/your-repo/pp-manager.git
   cd pp-manager
   
   # Install frontend dependencies
   npm install
   
   # Build the application
   npm run tauri build
   ```

3. **Development Mode**:
   ```bash
   # Run in development mode
   npm run tauri dev
   ```

## Usage

1. **Run as Administrator** (recommended) for full access to system processes
2. Configure your target process name (e.g., "icad.exe", "notepad.exe")
3. Select desired CPU cores using the checkboxes
4. Choose a priority level from the dropdown
5. Click "Start Monitoring" to begin automatic process management
6. The application will minimize to the system tray and continue monitoring

### Quick Presets

- **Performance Cores**: Selects middle-high performance cores (ideal for hybrid CPUs)
- **Efficiency Cores**: Selects efficiency cores (lower power consumption)
- **High Performance**: Selects highest performance cores
- **All Cores**: Uses all available CPU cores
- **Maximum Performance**: All cores with high priority
- **Half Cores**: Uses half of available cores (balanced approach)

### System Tray

- Left-click the tray icon to show/hide the main window
- Right-click for context menu options
- Use Ctrl+Shift+P keyboard shortcut to toggle window visibility

## Configuration

Settings are automatically saved to `config.toml` in the application directory. The configuration includes:

- Target process name
- CPU core selections
- Priority class setting

## API Commands

The application exposes the following Tauri commands:

- `get_system_info()`: Retrieve system information (CPU count, admin status)
- `get_config()`: Load current configuration
- `save_config_cmd(config)`: Save configuration
- `start_monitoring()`: Begin process monitoring
- `stop_monitoring()`: Stop process monitoring
- `get_monitoring_status()`: Check if monitoring is active
- `get_logs()`: Retrieve activity logs
- `clear_logs()`: Clear all logs

## Development

### Project Structure

```
pp-manager/
├── src/                    # Vue.js frontend source
│   ├── App.vue            # Main application component
│   ├── main.ts            # Frontend entry point
│   └── style.css          # Global styles
├── src-tauri/             # Rust backend source
│   ├── src/
│   │   ├── main.rs        # Tauri application entry
│   │   └── process_manager.rs  # Process management logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Node.js dependencies
├── vite.config.ts         # Vite configuration
└── tsconfig.json          # TypeScript configuration
```

### Available Scripts

```bash
# Development
npm run dev          # Start Vite dev server
npm run tauri dev    # Start Tauri in development mode

# Building
npm run build        # Build frontend for production
npm run tauri build  # Build complete Tauri application

# Type checking
npm run vue-tsc      # Run TypeScript compiler
```

## Logging

The application provides real-time logging of all process management activities:

- **INFO**: General status updates
- **SUCCESS**: Process found and settings applied
- **REAPPLY**: Settings reapplied due to changes
- **MONITOR**: Continuous monitoring status
- **ERROR**: Error conditions and troubleshooting information

## Troubleshooting

### "Access Denied" Errors
- Run the application as Administrator
- Some system processes are protected and cannot be modified

### Process Not Found
- Verify the exact process name (including .exe extension)
- Check that the target process is actually running
- Process names are case-insensitive

### Settings Not Applied
- Ensure you have Administrator privileges
- Some processes may reset their own priority/affinity
- Check the logs for detailed error information

### Build Issues
- Ensure all prerequisites are installed
- Clear node_modules and reinstall: `rm -rf node_modules && npm install`
- Clear Rust target directory: `cargo clean`

## Technical Details

### Backend (Rust/Tauri)
- Uses Windows API for process management
- Implements secure command handlers for frontend communication
- Manages system tray integration and window lifecycle

### Frontend (Vue.js/TypeScript)
- Reactive UI with real-time updates
- Type-safe communication with backend
- Modern, responsive design

### Process Management
- Enumerates running processes using Windows API
- Opens process handles with appropriate permissions
- Sets process priority class using `SetPriorityClass`
- Sets CPU affinity using `SetProcessAffinityMask`
- Monitors for process changes and reapplies settings as needed

## Migration from egui Version

This Tauri version maintains full feature parity with the original egui-based application while providing:

- Modern web-based UI with better responsiveness
- Improved maintainability with separation of concerns
- Better cross-platform potential
- Enhanced developer experience with hot reload
- Type-safe frontend-backend communication

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow TypeScript best practices for frontend code
2. Use Rust conventions for backend code
3. Maintain type safety across the frontend-backend boundary
4. Test on Windows with and without administrator privileges
5. Update documentation for any API changes
