# Windows ARM64 builds

The **Windows build** GitHub Actions workflow accepts `architecture: x64`,
`arm64`, or `both`. ARM64 uses a native Windows 11 Arm runner and Rust's
`aarch64-pc-windows-msvc` host toolchain. Leave `release` empty for a validation
build; artifacts remain attached to its workflow run.

ARM64 packages are named `SkyPDF-<version>-windows-arm64.zip` and
`SkyPDF-<version>-Setup-arm64.exe`, each with a SHA-256 checksum. The existing
x64 filenames remain unchanged. The ARM installer accepts ARM64 Windows only.
Standalone update checks retain the installed architecture and never offer an
x64 installer to the ARM64 app.

The application, bundled Tesseract executable and CTranslate2/SentencePiece DLL
are built for the same architecture and checked before packaging. English,
Turkish and German OCR models are unchanged. Translation models remain optional
downloads. The feature guide and dependency notices are included in both formats.

Validation runs the Rust tests on ARM64, checks DLL loading and ABI, rejects a
deliberately mismatched DLL, performs real OCR on a synthetic scanned PDF, and
translates synthetic text with one explicitly downloaded, checksum-verified
language pack. The real model checks use release-mode code, separate timed
download/inference steps, and print progress without document content. Verified
native dependencies are cached before later checks so a failed validation does
not discard their successful builds. No user documents or recordings are used.

For Windows on ARM in Parallels on Apple Silicon, use the ARM64 build. Ordinary
Intel/AMD Windows PCs need x64. Architecture alone does not solve graphics
compatibility: this build still uses the existing DirectX 12 renderer, which
can fall back to software when the VM cannot provide compatible acceleration.
Performance in Parallels must be measured separately.

Microsoft Store packaging remains x64; adding an ARM64 Store package is a
separate submission step. This workflow does not change the accepted Store
identity or the existing Store submission.
