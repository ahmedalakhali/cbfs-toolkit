# CBFS Toolkit

> **A professional CLI that redefines filesystem interactions by making advanced security transparent and effortless. Delivers enterprise-grade encryption and virtual storage solutions that scale from individual needs to complex infrastructure.**

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-edition%202021-orange.svg)
![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey.svg)

**CBFS Toolkit** is a powerful command-line application designed to demonstrate the capabilities of the CBFS Vault and CBFS Connect SDKs. It provides a robust solution for creating encrypted file vaults, mounting them as virtual drives, and integrating with cloud storage providers like Google Drive—all secured by enterprise-grade authentication and access control.

---

## 🚀 Features

### 🛡️ Secure Storage
- **Secure Storage**: Uses Vaults files to store users data in a compromised and encrupted container.
- **File Operations**: Add, list, extra, and delete files with optional compression and encryption.
- **SQLite Support**: Operate on SQLite databases as vaults using the CBFS Vault callback mode.

### ☁️ Cloud & Virtual Drive Integration
- **FUSE Integration**: Mount vaults as local virtual drives (e.g., `Z:` drive).
- **Google Drive Integration**: Map your Google Drive as a local virtual disk with on-the-fly caching.
- **Smart Caching**: Minimized network calls for optimal performance.

### 🔐 Security & Access Control
- **RBAC System**: Role-Based Access Control (Admin, CreateUser, CloudWrite, etc.).
- **Argon2 Hashing**: Industry-standard password hashing for user credentials.
- **Credential Manager**: Secure storage of OAuth tokens using Windows Credential Manager.

---

## 🛠️ Installation

### Prerequisites
- **Rust Toolchain**: Stable release (install via `rustup`).
- **CBFS Drivers**: 
  - [CBFS Connect](https://www.callback.com/cbfsconnect/)
  - [CBFS Vault](https://www.callback.com/cbfsvault/)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/ahmedalakhali/cbfs-toolkit.git
cd cbfs-toolkit

# Build for release
cargo build --release
```

---

## 📖 Usage

Run the toolkit in interactive mode for the best experience:

```bash
./target/release/cbfs-toolkit -i
```

### Common Commands

| Command | Description |
|---------|-------------|
| `open <path>` | Open an existing vault file |
| `create <path>` | Create a new encrypted vault |
| `mount <point>` | Mount the current vault as a virtual drive |
| `add <files>` | Add files to the vault |
| `google-drive` | Mount Google Drive (Requires `CloudWrite` or Admin) |
| `users` | Manage users (Admin only) |

### Command Line Flags

- `-i`, `--interactive`: Start interactive shell
- `-c`: Create new vault
- `-pw <password>`: Specify vault password
- `-m <mount_point>`: Mount immediately on startup

---

## 🏗️ Architecture

The project follows Clean Architecture principles:

- **Domain Layer**: Core business logic and interfaces (`src/domain`)
- **Infrastructure Layer**: Concrete implementations for specific technologies (`src/infrastructure`)
- **Presentation Layer**: CLI and Interactive Shell (`src/interactive.rs`)

### Key Components

- **VaultService**: Abstract interface for vault operations.
- **AuthService**: Handles user authentication and session management.
- **FuseDrive**: Implements the virtual filesystem logic using CBFS Connect.
- **GoogleDriveFS**: Specialized FUSE implementation for Google Drive API.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.
