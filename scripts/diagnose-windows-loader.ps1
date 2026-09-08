param(
    [Parameter(Mandatory = $true)]
    [string]$Executable
)

# Read PE imports and ask the loader whether each exported entry point exists.
# The target executable is never started; no application data is inspected.
$ErrorActionPreference = 'Stop'
$taskExecutablePath = (Resolve-Path -LiteralPath $Executable).Path
$taskPeBytes = [System.IO.File]::ReadAllBytes($taskExecutablePath)
$taskPeOffset = [BitConverter]::ToInt32($taskPeBytes, 0x3c)
if ([BitConverter]::ToUInt32($taskPeBytes, $taskPeOffset) -ne 0x00004550) {
    throw 'Target is not a PE executable'
}
$taskSectionCount = [BitConverter]::ToUInt16($taskPeBytes, $taskPeOffset + 6)
$taskOptionalSize = [BitConverter]::ToUInt16($taskPeBytes, $taskPeOffset + 20)
$taskOptionalOffset = $taskPeOffset + 24
$taskIs64 = [BitConverter]::ToUInt16($taskPeBytes, $taskOptionalOffset) -eq 0x20b
$taskDirectoryOffset = $taskOptionalOffset + $(if ($taskIs64) { 112 } else { 96 })
$taskImportRva = [BitConverter]::ToUInt32($taskPeBytes, $taskDirectoryOffset + 8)
$taskSectionsOffset = $taskOptionalOffset + $taskOptionalSize

function Convert-TaskRvaToOffset([uint32]$Rva) {
    for ($taskSectionIndex = 0; $taskSectionIndex -lt $taskSectionCount; $taskSectionIndex++) {
        $taskSectionOffset = $taskSectionsOffset + $taskSectionIndex * 40
        $taskVirtualSize = [BitConverter]::ToUInt32($taskPeBytes, $taskSectionOffset + 8)
        $taskVirtualAddress = [BitConverter]::ToUInt32($taskPeBytes, $taskSectionOffset + 12)
        $taskRawSize = [BitConverter]::ToUInt32($taskPeBytes, $taskSectionOffset + 16)
        $taskRawPointer = [BitConverter]::ToUInt32($taskPeBytes, $taskSectionOffset + 20)
        $taskSpan = [Math]::Max($taskVirtualSize, $taskRawSize)
        if ($Rva -ge $taskVirtualAddress -and $Rva -lt $taskVirtualAddress + $taskSpan) {
            return [int]($taskRawPointer + $Rva - $taskVirtualAddress)
        }
    }
    throw "Unmapped PE RVA $Rva"
}

function Read-TaskAsciiZ([int]$Offset) {
    $taskEndOffset = $Offset
    while ($taskEndOffset -lt $taskPeBytes.Length -and $taskPeBytes[$taskEndOffset] -ne 0) {
        $taskEndOffset++
    }
    return [Text.Encoding]::ASCII.GetString($taskPeBytes, $Offset, $taskEndOffset - $Offset)
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class DynamLoaderProbe {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryExW(string path, IntPtr file, uint flags);
    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);
    [DllImport("kernel32.dll", EntryPoint = "GetProcAddress", ExactSpelling = true, SetLastError = true)]
    public static extern IntPtr GetProcAddressOrdinal(IntPtr module, IntPtr ordinal);
    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool FreeLibrary(IntPtr module);
}
'@

Write-Output "Windows loader diagnosis: $([Environment]::OSVersion.VersionString)"
Write-Output "Executable: $([IO.Path]::GetFileName($taskExecutablePath))"
if ($taskImportRva -eq 0) {
    Write-Output 'No regular PE import directory.'
    exit 0
}
$taskDescriptorOffset = Convert-TaskRvaToOffset $taskImportRva
$taskMissingCount = 0
$taskSymbolCount = 0
$taskModuleCount = 0
while ($true) {
    $taskOriginalThunk = [BitConverter]::ToUInt32($taskPeBytes, $taskDescriptorOffset)
    $taskNameRva = [BitConverter]::ToUInt32($taskPeBytes, $taskDescriptorOffset + 12)
    $taskFirstThunk = [BitConverter]::ToUInt32($taskPeBytes, $taskDescriptorOffset + 16)
    if ($taskNameRva -eq 0) { break }
    $taskDllName = Read-TaskAsciiZ (Convert-TaskRvaToOffset $taskNameRva)
    $taskAppLocalDll = Join-Path ([IO.Path]::GetDirectoryName($taskExecutablePath)) $taskDllName
    $taskDllPath = if (Test-Path -LiteralPath $taskAppLocalDll) { $taskAppLocalDll } else { $taskDllName }
    # DONT_RESOLVE_DLL_REFERENCES maps exports without running the DLL entry point.
    $taskModule = [DynamLoaderProbe]::LoadLibraryExW($taskDllPath, [IntPtr]::Zero, 1)
    $taskModuleCount++
    if ($taskModule -eq [IntPtr]::Zero) {
        Write-Output "UNRESOLVED DLL: $taskDllName (Win32 error $([Runtime.InteropServices.Marshal]::GetLastWin32Error()))"
        $taskMissingCount++
    } else {
        try {
            $taskThunkRva = if ($taskOriginalThunk -ne 0) { $taskOriginalThunk } else { $taskFirstThunk }
            $taskThunkOffset = Convert-TaskRvaToOffset $taskThunkRva
            $taskStride = if ($taskIs64) { 8 } else { 4 }
            while ($true) {
                $taskThunk = if ($taskIs64) {
                    [BitConverter]::ToUInt64($taskPeBytes, $taskThunkOffset)
                } else {
                    [uint64][BitConverter]::ToUInt32($taskPeBytes, $taskThunkOffset)
                }
                if ($taskThunk -eq 0) { break }
                $taskIsOrdinal = if ($taskIs64) {
                    ($taskThunk -band [uint64]::Parse('8000000000000000', [Globalization.NumberStyles]::HexNumber)) -ne 0
                } else { ($taskThunk -band 0x80000000L) -ne 0 }
                if ($taskIsOrdinal) {
                    $taskOrdinal = [int]($taskThunk -band 0xffff)
                    $taskSymbol = "ordinal#$taskOrdinal"
                    $taskAddress = [DynamLoaderProbe]::GetProcAddressOrdinal($taskModule, [IntPtr]::new($taskOrdinal))
                } else {
                    $taskSymbol = Read-TaskAsciiZ ((Convert-TaskRvaToOffset ([uint32]$taskThunk)) + 2)
                    $taskAddress = [DynamLoaderProbe]::GetProcAddress($taskModule, $taskSymbol)
                }
                $taskSymbolCount++
                if ($taskAddress -eq [IntPtr]::Zero) {
                    Write-Output "MISSING ENTRYPOINT: $taskDllName!$taskSymbol"
                    $taskMissingCount++
                }
                $taskThunkOffset += $taskStride
            }
        } finally {
            [DynamLoaderProbe]::FreeLibrary($taskModule) | Out-Null
        }
    }
    $taskDescriptorOffset += 20
}
Write-Output "Checked $taskSymbolCount direct imported symbols in $taskModuleCount DLLs; unresolved: $taskMissingCount"
Write-Output 'This checks direct regular imports. A missing transitive or delay-loaded import still needs loader diagnostics.'
Write-Output 'DLL selection uses this diagnostic process activation context; a target manifest can select a different side-by-side DLL version.'
