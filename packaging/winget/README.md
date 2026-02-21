# WinGet packaging notes

Target package identifier:

`PaternosterRack.Pater`

Publish flow:

1. Build and publish a tagged release (zip with `pater.exe`).
2. Compute SHA256 for installer/zip.
3. Create winget manifest PR in `microsoft/winget-pkgs` using this identifier.
4. After merge, users can run:

```powershell
winget install PaternosterRack.Pater
```
