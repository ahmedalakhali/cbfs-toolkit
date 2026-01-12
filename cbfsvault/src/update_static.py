import re
import os

file_path = r"C:\Users\Ahmed\Desktop\RustVualtProject\cbfsvault\src\cbvaultdrive.rs"

if not os.path.exists(file_path):
    print(f"File not found: {file_path}")
    exit(1)

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Replace function pointers
# Pattern: CBFSVault_CBVaultDrive_Do.clone().unwrap()
# Replacement: CBFSVault_CBVaultDrive_Do.get().unwrap()
content = re.sub(r'(CBFSVault_CBVaultDrive_\w+)\.clone\(\)\.unwrap\(\)', r'\1.get().unwrap()', content)

# 2. Remove CBVaultDriveDictMutex usage
# Pattern: let _map = CBVaultDriveDictMutex.lock().unwrap();
content = re.sub(r'(?m)^\s*let _map = CBVaultDriveDictMutex\.lock\(\)\.unwrap\(\);.*\n?', '', content)

# 3. Update CBVaultDriveDict usage
# Pattern: CBVaultDriveDict.method()
# Replacement: CBVaultDriveDict.lock().unwrap().method()
# Avoid matching "static CBVaultDriveDict :"
content = re.sub(r'(?<!static )(?<!static mut )CBVaultDriveDict\.', r'CBVaultDriveDict.lock().unwrap().', content)

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)

print("Successfully updated cbvaultdrive.rs")
