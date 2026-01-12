$path = "cbvaultdrive.rs"
if (!(Test-Path $path)) { Write-Host "File not found"; exit 1 }
$c = Get-Content $path -Raw

$c = $c.Replace("CBFSVault_CBVaultDrive_StaticInit.clone().unwrap()", "CBFSVault_CBVaultDrive_StaticInit.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_StaticDestroy.clone().unwrap()", "CBFSVault_CBVaultDrive_StaticDestroy.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_Create.clone().unwrap()", "CBFSVault_CBVaultDrive_Create.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_Destroy.clone().unwrap()", "CBFSVault_CBVaultDrive_Destroy.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_Set.clone().unwrap()", "CBFSVault_CBVaultDrive_Set.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_SetCStr.clone().unwrap()", "CBFSVault_CBVaultDrive_SetCStr.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_SetInt.clone().unwrap()", "CBFSVault_CBVaultDrive_SetInt.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_SetInt64.clone().unwrap()", "CBFSVault_CBVaultDrive_SetInt64.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_Get.clone().unwrap()", "CBFSVault_CBVaultDrive_Get.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetAsCStr.clone().unwrap()", "CBFSVault_CBVaultDrive_GetAsCStr.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetAsInt.clone().unwrap()", "CBFSVault_CBVaultDrive_GetAsInt.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetAsInt64.clone().unwrap()", "CBFSVault_CBVaultDrive_GetAsInt64.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetAsBSTR.clone().unwrap()", "CBFSVault_CBVaultDrive_GetAsBSTR.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetLastError.clone().unwrap()", "CBFSVault_CBVaultDrive_GetLastError.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetLastErrorCode.clone().unwrap()", "CBFSVault_CBVaultDrive_GetLastErrorCode.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_SetLastErrorAndCode.clone().unwrap()", "CBFSVault_CBVaultDrive_SetLastErrorAndCode.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetEventError.clone().unwrap()", "CBFSVault_CBVaultDrive_GetEventError.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_GetEventErrorCode.clone().unwrap()", "CBFSVault_CBVaultDrive_GetEventErrorCode.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_SetEventErrorAndCode.clone().unwrap()", "CBFSVault_CBVaultDrive_SetEventErrorAndCode.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_CheckIndex.clone().unwrap()", "CBFSVault_CBVaultDrive_CheckIndex.get().unwrap()")
$c = $c.Replace("CBFSVault_CBVaultDrive_Do.clone().unwrap()", "CBFSVault_CBVaultDrive_Do.get().unwrap()")

$c = $c -replace '(?m)^\s*let _map = CBVaultDriveDictMutex\.lock\(\)\.unwrap\(\);.*$', ''

# For Dict, we need to be careful not to replace the definition line.
# Definition: "static CBVaultDriveDict : Lazy<Mutex<HashMap<usize, CBVaultDrive>>> ="
# Replace "CBVaultDriveDict." with "CBVaultDriveDict.lock().unwrap()."
# Using regex to exclude static definition
$c = $c -replace '(?<!static )CBVaultDriveDict\.', 'CBVaultDriveDict.lock().unwrap().'

Set-Content $path $c -NoNewline
Write-Host "Done"
