# Third-party notices

This inventory covers the Rust packages selected by the locked Windows x64 build graph. It includes normal and build dependencies and excludes development dependencies.

The inventory was generated from `Cargo.lock` with:

```powershell
cargo tree --offline --locked --target x86_64-pc-windows-msvc --edges normal,build --prefix none --format '{p}|{l}'
```

The SPDX expressions below describe the choices published by each package. The portable package must also include `notices/THIRD_PARTY_LICENSES.txt`, which contains the license texts found in the corresponding Cargo packages.

This file records third-party terms. It does not select or grant a license for PowerPlanSync.

## Slint

PowerPlanSync uses Slint 1.18.0 and its internal crates under the license expression `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0`.

For the Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0, section 2 requires either the `AboutSlint` widget in an accessible About screen or splash screen, or the Slint attribution badge on a public webpage where visitors can easily find it. Section 3 excludes embedded systems and distribution of Slint by itself. The complete license text is included in `notices/THIRD_PARTY_LICENSES.txt`.

Official sources:

- [Slint 1.18.0 license overview](https://github.com/slint-ui/slint/blob/v1.18.0/LICENSE.md)
- [Slint Royalty-free License 2.0](https://slint.dev/agreements/slint-royalty-free-license.pdf)
- [AboutSlint reference](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/misc/aboutslint/)

## Embedded resources

The tray icon is generated in `src/runtime.rs`. The official `AboutSlint` component embeds its Slint attribution logo; its source and licensing belong to the Slint package above. PNG files under `specs/001-app-power-profiles/evidence` are test evidence and are not application resources. The Slint interface uses system fonts. The GPUI interface module `src/interface/theme.rs` embeds IBM Plex Sans and IBM Plex Mono from `assets/fonts/`, copyright 2017 IBM Corp., under the SIL Open Font License 1.1, whose text is `assets/fonts/OFL.txt`. The theme tokens, the motion module, the widgets (`widgets.rs`) and the asset source (`assets.rs`) in `src/interface/` are adapted from Bientôt (Thanaen/Bientot), Copyright (c) 2026 Thanaen, under the MIT license. `scripts/package-portable.ps1` ships the OFL text as `notices/OFL-IBM-Plex.txt`. "Plex" is a Reserved Font Name. The text field in `src/interface/text_input.rs` is adapted from Bientôt under the same MIT license; Bientôt derived it from the `input` example of GPUI (Zed Industries), under the Apache License 2.0. The icons in `assets/icons/` come from Lucide (lucide-static 1.61.0, some through Bientôt), copyright 2026 Lucide Icons and Contributors, under the ISC license; those derived from Feather are also copyright 2013-present Cole Bemis, under the MIT license. Both texts are in `assets/icons/LICENSE`; the packaging script ships it as `notices/LICENSE-Lucide.txt`.

## Locked Windows dependency inventory (246 packages)

### `(MIT OR Apache-2.0) AND Unicode-3.0`

unicode-ident v1.0.26.

### `0BSD OR MIT OR Apache-2.0`

adler2 v2.0.1.

### `Apache-2.0`

accesskit_winit v0.33.2, unicode-linebreak v0.1.5, winit v0.30.13.

### `Apache-2.0 AND MIT`

dpi v0.1.2.

### `Apache-2.0 OR MIT`

autocfg v1.5.1, equivalent v1.0.2, fontique v0.11.1, idna_adapter v1.2.2, indexmap v2.14.2, kurbo v0.13.1, linebender_resource_handle v0.1.1, muda v0.19.3, parlance v0.1.0, parley v0.11.1, parley_data v0.11.1, pin-project v1.1.13, pin-project-internal v1.1.13, pin-project-lite v0.2.17, portable-atomic v1.15.0, resvg v0.48.1, simplecss v0.2.2, spin_on v0.1.1, svgtypes v0.16.1, swash v0.2.10, usvg v0.48.1, utf8_iter v1.0.4, uuid v1.26.1, yazi v0.2.1, zeno v0.3.3.

### `Apache-2.0/MIT`

rustc-hash v1.1.0.

### `BSD-2-Clause`

arrayref v0.3.9.

### `BSD-3-Clause`

tiny-skia v0.12.0, tiny-skia-path v0.12.0.

### `BSD-3-Clause OR Apache-2.0`

moxcms v0.8.1, pxfm v0.1.30.

### `BSD-3-Clause OR MIT OR Apache-2.0`

num_enum v0.7.6, num_enum_derive v0.7.6.

### `BSL-1.0`

clipboard-win v5.4.1, error-code v3.4.0.

### `CC0-1.0 OR Apache-2.0`

imgref v1.12.3.

### `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0`

i-slint-backend-selector v1.18.0, i-slint-backend-winit v1.18.0, i-slint-common v1.18.0, i-slint-compiler v1.18.0, i-slint-core v1.18.0, i-slint-core-macros v1.18.0, i-slint-renderer-software v1.18.0, slint v1.18.0, slint-build v1.18.0, slint-macros v1.18.0.

### `MIT`

cfg_aliases v0.2.2, clru v0.6.3, color_quant v1.1.0, convert_case v0.10.0, derive_more v2.1.1, derive_more-impl v2.1.1, float-cmp v0.9.0, fontdb v0.24.0, harfrust v0.12.0, imagesize v0.15.0, libm v0.2.16, memoffset v0.9.1, natord v1.0.9, pico-args v0.5.0, pin-weak v1.1.0, pulldown-cmark v0.13.4, rgb v0.8.53, rspolib v0.1.2, simd-adler32 v0.3.10, slab v0.4.12, strict-num v0.1.1, strum v0.28.0, strum_macros v0.28.0, synstructure v0.14.0, taffy v0.10.1, tracing v0.1.44, tracing-core v0.1.36, winnow v1.0.4, xmlwriter v0.1.0, zmij v1.0.23.

### `MIT OR Apache-2.0`

accesskit v0.24.1, accesskit_consumer v0.38.0, accesskit_windows v0.34.0, allocator-api2 v0.2.21, annotate-snippets v0.12.16, anstyle v1.0.14, arboard v3.6.1, arrayvec v0.7.8, base64 v0.23.1, bitflags v2.13.2, by_address v1.2.1, cfg-if v1.0.5, chrono v0.4.45, const-field-offset v0.2.1, const-field-offset-macro v0.2.1, countme v3.0.1, crc32fast v1.5.2, critical-section v1.2.0, crossbeam-channel v0.5.17, crossbeam-deque v0.8.8, crossbeam-epoch v0.9.21, crossbeam-utils v0.8.23, data-url v0.3.2, displaydoc v0.2.7, either v1.18.0, euclid v0.22.14, fdeflate v0.3.7, field-offset v0.3.6, flate2 v1.1.10, font-types v0.12.5, form_urlencoded v1.2.2, gif v0.14.2, hashbrown v0.14.5, hashbrown v0.16.1, hashbrown v0.17.1, heck v0.5.0, htmlparser v0.2.1, idna v1.1.0, image v0.25.10, image-webp v0.2.4, itertools v0.15.0, itoa v1.0.18, keyboard-types v0.7.0, lazy_static v1.5.0, log v0.4.34, lyon_algorithms v1.0.21, lyon_extra v1.1.0, lyon_geom v1.0.19, lyon_path v1.0.19, memmap2 v0.9.11, num-traits v0.2.19, once_cell v1.21.4, percent-encoding v2.3.2, pin-utils v0.1.0, png v0.18.1, polycool v0.4.0, proc-macro-crate v3.5.0, proc-macro2 v1.0.107, quote v1.0.47, rayon v1.12.0, rayon-core v1.13.0, read-fonts v0.41.0, rowan v0.17.0, roxmltree v0.21.1, rustc_version v0.4.1, rustversion v1.0.23, scopeguard v1.2.0, semver v1.0.28, serde v1.0.229, serde_core v1.0.229, serde_derive v1.0.229, serde_json v1.0.151, skrifa v0.44.0, smallvec v1.16.1, smol_str v0.2.2, smol_str v0.3.6, snafu v0.8.9, snafu-derive v0.8.9, softbuffer v0.4.8, stable_deref_trait v1.2.1, static_assertions v1.1.0, syn v2.0.119, syn v3.0.6, sys-locale v0.3.2, text-size v1.1.1, thiserror v2.0.20, thiserror-impl v2.0.20, toml_datetime v1.1.1+spec-1.1.0, toml_edit v0.25.15+spec-1.1.0, toml_parser v1.1.3+spec-1.1.0, toml_writer v1.1.2+spec-1.1.0, typed-index-collections v3.5.0, unicase v2.9.0, unicode-bidi v0.3.18, unicode-script v0.5.8, unicode-segmentation v1.13.3, unicode-width v0.2.2, unicode-xid v0.2.6, url v2.5.8, vtable v0.5.0, vtable-macro v0.5.0, webbrowser v1.2.4, weezl v0.1.12, windows v0.62.2, windows_x86_64_msvc v0.52.6, windows_x86_64_msvc v0.53.1, windows-collections v0.3.2, windows-core v0.62.2, windows-future v0.3.2, windows-implement v0.60.2, windows-interface v0.59.3, windows-link v0.2.1, windows-numerics v0.3.1, windows-result v0.4.1, windows-strings v0.5.1, windows-sys v0.52.0, windows-sys v0.60.2, windows-sys v0.61.2, windows-targets v0.52.6, windows-targets v0.53.5, windows-threading v0.2.1.

### `MIT OR Apache-2.0 OR Zlib`

cursor-icon v1.2.0, raw-window-handle v0.6.2, zune-core v0.5.3, zune-jpeg v0.5.15.

### `MIT OR Zlib OR Apache-2.0`

miniz_oxide v0.8.9, miniz_oxide v0.9.1.

### `MIT/Apache-2.0`

quick-error v2.0.1, scoped-tls-hkt v0.1.5, siphasher v1.0.3, unicode-vo v0.1.0, version_check v0.9.5.

### `Unicode-3.0`

fixed_decimal v0.7.2, icu_collections v2.3.0, icu_decimal v2.3.0, icu_decimal_data v2.3.0, icu_locale_core v2.3.0, icu_locale_fallback v2.3.0, icu_locale_fallback_data v2.3.0, icu_normalizer v2.3.0, icu_normalizer_data v2.3.0, icu_properties v2.3.0, icu_properties_data v2.3.0, icu_provider v2.3.1, icu_segmenter v2.3.0, icu_segmenter_data v2.3.0, litemap v0.8.3, potential_utf v0.1.6, tinystr v0.8.4, writeable v0.6.4, yoke v0.8.3, yoke-derive v0.8.3, zerofrom v0.1.8, zerofrom-derive v0.1.8, zerotrie v0.2.5, zerovec v0.11.8, zerovec-derive v0.11.6.

### `Unlicense OR MIT`

byteorder-lite v0.1.0, memchr v2.8.3.

### `Zlib`

foldhash v0.2.0, slotmap v1.1.1.

### `Zlib OR Apache-2.0 OR MIT`

bytemuck v1.25.2, bytemuck_derive v1.12.1, tinyvec v1.13.3.
