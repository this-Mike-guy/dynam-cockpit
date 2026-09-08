# DYNAM Cockpit — personal edition

Personal, noncommercial fork of [Cockpit Tools](https://github.com/jlcodes99/cockpit-tools) v1.3.43 by **jlcodes99 and contributors**, based on upstream commit `4089320ac78bbd7ebaabc5bb878507b292b52c01`.

Source for this fork: <https://github.com/this-Mike-guy/dynam-cockpit>.

## Changes

- Target the Windows stack overflow during Codex quota refresh. See the commit and regression tests for the precise scope; a successful source build alone does not establish runtime stability.
- DYNAM Cockpit name, navy and mint D mark, desktop identity `com.dynam.cockpit.personal`, and visible upstream attribution in About.
- Upstream self-updates are disabled in both the interface and the native updater registration. Install reviewed builds manually so an upstream release cannot silently replace the patch.
- The existing account/configuration storage format and paths remain compatible with Cockpit Tools.
- Version `1.3.43-dynam.2` recognizes the Windows `OpenAI.Codex` package even when its desktop executable is named `ChatGPT.exe`. ChatGPT Classic and bundled Codex command-line processes are excluded from automatic desktop selection.
- Desktop launch validation runs before preparing credentials, stopping an instance, or writing its profile. If Codex is open, manual changes require a restart confirmation; this detects the application, not whether its tasks are idle. Automatic switching defers while the desktop is open.
- English entries cover missing buttons and labels in the Codex and settings flows, including Retry. Other languages and upstream attribution remain available.
- Version `1.3.43-dynam.3` distinguishes the saved Codex profile from the desktop's signed-in account. The accounts page reports whether a Codex process is detected, not detected, or unknown; task activity and the running account are explicitly unverified. Launch completion requires observing the desktop process, but does not claim that its account identity has been verified.

## License and attribution

The upstream software and these modifications are provided under [Creative Commons Attribution-NonCommercial-ShareAlike 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/). Original authorship, copyright notices, and upstream license restrictions remain in force. Modifications and the new branding are by DYNAM, September 2026.

This is a personal edition, not an official upstream release or a commercial DYNAM product. DYNAM branding does not grant commercial rights to the underlying software. Distributed adaptations must retain appropriate attribution, identify modifications, and use the same license. See the original [license section](README.en.md#license) and notices for bundled third-party software.

## Icon source

The editable mark is `src/assets/dynam-mark.svg`. `python scripts/generate-dynam-icons.py` (requires Pillow) regenerates desktop PNG/ICO/ICNS assets from the same geometry.
