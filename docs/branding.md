# SkyPDF naming and upgrade compatibility

SkyPDF was named MyPDF through version 3.1.1. Version 3.1.2 uses SkyPDF in the desktop interface, About, native menus, Windows product resources and installers, Finder bundle and downloadable packages. The Microsoft Store display name is **SkyPDF (formerly MyPDF)**.

These identifiers intentionally remain unchanged:

- GitHub repository `gokcol/MyPDF` and existing release URLs, so installed update clients do not encounter redirects or lose their download source.
- Mac bundle ID `dev.mypdf.desktop`, executable `MyPDF` and existing signing identities/Keychain accounts.
- Windows installer AppId, existing installation directory, executable `MyPDF.exe`, file-association keys and registered capabilities.
- Microsoft Store product `9MXR0WLW42XT`, Partner Center package identity/publisher, application ID `MyPDF` and `mypdf-og.exe` launch alias.
- App-data folders under MyPDF, `MYPDF_*` environment overrides, IPC and signing storage identifiers.
- PDF metadata/resource keys and annotation IDs beginning with MyPDF. Existing annotations, tables, OCR, watermarks and thread metadata must remain editable.
- The `.mypdf-review` extension and its serialized format identifier, so saved review exchanges work in both versions.
- Rust crate/binary targets, native library ABI names and icon source filenames. These are implementation names, not the displayed product name.

New release files use `SkyPDF-<version>`. Windows portable ZIPs contain a `SkyPDF` folder with the compatible `MyPDF.exe` executable. Mac installs use `SkyPDF.app`. The Store rename is published through an update to the same product; no replacement product or new user-data migration is required.
