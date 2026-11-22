# Ashina naming and upgrade compatibility

Ashina was named MyPDF through version 3.1.1 and SkyPDF in version 3.1.2. Version 3.1.3 uses **Ashina** in the desktop interface, About, native menus, Windows product resources and installers, Finder bundle, downloadable packages and Microsoft Store listing.

Microsoft rejected the 3.1.2 Store title under policy 10.1.1.1 (Inaccurate Representation) on November 17, 2026. The replacement Store product name is exactly **Ashina**, reserved on the same product. Do not append former app names to the Store product title. Historical names can be explained in documentation.

The GitHub repository is `gokcol/Ashina`. Keep the former `gokcol/MyPDF` path available as a GitHub redirect so existing Mac update feeds and download links continue to work. GitHub builds through 3.1.5 reject the release API redirect during their in-app update check, so users need to download the next build manually from GitHub Releases. Microsoft Store installs continue to use the same product ID.

These identifiers intentionally remain unchanged:

- Mac bundle ID `dev.mypdf.desktop` and existing signing identities and Keychain accounts.
- Windows installer AppId, previous installation paths and the PDF association ProgID used by Windows to remember the default app.
- Microsoft Store product `9MXR0WLW42XT`, Partner Center package identity and publisher, and application ID `MyPDF`. The legacy execution alias still launches the app for existing pinned items.
- App-data folders under MyPDF, `MYPDF_*` environment overrides, IPC and signing storage identifiers.
- PDF metadata/resource keys and annotation IDs beginning with MyPDF. Existing annotations, tables, OCR, watermarks and thread metadata must remain editable.
- The `.mypdf-review` extension and its serialized format identifier, so saved review exchanges work in both versions.
- Rust library and proof CLI identifiers, native library ABI names and icon source filenames.

The desktop build target is `ashina`. Windows installers, portable ZIPs and the Store package use `Ashina.exe`. On macOS, `Ashina.app` launches `Contents/MacOS/Ashina`. Fresh standalone Windows installs default to an `Ashina` folder. An upgrade reuses its previous folder and redirects existing PDF associations to the new executable. A manually created or pinned Windows shortcut to a specific old executable path may need to be replaced.
