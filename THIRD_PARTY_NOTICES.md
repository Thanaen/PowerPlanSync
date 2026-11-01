# Third-party notices

This inventory covers the Rust packages selected by the locked Windows x64 build graph. It includes normal and build dependencies and excludes development dependencies.

The inventory and `notices/THIRD_PARTY_LICENSES.txt` are generated from `Cargo.lock` by `scripts/generate-third-party-notices.ps1`, which lists the packages with:

```powershell
cargo tree --offline --locked --target x86_64-pc-windows-msvc --edges normal,build --prefix none --format '{p}|{l}'
```

The SPDX expressions below describe the choices published by each package. The portable package must also include `notices/THIRD_PARTY_LICENSES.txt`, which contains the license texts found in the corresponding Cargo packages.

PowerPlanSync itself is distributed under the MIT license, whose text is in `LICENSE` at the root of the repository and of the portable package. This file records the terms of third-party components only. The MIT license does not cover them, and their terms do not change it.

All these terms are permissive and allow distributing the binary of an MIT project, provided their notices ship with it. Every package offered under several licenses includes MIT or Apache-2.0 among them. The packages under a single license use MIT, Apache-2.0 (the `gpui-pre` packages), BSD-2-Clause, BSD-3-Clause, Zlib or Unicode-3.0. None is copyleft. The embedded icons are under ISC and MIT, and the embedded fonts under the SIL Open Font License 1.1, which allows bundling them with software under any license as long as the fonts are not sold by themselves.

## Embedded resources

The tray icon is generated in `src/interface/tray.rs`. PNG files under `specs/001-app-power-profiles/evidence` are test evidence and are not application resources. The interface module `src/interface/theme.rs` embeds IBM Plex Sans and IBM Plex Mono from `assets/fonts/`, copyright 2017 IBM Corp., under the SIL Open Font License 1.1, whose text is `assets/fonts/OFL.txt`. The theme tokens, the motion module, the widgets (`widgets.rs`) and the asset source (`assets.rs`) in `src/interface/` are adapted from Bientôt (Thanaen/Bientot), Copyright (c) 2026 Thanaen, under the MIT license. `scripts/package-portable.ps1` ships the OFL text as `notices/OFL-IBM-Plex.txt`. "Plex" is a Reserved Font Name. The text field in `src/interface/text_input.rs` is adapted from Bientôt under the same MIT license; Bientôt derived it from the `input` example of GPUI (Zed Industries), under the Apache License 2.0. The icons in `assets/icons/` come from Lucide (lucide-static 1.61.0, some through Bientôt), copyright 2026 Lucide Icons and Contributors, under the ISC license; those derived from Feather are also copyright 2013-present Cole Bemis, under the MIT license. Both texts are in `assets/icons/LICENSE`; the packaging script ships it as `notices/LICENSE-Lucide.txt`.

## Locked Windows dependency inventory (280 packages)

### `(MIT OR Apache-2.0) AND Unicode-3.0`

unicode-ident v1.0.26.

### `0BSD OR MIT OR Apache-2.0`

adler2 v2.0.1.

### `Apache-2.0`

gpui-pre v0.3.9, gpui-pre-collections v0.3.9, gpui-pre-derive-refineable v0.3.9, gpui-pre-http-client v0.3.9, gpui-pre-macros v0.3.9, gpui-pre-platform v0.3.9, gpui-pre-refineable v0.3.9, gpui-pre-scheduler v0.3.9, gpui-pre-shared-string v0.3.9, gpui-pre-sum-tree v0.3.9, gpui-pre-util v0.3.9, gpui-pre-windows v0.3.9, gpui-pre-ztracing v0.3.9, gpui-pre-ztracing-macro v0.3.9.

### `Apache-2.0 AND MIT`

dpi v0.1.2.

### `Apache-2.0 OR BSL-1.0`

ryu v1.0.23.

### `Apache-2.0 OR MIT`

async-channel v2.5.0, async-task v4.7.1, autocfg v1.5.1, concurrent-queue v2.5.0, ctor v1.0.13, equivalent v1.0.2, event-listener v5.4.2, event-listener-strategy v0.5.4, idna_adapter v1.2.2, indexmap v2.14.2, kurbo v0.13.1, link-section v0.19.3, linktime-proc-macro v0.2.3, muda v0.21.2, parking v2.2.1, pin-project v1.1.13, pin-project-internal v1.1.13, pin-project-lite v0.2.17, resvg v0.46.0, rustc-hash v2.1.3, simplecss v0.2.2, svgtypes v0.16.1, usvg v0.46.0, utf8_iter v1.0.4, uuid v1.26.1, waker-fn v1.2.0.

### `Apache-2.0/MIT`

bit_field v0.10.3, flume v0.12.0, pollster v0.4.0.

### `BSD-2-Clause`

arrayref v0.3.9.

### `BSD-2-Clause OR Apache-2.0 OR MIT`

zerocopy v0.8.57, zerocopy-derive v0.8.57.

### `BSD-3-Clause`

exr v1.74.2, lebe v0.5.3, sha1_smol v1.0.1, tiny-skia v0.11.4, tiny-skia-path v0.11.4.

### `BSD-3-Clause OR Apache-2.0`

moxcms v0.8.1, pxfm v0.1.30.

### `CC0-1.0 OR MIT-0 OR Apache-2.0`

dunce v1.0.5.

### `MIT`

bytes v1.12.1, color_quant v1.1.0, convert_case v0.10.0, core_maths v0.1.1, derive_more v2.1.1, derive_more-impl v2.1.1, embed-resource v3.0.12, fax v0.2.7, float-cmp v0.9.0, float_next_after v1.0.0, fontdb v0.23.0, http-body v1.1.0, imagesize v0.14.0, libm v0.2.16, pico-args v0.5.0, pulp v0.22.3, pulp-wasm-simd-flag v0.1.1, raw-cpuid v11.6.0, reborrow v0.5.5, rgb v0.8.53, rustybuzz v0.20.1, schemars v1.2.3, schemars_derive v1.2.3, seahash v4.1.0, simd-adler32 v0.3.10, slab v0.4.12, spin v0.10.1, spin v0.9.9, strict-num v0.1.1, strum v0.28.0, strum_macros v0.28.0, synstructure v0.14.0, taffy v0.13.0, tiff v0.11.3, tracing v0.1.44, tracing-attributes v0.1.31, tracing-core v0.1.36, vswhom v0.1.0, vswhom-sys v0.1.3, which v8.0.6, winnow v1.0.4, winreg v0.56.0, xmlwriter v0.1.0, zmij v1.0.23.

### `MIT OR Apache-2.0`

accesskit v0.24.1, accesskit_consumer v0.38.0, accesskit_windows v0.34.0, anyhow v1.0.104, arrayvec v0.7.8, backtrace v0.3.76, base64 v0.22.1, bitflags v2.13.2, cc v1.4.7, cfg-if v1.0.5, chrono v0.4.45, crc32fast v1.5.2, crossbeam-channel v0.5.17, crossbeam-deque v0.8.8, crossbeam-epoch v0.9.21, crossbeam-utils v0.8.23, data-url v0.3.2, displaydoc v0.2.7, dyn-clone v1.0.20, either v1.18.0, enumn v0.1.14, euclid v0.22.14, fdeflate v0.3.7, find-msvc-tools v0.1.13, flate2 v1.1.10, form_urlencoded v1.2.2, futures v0.3.34, futures-channel v0.3.34, futures-core v0.3.34, futures-executor v0.3.34, futures-io v0.3.34, futures-macro v0.3.34, futures-sink v0.3.34, futures-task v0.3.34, futures-util v0.3.34, getrandom v0.3.4, getrandom v0.4.3, gif v0.14.2, half v2.7.1, hash32 v0.3.1, hashbrown v0.16.1, hashbrown v0.17.1, heapless v0.9.3, heck v0.5.0, http v1.5.0, idna v1.1.0, image v0.25.10, image-webp v0.2.4, inventory v0.3.25, itertools v0.14.0, itoa v1.0.18, keyboard-types v0.8.3, libc v0.2.189, lock_api v0.4.14, log v0.4.34, lyon v1.0.19, lyon_algorithms v1.0.21, lyon_geom v1.0.19, lyon_path v1.0.19, lyon_tessellation v1.0.23, memmap2 v0.9.11, num-complex v0.4.6, num-traits v0.2.19, num_cpus v1.17.0, once_cell v1.21.4, parking_lot v0.12.5, parking_lot_core v0.9.12, paste v1.0.15, percent-encoding v2.3.2, png v0.17.16, png v0.18.1, polycool v0.4.0, ppv-lite86 v0.2.21, proc-macro-crate v3.5.0, proc-macro2 v1.0.107, profiling v1.0.18, profiling-procmacros v1.0.18, quote v1.0.47, rand v0.9.5, rand_chacha v0.9.0, rand_core v0.9.5, rayon v1.12.0, rayon-core v1.13.0, ref-cast v1.0.27, ref-cast-impl v1.0.27, regex v1.13.1, regex-automata v0.4.18, regex-syntax v0.8.11, roxmltree v0.21.1, rustc_version v0.4.1, scopeguard v1.2.0, semver v1.0.28, serde v1.0.229, serde_core v1.0.229, serde_derive v1.0.229, serde_derive_internals v0.30.0, serde_json v1.0.151, serde_spanned v1.1.1, shlex v2.0.1, smallvec v1.16.1, smol_str v0.3.6, stable_deref_trait v1.2.1, static_assertions v1.1.0, syn v2.0.119, syn v3.0.6, thiserror v2.0.20, thiserror-impl v2.0.20, toml v1.1.6+spec-1.1.0, toml_datetime v1.1.1+spec-1.1.0, toml_edit v0.25.15+spec-1.1.0, toml_parser v1.1.3+spec-1.1.0, toml_writer v1.1.2+spec-1.1.0, tray-icon v0.26.1, ttf-parser v0.25.1, unicode-bidi v0.3.18, unicode-script v0.5.8, unicode-segmentation v1.13.3, url v2.5.8, web-time v1.1.0, weezl v0.1.12, windows v0.62.2, windows-collections v0.3.2, windows-core v0.62.2, windows-future v0.3.2, windows-implement v0.60.2, windows-interface v0.59.3, windows-link v0.2.1, windows-numerics v0.3.1, windows-registry v0.6.1, windows-result v0.4.1, windows-strings v0.5.1, windows-sys v0.61.2, windows-threading v0.2.1.

### `MIT OR Apache-2.0 OR Zlib`

raw-window-handle v0.6.2, zune-core v0.5.3, zune-inflate v0.2.54, zune-jpeg v0.5.15.

### `MIT OR Zlib OR Apache-2.0`

miniz_oxide v0.8.9, miniz_oxide v0.9.1.

### `MIT/Apache-2.0`

bitflags v1.3.2, etagere v0.2.15, qoi v0.4.1, quick-error v2.0.1, rustc-demangle v0.1.28, serde_urlencoded v0.7.1, siphasher v1.0.3, svg_fmt v0.4.5, unicode-bidi-mirroring v0.4.0, unicode-ccc v0.4.0, unicode-properties v0.1.4, unicode-vo v0.1.0, version_check v0.9.5.

### `Unicode-3.0`

icu_collections v2.3.0, icu_locale_core v2.3.0, icu_normalizer v2.3.0, icu_normalizer_data v2.3.0, icu_properties v2.3.0, icu_properties_data v2.3.0, icu_provider v2.3.1, litemap v0.8.3, potential_utf v0.1.6, tinystr v0.8.4, writeable v0.6.4, yoke v0.8.3, yoke-derive v0.8.3, zerofrom v0.1.8, zerofrom-derive v0.1.8, zerotrie v0.2.5, zerovec v0.11.8, zerovec-derive v0.11.6.

### `Unlicense OR MIT`

aho-corasick v1.1.5, byteorder v1.5.0, byteorder-lite v0.1.0, memchr v2.8.3.

### `Zlib`

foldhash v0.2.0, slotmap v1.1.1.

### `Zlib OR Apache-2.0 OR MIT`

bytemuck v1.25.2, tinyvec v1.13.3.
