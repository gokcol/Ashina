# Privacy Policy for Ashina

The app was previously named MyPDF. Its local storage and application identity remain the same after the rename.

Effective date: 15 November 2026

This policy covers Ashina, the Microsoft Store edition of Ashina by
Orhan Gökçöl, and the Ashina desktop application distributed through the project's
GitHub releases. It describes the behavior in version 3.1.2.

## Information processed on your device

Ashina is a desktop PDF editor. It accesses the documents, images, SVG artwork,
form data and certificate files you choose to open or import. These files may
contain personal information. The app processes their contents on your device
to display, search, edit, annotate, convert, organize, print and sign documents.
Text recognition also runs on the device: Apple Vision on Mac, and bundled
Tesseract with local language models on Windows. OCR results can be reviewed and
explicitly copied to the clipboard; they are not uploaded. Ashina does not upload these
files or their contents to the developer or to a Ashina server.

No Ashina account is required. The app has no advertising, usage analytics,
tracking SDKs or automatic crash-report uploads. The developer does not receive
your document contents, usage history or signing keys through the app, and does
not sell or share that information for advertising.

## Local storage and recovery

To provide its features, Ashina stores settings, recent-file and favorite
shortcuts, file paths, reading positions, custom stamps and signature artwork,
saved signature appearances, public certificate information, trust preferences
and references to signing identities. It also saves recovery copies of applied
unsaved document changes, including document content and source-path metadata.
Document templates contain local copies of the PDFs you save as templates. Batch-processing preferences are stored locally; generated documents are saved to the destination you choose. Temporary local files may be used while processing or printing documents.

The default application-data locations are:

- Windows installer/portable edition: `%LOCALAPPDATA%\MyPDF`
- Microsoft Store edition: Windows manages new AppData files in private package storage, presented to the app as `%LOCALAPPDATA%\MyPDF`. Existing installer files may be visible through Windows' normal fallback.
- macOS: `~/Library/Application Support/MyPDF`

A configured `MYPDF_DATA_DIR` can change this location. Files you save or export
remain in the locations you choose. Ashina does not provide cloud storage or
automatic cloud synchronization. If you use a cloud-synced folder, shared drive,
backup service or network printer, that service may handle the files according
to your settings and its own policies.

Settings, artwork and recovery files are not separately encrypted by Ashina.
Their protection depends on operating-system access controls and any device
encryption you enable. Recovery copies can contain sensitive document content.

## Voice comments

Ashina accesses the default microphone only when you press Record for a voice comment and the operating system grants access. Recording and playback run locally. You can pause, stop and keep, or discard a recording. Each voice note is limited to three minutes.

Kept recordings become part of the document's local edits and may be included in recovery copies. Saving the PDF embeds the recording as a WAV file attachment; anyone who receives that PDF may be able to extract and listen to it. Save audio writes a separate local WAV file only at your request. Ashina does not upload recordings or transcribe them. Prepare for sharing can remove recordings from a separate output copy.

## On-device translation

Translation runs on the device with no cloud fallback. On supported Macs, Ashina
uses Apple's installed translation models. Apple's translation service may
collect language and usage metrics, but not original or translated text. Ashina
requires a separate, explicit language-download action before a missing pair
can be used; document text is not included in that download request.

On Windows, Ashina uses a local CTranslate2 runtime. When you choose to download
a language pack, Ashina contacts the pack's public distribution host (currently
Argos Open Tech's argos-net.com service). That host may receive ordinary
connection information such as your IP address and the requested model file,
but no document text. Downloads are verified before use. Inference uses local
files and does not make network requests.

Language choices and favorites, model files and model verification metadata are
stored locally. Source and translated text are held in memory and are not saved in a separate
translation history. Translation itself does not change the PDF. If you choose
Attach translation as note, the result becomes a document annotation and is
included in normal PDF saves and recovery copies. Closing the translation panel
or changing the document/page clears the displayed text. Copying a result to
the system clipboard is an explicit action. Downloaded Windows model packs can
be removed from the app's translation-models directory while Ashina is closed;
Apple's downloaded models are managed by the operating system.

## Certificates and digital signatures

Creating a digital identity uses the name and optional details you enter, such
as an email address. Signing uses the certificate and signing identity you
select. Private keys are managed by the operating-system certificate store,
a configured hardware provider, or a password-protected PKCS#12 file, depending
on the identity you use. Identity-file passphrases and private keys are not saved
in Ashina's settings or identity catalogue.

Signed PDFs contain public certificate information and may contain the signer
name, signing time, reason, location and signature artwork. Anyone receiving
the signed PDF may be able to read that information. Signature verification
uses local information; Ashina does not contact an online timestamp or
certificate-revocation service.

## External services and information you choose to share

Ashina can open a document's web or email link in your browser or mail application
after confirmation. That destination's policies then apply. Copying content uses
the system clipboard, which may be accessible to other software or synchronized
by your operating-system settings.

The SVG prompt helper creates text locally. It does not send prompts or documents
to an AI provider. If you copy a prompt into an external AI service yourself,
that service handles the information under its own policies.

Microsoft Store, GitHub, operating-system services and third-party certificate
providers may process information independently under their own policies. Ashina
does not control their distribution, account, diagnostic or support systems.

## Retention and your controls

You can clear recent-file shortcuts, remove favorites and saved artwork, and
restore or discard recovery copies in the app. Clearing a shortcut does not
delete its PDF. Saving or explicitly discarding a document clears its associated
recovery copy; abandoned recovery copies remain available for your decision.
Other saved preferences and library items remain until changed or removed.

Uninstalling the Microsoft Store edition removes its private application data,
including settings, custom stamps and recovery copies. Save recovered documents
and export artwork you want to keep before uninstalling. Existing application data
from the Windows installer/portable edition and the Mac app is retained after
uninstall. To remove that data, close the app and delete its application-data
folder, keeping a backup of anything you want to retain. Saved
documents and exported certificates must be managed separately. Certificates
and private keys in Keychain, the Windows certificate store or a hardware token
must be managed through that system or provider.

## Contact and policy changes

For privacy questions, contact the maintainer through
[the Ashina issue tracker](https://github.com/gokcol/Ashina/issues).
GitHub issues are public; do not include confidential documents, passwords or
private keys. Information you voluntarily provide there is used to address
your request and is also subject to GitHub's policies and retention controls.

This policy will be updated when the app's data practices change. The effective
date and applicable app version will be shown on this page.
