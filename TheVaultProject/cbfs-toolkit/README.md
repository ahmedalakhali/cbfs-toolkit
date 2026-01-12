# Vault Console

A professional command-line vault management application integrated with the CBFS Vault SDK.

## Features

- **Secure Storage**: Uses Argon2id password hashing for user credentials.
- **RBAC**: Role-Based Access Control for managing user permissions (Admin, CreateUser, CreateVault, CloudWrite).
- **FUSE Integration**: Mount vaults as virtual drives (standard and Google Drive integration).
- **File Operations**: Add, list, extra, and delete files with optional compression and encryption.
- **SQLite Support**: Operate on SQLite databases as vaults using the CBFS Vault callback mode.

## Setup

1.  **Prerequisites**:
    *   Rust toolchain (stable).
    *   CBFS Connect / CBFS Vault drivers installed (for mounting).

2.  **Build**:
    ```bash
    cargo build --release
    ```

3.  **Run**:
    ```bash
    cargo run -- -i
    ```
    This launches the interactive shell.

## Usage

### Interactive Mode
Launch with `-i` or `--interactive`.

**First Run**:
On the first run, if no `users.db` exists, you will be prompted to create an **Admin** account.

**Commands**:
*   `open <path>`: Open a vault.
*   `create <path>`: Create a new vault.
*   `add <files>`: Add files to the open vault.
*   `list`: List files.
*   `mount <drive_letter>`: Mount the vault as a local drive.
*   `info`: Show session info.

### Command Line Flags
*   `-c`: Create new vault.
*   `-pw <password>`: Specify password.
*   `-m <mount_point>`: Mount immediately.
*   `-a <files>`: Add files (batch mode).

## Architecture

*   **`users.rs`**: User management and authentication (SQLite + Argon2).
*   **`file_ops.rs`**: Core vault file operations.
*   **`fuse_drive.rs` / `fuse_gdrive.rs`**: FUSE implementation for virtual drives.
*   **`sqlite_vault.rs`**: implementation of CBFS Vault callback API for SQLite.

# Presentation Layer: This layer exists to ensure that changing the user interface from a Command Line to a Web Page does not require rewriting the entire application.

# Domain Layer: This layer exists to define what your business does (rules and data structures) completely independently of how it is stored or accessed, so your business logic survives technology shifts.

# Infrastructure Layer: This layer exists to isolate technical details like databases and file systems so that switching vendors or libraries does not break your business rules.
