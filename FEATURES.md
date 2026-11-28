# Ashina feature guide

Previously named **MyPDF** and **SkyPDF**. The Store listing uses **Ashina**; existing documents, settings and review exchange files remain compatible.

**Release: 3.1.9** · macOS (Apple Silicon) and Windows (x64 and ARM64)

Ashina is a native desktop PDF editor for local documents. Its main workspaces are **Read, Annotate, Edit, Draw, Forms, Sign, Pages and Export**. Contextual controls appear in the right panel; tool variations share compact menus that open on hover, click or tap.

## Open, read and navigate

- Open PDFs in document tabs; reorder tabs, use named groups, and restore a workspace.
- Home with file opening and drag-and-drop, tool shortcuts, recent documents, favorites and page previews.
- Customize Home section order and visibility.
- Page thumbnails, bookmarks/outline, document search, page navigation and reading history.
- Fit page, fit width, zoom, continuous viewing and a distraction-free Zen mode.
- Follow internal links and approved web/email links; PDF scripts are not executed.
- Select and copy text, including supported formatting; copy images and page regions.
- Inspect document properties, page dimensions, protection and signatures.
- Print through the native Mac print system or Windows print preview.

## Edit document content

- Edit supported existing text in place, or add text on a blank or existing page.
- Font family and face, size, color, paragraph alignment and selected-character formatting.
- Move text by dragging inside its box; Shift-drag selects characters.
- Resize text width along its text direction; height grows and shrinks with its contents.
- Edit rotated and mirrored text in place; adjust rotation in the properties panel, including ±15° and Reset.
- First click outside finishes the current text edit; another click starts the next action.
- Match nearby text styling and save reusable styles.
- Select supported existing page images; replace, move, resize, rotate or delete them.
- Insert images with proportional page fitting; use opacity, image borders, blur and local background removal where supported.
- Create and edit tables, including detected bordered tables: cells, rows, columns, merged cells, sizing, formatting, templates and spreadsheet paste.
- Inspect and change PDF title, author, subject, keywords, creator and producer; synchronize supported Info/XMP metadata.

**Mode separation:** Edit works on document content. Annotation text boxes, comments, stamps and drawings remain annotation objects; Edit does not modify them. Text created in Edit is not an annotation text box in Annotate.

## Annotate and review

- Highlight, underline, squiggly underline, strikethrough and rectangular area highlighting.
- Annotation text boxes with font family/style, size and color.
- Text comments anchored by small page markers: click to update an open right panel, or show an editable popover beside the marker when the panel is hidden; open the right panel for the complete conversation, with author names, initials, timestamps, chronological replies, a reply composer, and Resolve/Reopen. An outside click dismisses the popover without placing another note.
- Suggest replacement wording or an insertion under Annotate → More; replacement is also available on selected text. Suggestions never rewrite the original page text.
- Voice comments under the same **Comment** menu: record locally, pause/resume, keep or discard, play back, and save audio separately.
- Voice recordings use the default microphone only after Record and system permission; each recording is limited to three minutes.
- Saved PDFs contain voice audio as standard WAV attachments. Ashina provides inline playback; playback in other readers varies.
- Callouts with visible text boxes and leaders pointing to page details.
- Review replies, author information, open/resolved status and author/status/color filtering.
- Export CSV summaries or searchable PDF summaries with numbered comments beside page previews, including long replies on continuation pages.
- Saved PDFs include standard reply annotations with author names, dates and thread links for other PDF readers. Ashina reloads supported reply changes on its own unchanged parent comments; unsupported or externally altered parent annotations are preserved as foreign PDF objects.
- Export/import Ashina review files (`.mypdf-review`) from Review actions. Includes native comments, drawings, stamps, voice recordings and replies; validates the base document, skips duplicates, and keeps conflicts as separate comments after a preview. Import is undoable. This is a Ashina exchange format, not Acrobat XFDF/FDF.
- Move annotations by dragging inside them; resize and rotate using selection controls.
- Undo/Redo for supported annotation changes, including replacement recordings.

## Draw and arrange

- Freehand pen, eraser, polylines and polygons.
- Revision clouds for outlining areas that need review.
- Rectangle, square, diamond, triangle, pentagon and hexagon shapes; ellipse, circle and semicircle variations.
- Straight, dashed, dotted, dash-dot, wavy, double, elbow, stepped and curved line options.
- Single/double arrows and routed arrow variations, with configurable start/end heads and head size.
- Stroke color, width and opacity; solid or patterned fills where supported.
- Select or Lasso from one grouped tool button.
- Immediately grab a completed drawing without first clicking outside.
- Smooth dragging with alignment guides, proportional resizing, rotation/reset and flipping.
- Multi-selection, grouping, layer order, duplication, alignment, equal spacing and equal sizing.
- Repeat selected artwork on chosen pages with destination previews.
- Only the selected drawing-tool icon previews the current drawing color.

## Stamps, artwork and signatures library

- Built-in Arrows, Symbols and Stamps collections, plus Saved artwork.
- Existing artwork remains visible in its original order by default.
- Fifteen optional additions in each built-in category, hidden until enabled.
- One collection-management button opens **Add artwork** and **Manage collection**.
- Reorder items by dragging or using arrows; show/hide items per category, search/filter the manager, restore defaults, save or cancel.
- Search saved artwork by name, folder or tags; stamp favorites and recent items.
- Create custom stamp variants without changing the built-in originals.
- Edit stamp text, colors and appearance; manage custom artwork, duplicate it, or remove it with session undo.
- SVG Studio with a visual editor and source editing, text/shape controls, border templates, fill/background styling and transparency.
- Import supported artwork or paste SVG; use the prompt builder to create an external prompt and import the resulting SVG. Ashina does not send that prompt to an AI service.
- Remember the last explicitly chosen placement size for stamps and saved ink signatures.

## Touch interaction

- Automatic detection based on touch capabilities reported by the operating system.
- Settings for automatic, touch-oriented or mouse-oriented operation.
- Optional finger drawing in Draw mode.
- Long-press context actions and a contextual floating toolbar.
- Touch settings remain behind one contextual button, with parameters in the right panel.
- Mouse and keyboard remain usable alongside touch. Hardware-dependent behavior must be verified on the target device.

## Pages and document assembly

- New blank pages use A4 by default.
- Resize individual pages while keeping content at its current size.
- Insert, reorder, rotate, crop, duplicate, delete and extract pages.
- Multi-page selection, page ranges, copy/cut/paste pages and copying between documents.
- Merge PDFs or split a document into separate outputs.
- Insert supported files by drag-and-drop or Insert, including into an empty document.
- Supported import families: PDF; PNG/JPEG, TIFF, HEIC/HEIF, WebP, GIF and other supported raster images; SVG; TXT, Markdown, DOCX and RTF.
- DOCX/RTF layout conversion requires a supported installed converter such as Microsoft Word or LibreOffice; Ashina explains the requirement if none is available.
- Page numbers, headers, footers, Bates-style numbering and text/image watermarks, with page-range and appearance controls.
- Multiple-pages-per-sheet output with margins, headers/footers and watermarks.

## Forms

- Create text, checkbox, radio, dropdown, list, button, image, date and signature fields.
- Move, resize and style fields; configure names, tooltips, requirements, choices and defaults.
- Fill forms, clear values and flatten supported form fields.
- Detect likely fields in paper-style forms.
- Reusable layouts for tables, labeled fields, checklists, choice groups and signature blocks.
- Arrange tab order by rows or columns and show field-order numbers.
- Import/export form values as FDF, XFDF or CSV.
- Store personal details locally and review suggested field matches before filling.

## Sign and protect

- Visual signatures from handwriting, typed names or imported images, with a local reusable library.
- Certificate-based digital signing, signature inspection and verification reporting.
- Four digital signature templates with a live page preview and English, Turkish or German signing details.
- Quick Sign with a chosen default identity and visual appearance.
- Supported signing and editing workflows preserve existing signed revisions; changes may affect a signature's document-modification status.
- Open password-protected PDFs; save protected copies or remove protection when permitted.
- Secure redaction workflows for supported text and image content; raster fallback for content that cannot be removed precisely.
- Signed/encrypted documents have operation-specific restrictions, explained before changing them.

## OCR, comparison and conversion

- On-device OCR for all pages, the current page or a selected range.
- Review/copy recognized text and jump to its page; save a searchable text layer without changing the scan's visible appearance.
- Mac OCR uses Apple Vision. Windows bundles Tesseract with English, Turkish and German models.
- Compare two PDFs side by side, highlighting added, removed and changed text with change navigation.
- Export text to DOCX, TXT or Markdown; export page appearances as PNG/JPEG.
- Reduce file size using quality presets; keep text and vector drawings where supported.
- Flatten PDFs to rendered page images.
- Reusable document templates and local batch processing for OCR, compression, numbering and watermarks.

## Private translation

- Translate selected text or a page in the right panel, with original/result views, language choices, Copy and cancellation.
- Optionally attach a translation as an undoable note; translation does not replace the source text.
- macOS 26+ uses Apple's installed on-device translation models; language downloads require an explicit action.
- Windows uses a local CTranslate2/SentencePiece runtime with downloadable English ↔ Turkish, German, French and Spanish packs.
- No cloud fallback and no separate translation history. Model availability and supported language pairs vary by backend.

## Prepare for sharing

**Export → Prepare for sharing…** creates a separate copy. The same dialog is available through Tools and Find a Tool.

- Remove comments/drawings, document metadata and attachments/voice recordings.
- Default full cleanup rebuilds each visible page as an image in a fresh PDF, excluding source objects, hidden text, cropped content and previous revisions.
- Choose output resolution; cancel processing before the output is written.
- Keep searchable text with selective cleanup by turning off hidden-content removal. This mode clearly reports that hidden information may remain.
- The original document is not overwritten.

Visible sensitive information must be redacted first. Full cleanup removes search/editability, interactive forms/links and digital signature validity from the sharing copy. Review the exported document before distributing it.

## Appearance, productivity and file safety

- Five visual styles: Studio (default), Sage, Graphite, Glacier and Calico.
- Light, dark or system appearance; compact visual previews and icon-color choices.
- Distinct selected modes and tools, contextual settings icons and configurable interface density.
- Reusable text/artwork styles, keyboard shortcuts and Find a Tool.
- Safe Save/Save As behavior, Undo/Redo, recovery copies and document history where supported.
- Mac update checks use GitHub releases; Windows Store installations receive updates through Microsoft Store.
- Local document processing and libraries. Optional translation features and external conversion have their own platform/service requirements.

## Practical limits

- Existing PDF editing depends on how the source encodes text, fonts, images and layout. Not every PDF can be reconstructed as if it were a word-processing document.
- DOCX export provides readable text and basic paragraphs, not a faithful recreation of arbitrary PDF layouts.
- Text comparison does not compare images or graphical layout; scanned documents need OCR first.
- XFA forms are not supported. Standard AcroForms are supported within the documented field capabilities.
- A black shape, cropped area or blurred image is not secure redaction.
- Readers differ in support for interactive attachments, review details and editable appearances.
- Published desktop packages target Apple Silicon Macs and Windows x64 / ARM64. Use ARM64 for Windows on ARM, including Parallels on Apple Silicon; use x64 for Intel/AMD PCs. Microsoft Store packaging currently remains x64. The separate iPad development preview does not have full desktop parity.

## Feature freeze

Version 3.1.3 closes the agreed feature set. Subsequent work prioritizes reliability, performance, compatibility, accessibility/usability corrections and regression prevention. New feature work requires a separate decision.
