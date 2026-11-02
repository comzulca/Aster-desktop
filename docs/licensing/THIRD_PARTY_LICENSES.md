# Third-party licence inventory

This is the declared-licence snapshot from successful main workflow [41346860296, attempt 1](https://github.com/comzulca/Aster-desktop/actions/runs/41346860296), source `ff474f9179b9e5f83b3f8cbd45c68a660b58ffcc`, downloaded on 2 November 2026. The Apache transition leaves these dependency inputs unchanged. Other branch tips may have a different locked graph; regenerate exact-build inventories using the repository scripts.

It records **573 third-party packages**, of which **298** were marked required for the Windows packaged-application graph. The baseline generator reported **zero required missing licence texts** and **zero required unknown licence declarations**. These counts describe source inventory and do not independently establish binary contents or legal clearance.

Upstream expressions are reproduced verbatim. Apache alternatives are available where expressly offered; other terms and copyright notices remain effective. `AND` obligations are cumulative. Legacy `/` separators remain visible as received; the SBOM generator normalises them separately. Full upstream texts are retained in vendored source and generated `THIRD-PARTY-NOTICES.txt`.

| Dependency input | SHA-256 |
| --- | --- |
| `pnpm-lock.yaml` | `303208ac5913d16f08a2c7c9b03868b679d2c35eec710da7cb31d192984c09f1` |
| `src-tauri/Cargo.lock` | `5e8345ba43c9cf8450cd58c1cca6cc397459de51f6373fc9fe5f4b8674c110a9` |

| Ecosystem | Package | Version | Declared licence | Required for Windows app graph | Source |
| --- | --- | --- | --- | --- | --- |
| cargo | `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | Yes | github.com/oyvindln/adler2 |
| cargo | `aho-corasick` | 1.1.5 | Unlicense OR MIT | Yes | github.com/burntsushi/aho-corasick |
| cargo | `alloc-no-stdlib` | 3.0.0 | BSD-3-Clause | Yes | github.com/dropbox/rust-alloc-no-stdlib |
| cargo | `alloc-stdlib` | 0.3.0 | BSD-3-Clause | Yes | github.com/dropbox/rust-alloc-no-stdlib |
| cargo | `android_system_properties` | 0.1.6 | MIT OR Apache-2.0 | No | github.com/nical/android_system_properties |
| cargo | `anyhow` | 1.0.104 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/anyhow |
| cargo | `atk` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `atk-sys` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | Yes | github.com/smol-rs/atomic-waker |
| cargo | `autocfg` | 1.5.1 | Apache-2.0 OR MIT | No | github.com/cuviper/autocfg |
| cargo | `aws-lc-rs` | 1.18.1 | ISC AND (Apache-2.0 OR ISC) | Yes | github.com/aws/aws-lc-rs |
| cargo | `aws-lc-sys` | 0.45.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) | Yes | github.com/aws/aws-lc-rs |
| cargo | `base64` | 0.21.7 | MIT OR Apache-2.0 | No | github.com/marshallpierce/rust-base64 |
| cargo | `base64` | 0.22.1 | MIT OR Apache-2.0 | Yes | github.com/marshallpierce/rust-base64 |
| cargo | `base64` | 0.23.1 | MIT OR Apache-2.0 | Yes | github.com/marshallpierce/rust-base64 |
| cargo | `bit-set` | 0.8.0 | Apache-2.0 OR MIT | Yes | github.com/contain-rs/bit-set |
| cargo | `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | Yes | github.com/contain-rs/bit-vec |
| cargo | `bitflags` | 1.3.2 | MIT/Apache-2.0 | Yes | github.com/bitflags/bitflags |
| cargo | `bitflags` | 2.13.2 | MIT OR Apache-2.0 | Yes | github.com/bitflags/bitflags |
| cargo | `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/utils |
| cargo | `block-buffer` | 0.12.1 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/utils |
| cargo | `block2` | 0.6.2 | MIT | No | github.com/madsmtm/objc2 |
| cargo | `brotli` | 9.0.0 | BSD-3-Clause AND MIT | Yes | github.com/dropbox/rust-brotli |
| cargo | `brotli-decompressor` | 6.0.1 | BSD-3-Clause/MIT | Yes | github.com/dropbox/rust-brotli-decompressor |
| cargo | `bs58` | 0.5.1 | MIT/Apache-2.0 | Yes | github.com/nullus157/bs58-rs |
| cargo | `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | No | github.com/fitzgen/bumpalo |
| cargo | `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | Yes | github.com/lokathor/bytemuck |
| cargo | `byteorder` | 1.5.0 | Unlicense OR MIT | Yes | github.com/burntsushi/byteorder |
| cargo | `byteorder-lite` | 0.1.0 | Unlicense OR MIT | Yes | github.com/image-rs/byteorder-lite |
| cargo | `bytes` | 1.12.1 | MIT | Yes | github.com/tokio-rs/bytes |
| cargo | `cairo-rs` | 0.18.5 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `cairo-sys-rs` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `camino` | 1.2.6 | MIT OR Apache-2.0 | Yes | github.com/camino-rs/camino |
| cargo | `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/cargo |
| cargo | `cargo_metadata` | 0.19.2 | MIT | Yes | github.com/oli-obk/cargo_metadata |
| cargo | `cargo_toml` | 1.0.1 | Apache-2.0 OR MIT | No | gitlab.com/lib.rs/cargo_toml |
| cargo | `cc` | 1.4.7 | MIT OR Apache-2.0 | No | github.com/rust-lang/cc-rs |
| cargo | `cesu8` | 1.1.0 | Apache-2.0/MIT | No | github.com/emk/cesu8-rs |
| cargo | `cfb` | 0.14.0 | MIT | Yes | github.com/mdsteele/rust-cfb |
| cargo | `cfg-expr` | 0.15.8 | MIT OR Apache-2.0 | No | github.com/embarkstudios/cfg-expr |
| cargo | `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/cfg-if |
| cargo | `cfg_aliases` | 0.2.2 | MIT | No | github.com/katharostech/cfg_aliases |
| cargo | `chacha20` | 0.10.2 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/stream-ciphers |
| cargo | `chrono` | 0.4.45 | MIT OR Apache-2.0 | Yes | github.com/chronotope/chrono |
| cargo | `cmake` | 0.1.58 | MIT OR Apache-2.0 | No | github.com/rust-lang/cmake-rs |
| cargo | `color_quant` | 1.1.0 | MIT | Yes | github.com/image-rs/color_quant |
| cargo | `combine` | 4.6.8 | MIT | No | github.com/marwes/combine |
| cargo | `const-oid` | 0.10.2 | Apache-2.0 OR MIT | Yes | github.com/rustcrypto/formats |
| cargo | `cookie` | 0.18.2 | MIT OR Apache-2.0 | Yes | github.com/sergiobenitez/cookie-rs |
| cargo | `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | No | github.com/servo/core-foundation-rs |
| cargo | `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | No | github.com/servo/core-foundation-rs |
| cargo | `core-graphics` | 0.25.0 | MIT OR Apache-2.0 | No | github.com/servo/core-foundation-rs |
| cargo | `core-graphics-types` | 0.2.0 | MIT OR Apache-2.0 | No | github.com/servo/core-foundation-rs |
| cargo | `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/utils |
| cargo | `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/utils |
| cargo | `crc32fast` | 1.5.2 | MIT OR Apache-2.0 | Yes | github.com/srijs/rust-crc32fast |
| cargo | `crossbeam-channel` | 0.5.17 | MIT OR Apache-2.0 | Yes | github.com/crossbeam-rs/crossbeam |
| cargo | `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | Yes | github.com/crossbeam-rs/crossbeam |
| cargo | `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/traits |
| cargo | `crypto-common` | 0.2.2 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/traits |
| cargo | `cssparser` | 0.37.0 | MPL-2.0 | Yes | github.com/servo/rust-cssparser |
| cargo | `cssparser-macros` | 0.7.1 | MPL-2.0 | Yes | github.com/servo/rust-cssparser |
| cargo | `ctor` | 1.0.13 | Apache-2.0 OR MIT | Yes | github.com/mmastrac/linktime |
| cargo | `darling` | 0.24.1 | MIT | Yes | github.com/teddriggs/darling |
| cargo | `darling_core` | 0.24.1 | MIT | Yes | github.com/teddriggs/darling |
| cargo | `darling_macro` | 0.24.1 | MIT | Yes | github.com/teddriggs/darling |
| cargo | `dbus` | 0.9.12 | Apache-2.0/MIT | No | github.com/diwic/dbus-rs |
| cargo | `defmt` | 1.1.1 | MIT OR Apache-2.0 | Yes | github.com/knurling-rs/defmt |
| cargo | `defmt-macros` | 1.1.1 | MIT OR Apache-2.0 | Yes | github.com/knurling-rs/defmt |
| cargo | `defmt-parser` | 1.0.0 | MIT OR Apache-2.0 | Yes | github.com/knurling-rs/defmt |
| cargo | `deranged` | 0.5.8 | MIT OR Apache-2.0 | Yes | github.com/jhpratt/deranged |
| cargo | `derive_more` | 2.1.1 | MIT | Yes | github.com/jeltef/derive_more |
| cargo | `derive_more-impl` | 2.1.1 | MIT | Yes | github.com/jeltef/derive_more |
| cargo | `digest` | 0.10.7 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/traits |
| cargo | `digest` | 0.11.3 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/traits |
| cargo | `dirs` | 7.0.0 | MIT OR Apache-2.0 | Yes | codeberg.org/dirs/dirs-rs |
| cargo | `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | Yes | github.com/dirs-dev/dirs-sys-rs |
| cargo | `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | Yes | github.com/yaahc/displaydoc |
| cargo | `dlopen2` | 0.8.2 | MIT | No | github.com/openbytedev/dlopen2 |
| cargo | `dlopen2_derive` | 0.4.3 | MIT | No | github.com/openbytedev/dlopen2 |
| cargo | `dom_query` | 0.28.0 | MIT | Yes | github.com/niklak/dom_query |
| cargo | `dpi` | 0.1.2 | Apache-2.0 AND MIT | Yes | github.com/rust-windowing/winit |
| cargo | `dtoa` | 1.0.11 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/dtoa |
| cargo | `dtoa-short` | 0.3.5 | MPL-2.0 | Yes | github.com/upsuper/dtoa-short |
| cargo | `dunce` | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | Yes | gitlab.com/kornelski/dunce |
| cargo | `dyn-clone` | 1.0.20 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/dyn-clone |
| cargo | `embed-resource` | 3.0.11 | MIT | No | github.com/nabijaczleweli/rust-embed-resource |
| cargo | `embed_plist` | 1.2.2 | MIT OR Apache-2.0 | No | github.com/nvzqz/embed-plist-rs |
| cargo | `equivalent` | 1.0.2 | Apache-2.0 OR MIT | Yes | github.com/indexmap-rs/equivalent |
| cargo | `erased-serde` | 0.4.10 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/erased-serde |
| cargo | `fastrand` | 2.5.0 | Apache-2.0 OR MIT | Yes | github.com/smol-rs/fastrand |
| cargo | `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | Yes | github.com/image-rs/fdeflate |
| cargo | `field-offset` | 0.3.6 | MIT OR Apache-2.0 | No | github.com/diggsey/rust-field-offset |
| cargo | `find-msvc-tools` | 0.1.13 | MIT OR Apache-2.0 | No | github.com/rust-lang/cc-rs |
| cargo | `flate2` | 1.1.10 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/flate2-rs |
| cargo | `fnv` | 1.0.7 | Apache-2.0 / MIT | Yes | github.com/servo/rust-fnv |
| cargo | `foldhash` | 0.2.0 | Zlib | Yes | github.com/orlp/foldhash |
| cargo | `foreign-types` | 0.5.0 | MIT/Apache-2.0 | No | github.com/sfackler/foreign-types |
| cargo | `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | No | github.com/sfackler/foreign-types |
| cargo | `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | No | github.com/sfackler/foreign-types |
| cargo | `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | Yes | github.com/servo/rust-url |
| cargo | `fs_extra` | 1.3.0 | MIT | No | github.com/webdesus/fs_extra |
| cargo | `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-core` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-executor` | 0.3.34 | MIT OR Apache-2.0 | No | github.com/rust-lang/futures-rs |
| cargo | `futures-io` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-task` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `futures-util` | 0.3.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/futures-rs |
| cargo | `gdk` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gdk-pixbuf` | 0.18.5 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `gdk-pixbuf-sys` | 0.18.0 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `gdk-sys` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gdkwayland-sys` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gdkx11` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gdkx11-sys` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `generic-array` | 0.14.7 | MIT | Yes | github.com/fizyk20/generic-array |
| cargo | `getrandom` | 0.2.17 | MIT OR Apache-2.0 | Yes | github.com/rust-random/getrandom |
| cargo | `getrandom` | 0.3.4 | MIT OR Apache-2.0 | Yes | github.com/rust-random/getrandom |
| cargo | `getrandom` | 0.4.3 | MIT OR Apache-2.0 | Yes | github.com/rust-random/getrandom |
| cargo | `gif` | 0.14.2 | MIT OR Apache-2.0 | Yes | github.com/image-rs/image-gif |
| cargo | `gio` | 0.18.4 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `gio-sys` | 0.18.1 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `glib` | 0.18.5 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `glib-macros` | 0.18.5 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `glib-sys` | 0.18.1 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `glob` | 0.3.4 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/glob |
| cargo | `gobject-sys` | 0.18.0 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `gtk` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gtk-sys` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `gtk3-macros` | 0.18.2 | MIT | No | github.com/gtk-rs/gtk3-rs |
| cargo | `hashbrown` | 0.12.3 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/hashbrown |
| cargo | `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/hashbrown |
| cargo | `heck` | 0.4.1 | MIT OR Apache-2.0 | No | github.com/withoutboats/heck |
| cargo | `heck` | 0.5.0 | MIT OR Apache-2.0 | Yes | github.com/withoutboats/heck |
| cargo | `hex` | 0.4.3 | MIT OR Apache-2.0 | Yes | github.com/kokakiwi/rust-hex |
| cargo | `html5ever` | 0.39.0 | MIT OR Apache-2.0 | Yes | github.com/servo/html5ever |
| cargo | `http` | 1.5.0 | MIT OR Apache-2.0 | Yes | github.com/hyperium/http |
| cargo | `http-body` | 1.1.0 | MIT | Yes | github.com/hyperium/http-body |
| cargo | `http-body-util` | 0.1.5 | MIT | Yes | github.com/hyperium/http-body |
| cargo | `httparse` | 1.10.1 | MIT OR Apache-2.0 | Yes | github.com/seanmonstar/httparse |
| cargo | `hybrid-array` | 0.4.15 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/hybrid-array |
| cargo | `hyper` | 1.11.1 | MIT | Yes | github.com/hyperium/hyper |
| cargo | `hyper-rustls` | 0.27.10 | Apache-2.0 OR ISC OR MIT | Yes | github.com/rustls/hyper-rustls |
| cargo | `hyper-util` | 0.1.21 | MIT | Yes | github.com/hyperium/hyper-util |
| cargo | `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | No | github.com/strawlab/iana-time-zone |
| cargo | `iana-time-zone-haiku` | 0.1.2 | MIT OR Apache-2.0 | No | github.com/strawlab/iana-time-zone |
| cargo | `ico` | 0.5.0 | MIT | Yes | github.com/mdsteele/rust-ico |
| cargo | `icu_collections` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_locale_core` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_normalizer` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_normalizer_data` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_properties` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_properties_data` | 2.3.0 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `icu_provider` | 2.3.1 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `ident_case` | 1.0.1 | MIT/Apache-2.0 | Yes | github.com/teddriggs/ident_case |
| cargo | `idna` | 1.1.0 | MIT OR Apache-2.0 | Yes | github.com/servo/rust-url |
| cargo | `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | Yes | github.com/hsivonen/idna_adapter |
| cargo | `image` | 0.25.10 | MIT OR Apache-2.0 | Yes | github.com/image-rs/image |
| cargo | `indexmap` | 1.9.3 | Apache-2.0 OR MIT | Yes | github.com/bluss/indexmap |
| cargo | `indexmap` | 2.14.2 | Apache-2.0 OR MIT | Yes | github.com/indexmap-rs/indexmap |
| cargo | `infer` | 0.22.0 | MIT | Yes | github.com/bojand/infer |
| cargo | `ipnet` | 2.12.2 | MIT OR Apache-2.0 | Yes | github.com/krisprice/ipnet |
| cargo | `itoa` | 1.0.18 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/itoa |
| cargo | `javascriptcore-rs` | 1.1.2 | MIT | No | github.com/tauri-apps/javascriptcore-rs |
| cargo | `javascriptcore-rs-sys` | 1.1.1 | MIT | No | github.com/tauri-apps/javascriptcore-rs |
| cargo | `jiff` | 0.2.37 | Unlicense OR MIT | Yes | github.com/burntsushi/jiff |
| cargo | `jiff-core` | 0.1.1 | Unlicense OR MIT | Yes | github.com/burntsushi/jiff |
| cargo | `jiff-static` | 0.2.37 | Unlicense OR MIT | No | github.com/burntsushi/jiff |
| cargo | `jiff-tzdb` | 0.1.8 | Unlicense OR MIT | Yes | github.com/burntsushi/jiff |
| cargo | `jiff-tzdb-platform` | 0.1.3 | Unlicense OR MIT | Yes | github.com/burntsushi/jiff |
| cargo | `jni` | 0.21.1 | MIT/Apache-2.0 | No | github.com/jni-rs/jni-rs |
| cargo | `jni` | 0.22.4 | MIT OR Apache-2.0 | No | github.com/jni-rs/jni-rs |
| cargo | `jni-macros` | 0.22.4 | MIT OR Apache-2.0 | No | github.com/jni-rs/jni-rs |
| cargo | `jni-sys` | 0.3.1 | MIT OR Apache-2.0 | No | github.com/jni-rs/jni-sys |
| cargo | `jni-sys` | 0.4.1 | MIT OR Apache-2.0 | No | github.com/jni-rs/jni-sys |
| cargo | `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 | No | github.com/jni-rs/jni-sys |
| cargo | `jobserver` | 0.1.35 | MIT OR Apache-2.0 | No | github.com/rust-lang/jobserver-rs |
| cargo | `js-sys` | 0.3.105 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys |
| cargo | `json-patch` | 4.2.0 | MIT/Apache-2.0 | Yes | github.com/idubrov/json-patch |
| cargo | `jsonptr` | 0.7.1 | MIT OR Apache-2.0 | Yes | github.com/chanced/jsonptr |
| cargo | `keyboard-types` | 0.8.3 | MIT OR Apache-2.0 | Yes | github.com/rust-windowing/keyboard-types |
| cargo | `libappindicator` | 0.9.0 | Apache-2.0 OR MIT | No | registry+https://github.com/rust-lang/crates.io-index |
| cargo | `libappindicator-sys` | 0.9.0 | Apache-2.0 OR MIT | No | registry+https://github.com/rust-lang/crates.io-index |
| cargo | `libc` | 0.2.189 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/libc |
| cargo | `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | No | github.com/diwic/dbus-rs |
| cargo | `libloading` | 0.7.4 | ISC | No | github.com/nagisa/rust_libloading |
| cargo | `libredox` | 0.1.25 | MIT | No | gitlab.redox-os.org/redox-os/libredox |
| cargo | `litemap` | 0.8.3 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `lock_api` | 0.4.14 | MIT OR Apache-2.0 | Yes | github.com/amanieu/parking_lot |
| cargo | `log` | 0.4.34 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/log |
| cargo | `lru-slab` | 0.1.3 | MIT OR Apache-2.0 OR Zlib | Yes | github.com/ralith/lru-slab |
| cargo | `markup5ever` | 0.39.0 | MIT OR Apache-2.0 | Yes | github.com/servo/html5ever |
| cargo | `memchr` | 2.8.3 | Unlicense OR MIT | Yes | github.com/burntsushi/memchr |
| cargo | `memoffset` | 0.9.1 | MIT | No | github.com/gilnaa/memoffset |
| cargo | `mime` | 0.3.17 | MIT OR Apache-2.0 | Yes | github.com/hyperium/mime |
| cargo | `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | Yes | github.com/frommi/miniz_oxide/tree/master/miniz_oxide |
| cargo | `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | Yes | github.com/frommi/miniz_oxide/tree/master/miniz_oxide |
| cargo | `mio` | 1.2.3 | MIT | Yes | github.com/tokio-rs/mio |
| cargo | `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 | Yes | github.com/awxkee/moxcms |
| cargo | `muda` | 0.20.0 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/muda |
| cargo | `ndk` | 0.9.0 | MIT OR Apache-2.0 | No | github.com/rust-mobile/ndk |
| cargo | `ndk-context` | 0.1.1 | MIT OR Apache-2.0 | No | github.com/rust-windowing/android-ndk-rs |
| cargo | `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 | No | github.com/rust-mobile/ndk |
| cargo | `new_debug_unreachable` | 1.0.6 | MIT | Yes | github.com/mbrubeck/rust-debug-unreachable |
| cargo | `num-conv` | 0.2.2 | MIT OR Apache-2.0 | Yes | github.com/jhpratt/num-conv |
| cargo | `num-traits` | 0.2.19 | MIT OR Apache-2.0 | Yes | github.com/rust-num/num-traits |
| cargo | `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | No | github.com/illicitonion/num_enum |
| cargo | `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | No | github.com/illicitonion/num_enum |
| cargo | `objc2` | 0.6.4 | MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-location` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-encode` | 4.1.0 | MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-exception-helper` | 0.1.1 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-foundation` | 0.3.2 | MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-user-notifications` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `objc2-web-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | No | github.com/madsmtm/objc2 |
| cargo | `once_cell` | 1.21.4 | MIT OR Apache-2.0 | Yes | github.com/matklad/once_cell |
| cargo | `openssl-probe` | 0.2.1 | MIT OR Apache-2.0 | No | github.com/rustls/openssl-probe |
| cargo | `option-ext` | 0.2.0 | MPL-2.0 | Yes | github.com/soc/option-ext |
| cargo | `pango` | 0.18.3 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `pango-sys` | 0.18.0 | MIT | No | github.com/gtk-rs/gtk-rs-core |
| cargo | `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | Yes | github.com/amanieu/parking_lot |
| cargo | `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | Yes | github.com/amanieu/parking_lot |
| cargo | `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | Yes | github.com/servo/rust-url |
| cargo | `phf` | 0.13.1 | MIT | Yes | github.com/rust-phf/rust-phf |
| cargo | `phf_codegen` | 0.13.1 | MIT | No | github.com/rust-phf/rust-phf |
| cargo | `phf_generator` | 0.13.1 | MIT | Yes | github.com/rust-phf/rust-phf |
| cargo | `phf_macros` | 0.13.1 | MIT | Yes | github.com/rust-phf/rust-phf |
| cargo | `phf_shared` | 0.13.1 | MIT | Yes | github.com/rust-phf/rust-phf |
| cargo | `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | Yes | github.com/taiki-e/pin-project-lite |
| cargo | `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | No | github.com/rust-lang/pkg-config-rs |
| cargo | `plist` | 1.10.1 | MIT | Yes | github.com/ebarnard/rust-plist |
| cargo | `png` | 0.17.16 | MIT OR Apache-2.0 | Yes | github.com/image-rs/image-png |
| cargo | `png` | 0.18.1 | MIT OR Apache-2.0 | Yes | github.com/image-rs/image-png |
| cargo | `portable-atomic` | 1.15.0 | Apache-2.0 OR MIT | No | github.com/taiki-e/portable-atomic |
| cargo | `portable-atomic-util` | 0.2.8 | Apache-2.0 OR MIT | No | github.com/taiki-e/portable-atomic-util |
| cargo | `potential_utf` | 0.1.6 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `powerfmt` | 0.2.0 | MIT OR Apache-2.0 | Yes | github.com/jhpratt/powerfmt |
| cargo | `precomputed-hash` | 0.1.1 | MIT | Yes | github.com/emilio/precomputed-hash |
| cargo | `proc-macro-crate` | 1.3.1 | MIT OR Apache-2.0 | No | github.com/bkchr/proc-macro-crate |
| cargo | `proc-macro-crate` | 2.0.2 | MIT OR Apache-2.0 | No | github.com/bkchr/proc-macro-crate |
| cargo | `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | No | github.com/bkchr/proc-macro-crate |
| cargo | `proc-macro-error` | 1.0.4 | MIT OR Apache-2.0 | No | gitlab.com/creepyskeleton/proc-macro-error |
| cargo | `proc-macro-error-attr` | 1.0.4 | MIT OR Apache-2.0 | No | gitlab.com/creepyskeleton/proc-macro-error |
| cargo | `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/proc-macro2 |
| cargo | `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 | Yes | github.com/awxkee/pxfm |
| cargo | `quick-xml` | 0.42.0 | MIT | Yes | github.com/tafia/quick-xml |
| cargo | `quinn` | 0.11.12 | MIT OR Apache-2.0 | Yes | github.com/quinn-rs/quinn |
| cargo | `quinn-proto` | 0.11.18 | MIT OR Apache-2.0 | Yes | github.com/quinn-rs/quinn |
| cargo | `quinn-udp` | 0.5.15 | MIT OR Apache-2.0 | Yes | github.com/quinn-rs/quinn |
| cargo | `quote` | 1.0.47 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/quote |
| cargo | `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | No | github.com/r-efi/r-efi |
| cargo | `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | No | github.com/r-efi/r-efi |
| cargo | `rand` | 0.10.3 | MIT OR Apache-2.0 | Yes | github.com/rust-random/rand |
| cargo | `rand_core` | 0.10.1 | MIT OR Apache-2.0 | Yes | github.com/rust-random/rand_core |
| cargo | `rand_pcg` | 0.10.2 | MIT OR Apache-2.0 | Yes | github.com/rust-random/rngs |
| cargo | `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | Yes | github.com/rust-windowing/raw-window-handle |
| cargo | `redox_syscall` | 0.5.18 | MIT | No | gitlab.redox-os.org/redox-os/syscall |
| cargo | `redox_users` | 0.5.3 | MIT | No | gitlab.redox-os.org/redox-os/users |
| cargo | `ref-cast` | 1.0.27 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/ref-cast |
| cargo | `ref-cast-impl` | 1.0.27 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/ref-cast |
| cargo | `regex` | 1.13.1 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/regex |
| cargo | `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/regex |
| cargo | `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/regex |
| cargo | `reqwest` | 0.13.5 | MIT OR Apache-2.0 | Yes | github.com/seanmonstar/reqwest |
| cargo | `rfd` | 0.17.2 | MIT | Yes | github.com/polymeilex/rfd |
| cargo | `ring` | 0.17.14 | Apache-2.0 AND ISC | Yes | github.com/briansmith/ring |
| cargo | `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | Yes | github.com/rust-lang/rustc-hash |
| cargo | `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | No | github.com/djc/rustc-version-rs |
| cargo | `rustls` | 0.23.46 | Apache-2.0 OR ISC OR MIT | Yes | github.com/rustls/rustls |
| cargo | `rustls-native-certs` | 0.8.4 | Apache-2.0 OR ISC OR MIT | No | github.com/rustls/rustls-native-certs |
| cargo | `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | Yes | github.com/rustls/pki-types |
| cargo | `rustls-platform-verifier` | 0.7.1 | MIT OR Apache-2.0 | Yes | github.com/rustls/rustls-platform-verifier |
| cargo | `rustls-platform-verifier-android` | 0.2.0 | MIT OR Apache-2.0 | No | github.com/rustls/rustls-platform-verifier |
| cargo | `rustls-webpki` | 0.103.16 | ISC | Yes | github.com/rustls/webpki |
| cargo | `rustversion` | 1.0.23 | MIT OR Apache-2.0 | No | github.com/dtolnay/rustversion |
| cargo | `same-file` | 1.0.6 | Unlicense/MIT | Yes | github.com/burntsushi/same-file |
| cargo | `schannel` | 0.1.29 | MIT | No | github.com/steffengy/schannel-rs |
| cargo | `schemars` | 0.8.22 | MIT | Yes | github.com/gresau/schemars |
| cargo | `schemars` | 0.9.0 | MIT | Yes | github.com/gresau/schemars |
| cargo | `schemars` | 1.2.2 | MIT | Yes | github.com/gresau/schemars |
| cargo | `schemars_derive` | 0.8.22 | MIT | Yes | github.com/gresau/schemars |
| cargo | `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | Yes | github.com/bluss/scopeguard |
| cargo | `security-framework` | 3.7.0 | MIT OR Apache-2.0 | No | github.com/kornelski/rust-security-framework |
| cargo | `security-framework-sys` | 2.17.0 | MIT OR Apache-2.0 | No | github.com/kornelski/rust-security-framework |
| cargo | `selectors` | 0.38.0 | MPL-2.0 | Yes | github.com/servo/stylo |
| cargo | `semver` | 1.0.28 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/semver |
| cargo | `serde` | 1.0.229 | MIT OR Apache-2.0 | Yes | github.com/serde-rs/serde |
| cargo | `serde-untagged` | 0.1.9 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/serde-untagged |
| cargo | `serde_core` | 1.0.229 | MIT OR Apache-2.0 | Yes | github.com/serde-rs/serde |
| cargo | `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | Yes | github.com/serde-rs/serde |
| cargo | `serde_derive_internals` | 0.29.1 | MIT OR Apache-2.0 | Yes | github.com/serde-rs/serde |
| cargo | `serde_json` | 1.0.154 | MIT OR Apache-2.0 | Yes | github.com/serde-rs/json |
| cargo | `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/serde-repr |
| cargo | `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `serde_spanned` | 1.1.1 | MIT OR Apache-2.0 | Yes | github.com/toml-rs/toml |
| cargo | `serde_with` | 3.23.0 | MIT OR Apache-2.0 | Yes | github.com/jonasbb/serde_with |
| cargo | `serde_with_macros` | 3.23.0 | MIT OR Apache-2.0 | Yes | github.com/jonasbb/serde_with |
| cargo | `serialize-to-javascript` | 0.1.2 | MIT OR Apache-2.0 | Yes | github.com/chippers/serialize-to-javascript |
| cargo | `serialize-to-javascript-impl` | 0.1.2 | MIT OR Apache-2.0 | Yes | github.com/chippers/serialize-to-javascript |
| cargo | `servo_arc` | 0.4.3 | MIT OR Apache-2.0 | Yes | github.com/servo/stylo |
| cargo | `sha2` | 0.10.9 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/hashes |
| cargo | `sha2` | 0.11.1 | MIT OR Apache-2.0 | Yes | github.com/rustcrypto/hashes |
| cargo | `shlex` | 2.0.1 | MIT OR Apache-2.0 | No | github.com/comex/rust-shlex |
| cargo | `simd-adler32` | 0.3.10 | MIT | Yes | github.com/mcountryman/simd-adler32 |
| cargo | `simd_cesu8` | 1.2.0 | Apache-2.0 OR MIT | No | github.com/seancroach/simd_cesu8 |
| cargo | `simdutf8` | 0.1.5 | MIT OR Apache-2.0 | No | github.com/rusticstuff/simdutf8 |
| cargo | `siphasher` | 1.0.3 | MIT/Apache-2.0 | Yes | github.com/jedisct1/rust-siphash |
| cargo | `slab` | 0.4.12 | MIT | Yes | github.com/tokio-rs/slab |
| cargo | `smallvec` | 1.16.1 | MIT OR Apache-2.0 | Yes | github.com/servo/rust-smallvec |
| cargo | `socket2` | 0.6.5 | MIT OR Apache-2.0 | Yes | github.com/rust-lang/socket2 |
| cargo | `softbuffer` | 0.4.8 | MIT OR Apache-2.0 | Yes | github.com/rust-windowing/softbuffer |
| cargo | `soup3` | 0.5.0 | MIT | No | gitlab.gnome.org/world/rust/soup3-rs |
| cargo | `soup3-sys` | 0.5.0 | MIT | No | gitlab.gnome.org/world/rust/soup3-rs |
| cargo | `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | Yes | github.com/storyyeller/stable_deref_trait |
| cargo | `string_cache` | 0.9.0 | MIT OR Apache-2.0 | Yes | github.com/servo/string-cache |
| cargo | `string_cache_codegen` | 0.6.1 | MIT OR Apache-2.0 | No | github.com/servo/string-cache |
| cargo | `strsim` | 0.11.1 | MIT | Yes | github.com/rapidfuzz/strsim-rs |
| cargo | `subtle` | 2.6.1 | BSD-3-Clause | Yes | github.com/dalek-cryptography/subtle |
| cargo | `swift-rs` | 1.0.8 | MIT OR Apache-2.0 | No | github.com/brendonovich/swift-rs |
| cargo | `syn` | 1.0.109 | MIT OR Apache-2.0 | No | github.com/dtolnay/syn |
| cargo | `syn` | 2.0.119 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/syn |
| cargo | `syn` | 3.0.6 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/syn |
| cargo | `sync_wrapper` | 1.0.2 | Apache-2.0 | Yes | github.com/actyx/sync_wrapper |
| cargo | `synstructure` | 0.14.0 | MIT | Yes | github.com/mystor/synstructure |
| cargo | `system-deps` | 6.2.2 | MIT OR Apache-2.0 | No | github.com/gdesmott/system-deps |
| cargo | `tao` | 0.37.1 | Apache-2.0 | Yes | github.com/tauri-apps/tao |
| cargo | `tao-macros` | 0.1.4 | MIT OR Apache-2.0 | No | github.com/tauri-apps/tao |
| cargo | `target-lexicon` | 0.12.16 | Apache-2.0 WITH LLVM-exception | No | github.com/bytecodealliance/target-lexicon |
| cargo | `tauri` | 2.12.2 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-build` | 2.7.2 | Apache-2.0 OR MIT | No | github.com/tauri-apps/tauri |
| cargo | `tauri-codegen` | 2.7.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-macros` | 2.7.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-plugin` | 2.7.1 | Apache-2.0 OR MIT | No | github.com/tauri-apps/tauri |
| cargo | `tauri-plugin-dialog` | 2.8.4 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/plugins-workspace |
| cargo | `tauri-plugin-fs` | 2.7.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/plugins-workspace |
| cargo | `tauri-runtime` | 2.12.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-runtime-wry` | 2.12.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-utils` | 2.10.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| cargo | `tauri-winres` | 0.3.6 | MIT | No | github.com/tauri-apps/winres |
| cargo | `tendril` | 0.5.1 | MIT OR Apache-2.0 | Yes | github.com/servo/html5ever |
| cargo | `thiserror` | 1.0.69 | MIT OR Apache-2.0 | No | github.com/dtolnay/thiserror |
| cargo | `thiserror` | 2.0.21 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/thiserror |
| cargo | `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | No | github.com/dtolnay/thiserror |
| cargo | `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/thiserror |
| cargo | `time` | 0.3.55 | MIT OR Apache-2.0 | Yes | github.com/time-rs/time |
| cargo | `time-core` | 0.1.9 | MIT OR Apache-2.0 | Yes | github.com/time-rs/time |
| cargo | `time-macros` | 0.2.32 | MIT OR Apache-2.0 | Yes | github.com/time-rs/time |
| cargo | `tinystr` | 0.8.4 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT | Yes | github.com/lokathor/tinyvec |
| cargo | `tokio` | 1.54.1 | MIT | Yes | github.com/tokio-rs/tokio |
| cargo | `tokio-macros` | 2.7.2 | MIT | Yes | github.com/tokio-rs/tokio |
| cargo | `tokio-rustls` | 0.26.5 | MIT OR Apache-2.0 | Yes | github.com/rustls/tokio-rustls |
| cargo | `tokio-util` | 0.7.19 | MIT | Yes | github.com/tokio-rs/tokio |
| cargo | `toml` | 0.8.2 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `toml` | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 | Yes | github.com/toml-rs/toml |
| cargo | `toml_datetime` | 0.6.3 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `toml_datetime` | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | Yes | github.com/toml-rs/toml |
| cargo | `toml_edit` | 0.19.15 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `toml_edit` | 0.20.2 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `toml_edit` | 0.25.15+spec-1.1.0 | MIT OR Apache-2.0 | No | github.com/toml-rs/toml |
| cargo | `toml_parser` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | Yes | github.com/toml-rs/toml |
| cargo | `toml_writer` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | Yes | github.com/toml-rs/toml |
| cargo | `tower` | 0.5.3 | MIT | Yes | github.com/tower-rs/tower |
| cargo | `tower-http` | 0.6.11 | MIT | Yes | github.com/tower-rs/tower-http |
| cargo | `tower-layer` | 0.3.3 | MIT | Yes | github.com/tower-rs/tower |
| cargo | `tower-service` | 0.3.3 | MIT | Yes | github.com/tower-rs/tower |
| cargo | `tracing` | 0.1.44 | MIT | Yes | github.com/tokio-rs/tracing |
| cargo | `tracing-core` | 0.1.36 | MIT | Yes | github.com/tokio-rs/tracing |
| cargo | `tray-icon` | 0.25.1 | MIT OR Apache-2.0 | Yes | github.com/tauri-apps/tray-icon |
| cargo | `try-lock` | 0.2.5 | MIT | Yes | github.com/seanmonstar/try-lock |
| cargo | `typeid` | 1.0.3 | MIT OR Apache-2.0 | Yes | github.com/dtolnay/typeid |
| cargo | `typenum` | 1.20.1 | MIT OR Apache-2.0 | Yes | github.com/paholg/typenum |
| cargo | `unicode-ident` | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | Yes | github.com/dtolnay/unicode-ident |
| cargo | `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | Yes | github.com/unicode-rs/unicode-segmentation |
| cargo | `untrusted` | 0.9.0 | ISC | Yes | github.com/briansmith/untrusted |
| cargo | `url` | 2.5.8 | MIT OR Apache-2.0 | Yes | github.com/servo/rust-url |
| cargo | `urlpattern` | 0.6.0 | MIT | Yes | github.com/denoland/rust-urlpattern |
| cargo | `utf8_iter` | 1.0.4 | Apache-2.0 OR MIT | Yes | github.com/hsivonen/utf8_iter |
| cargo | `uuid` | 1.26.1 | Apache-2.0 OR MIT | Yes | github.com/uuid-rs/uuid |
| cargo | `version-compare` | 0.2.1 | MIT | No | gitlab.com/timvisee/version-compare |
| cargo | `version_check` | 0.9.5 | MIT/Apache-2.0 | No | github.com/sergiobenitez/version_check |
| cargo | `vswhom` | 0.1.0 | MIT | No | github.com/nabijaczleweli/vswhom.rs |
| cargo | `vswhom-sys` | 0.1.3 | MIT | No | github.com/nabijaczleweli/vswhom-sys.rs |
| cargo | `walkdir` | 2.5.0 | Unlicense/MIT | Yes | github.com/burntsushi/walkdir |
| cargo | `want` | 0.3.1 | MIT | Yes | github.com/seanmonstar/want |
| cargo | `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | No | github.com/bytecodealliance/wasi |
| cargo | `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | No | github.com/bytecodealliance/wasi-rs |
| cargo | `wasm-bindgen` | 0.2.128 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen |
| cargo | `wasm-bindgen-futures` | 0.4.78 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures |
| cargo | `wasm-bindgen-macro` | 0.2.128 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro |
| cargo | `wasm-bindgen-macro-support` | 0.2.128 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support |
| cargo | `wasm-bindgen-shared` | 0.2.128 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared |
| cargo | `wasm-streams` | 0.5.0 | MIT OR Apache-2.0 | No | github.com/mattiasbuelens/wasm-streams |
| cargo | `web-sys` | 0.3.105 | MIT OR Apache-2.0 | No | github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys |
| cargo | `web-time` | 1.1.0 | MIT OR Apache-2.0 | Yes | github.com/daxpedda/web-time |
| cargo | `web_atoms` | 0.2.6 | MIT OR Apache-2.0 | Yes | github.com/servo/html5ever |
| cargo | `webkit2gtk` | 2.0.2 | MIT | No | github.com/tauri-apps/webkit2gtk-rs |
| cargo | `webkit2gtk-sys` | 2.0.2 | MIT | No | github.com/tauri-apps/webkit2gtk-rs |
| cargo | `webpki-root-certs` | 1.0.10 | CDLA-Permissive-2.0 | No | github.com/rustls/webpki-roots |
| cargo | `webview2-com` | 0.39.1 | MIT | Yes | github.com/wravery/webview2-rs |
| cargo | `webview2-com-macros` | 0.8.1 | MIT | Yes | github.com/wravery/webview2-rs |
| cargo | `webview2-com-sys` | 0.39.1 | MIT | Yes | github.com/wravery/webview2-rs |
| cargo | `weezl` | 0.1.12 | MIT OR Apache-2.0 | Yes | github.com/image-rs/weezl |
| cargo | `winapi` | 0.3.9 | MIT/Apache-2.0 | No | github.com/retep998/winapi-rs |
| cargo | `winapi-i686-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | No | github.com/retep998/winapi-rs |
| cargo | `winapi-util` | 0.1.11 | Unlicense OR MIT | Yes | github.com/burntsushi/winapi-util |
| cargo | `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | No | github.com/retep998/winapi-rs |
| cargo | `window-vibrancy` | 0.8.1 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri-plugin-vibrancy |
| cargo | `windows` | 0.62.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-collections` | 0.3.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-core` | 0.62.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-future` | 0.3.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-link` | 0.2.1 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-numerics` | 0.3.1 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-result` | 0.4.1 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-sys` | 0.45.0 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows-sys` | 0.59.0 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-targets` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows-threading` | 0.2.1 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows-version` | 0.1.7 | MIT OR Apache-2.0 | Yes | github.com/microsoft/windows-rs |
| cargo | `windows_aarch64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_aarch64_msvc` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_i686_gnu` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_i686_msvc` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_gnu` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_msvc` | 0.42.2 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | No | github.com/microsoft/windows-rs |
| cargo | `winnow` | 0.5.40 | MIT | No | github.com/winnow-rs/winnow |
| cargo | `winnow` | 1.0.4 | MIT | Yes | github.com/winnow-rs/winnow |
| cargo | `winreg` | 0.55.0 | MIT | No | github.com/gentoo90/winreg-rs |
| cargo | `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | No | github.com/bytecodealliance/wit-bindgen |
| cargo | `writeable` | 0.6.4 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `wry` | 0.57.0 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/wry |
| cargo | `x11` | 2.21.0 | MIT | No | github.com/altf02/x11-rs |
| cargo | `x11-dl` | 2.21.0 | MIT | No | github.com/altf02/x11-rs |
| cargo | `yoke` | 0.8.3 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `yoke-derive` | 0.8.4 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zerofrom` | 0.1.8 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zerofrom-derive` | 0.1.8 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zeroize` | 1.9.1 | Apache-2.0 OR MIT | Yes | github.com/rustcrypto/utils |
| cargo | `zerotrie` | 0.2.5 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zerovec` | 0.11.8 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zerovec-derive` | 0.11.6 | Unicode-3.0 | Yes | github.com/unicode-org/icu4x |
| cargo | `zlib-rs` | 0.6.8 | Zlib | Yes | github.com/trifectatechfoundation/zlib-rs |
| cargo | `zmij` | 1.0.23 | MIT | Yes | github.com/dtolnay/zmij |
| cargo | `zune-core` | 0.5.3 | MIT OR Apache-2.0 OR Zlib | Yes | github.com/etemesi254/zune-image |
| cargo | `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 OR Zlib | Yes | github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
| npm | `@asamuzakjp/css-color` | 7.1.9 | MIT | No | github.com/asamuzak/csscolor |
| npm | `@asamuzakjp/dom-selector` | 10.0.5 | MIT | No | github.com/asamuzak/domselector |
| npm | `@babel/code-frame` | 7.29.7 | MIT | No | github.com/babel/babel |
| npm | `@babel/helper-validator-identifier` | 7.29.7 | MIT | No | github.com/babel/babel |
| npm | `@babel/runtime` | 7.29.7 | MIT | No | github.com/babel/babel |
| npm | `@bramus/specificity` | 2.4.3 | MIT | No | github.com/bramus/specificity |
| npm | `@csstools/color-helpers` | 6.1.2 | MIT-0 | No | github.com/csstools/postcss-plugins |
| npm | `@csstools/css-calc` | 3.4.3 | MIT | No | github.com/csstools/postcss-plugins |
| npm | `@csstools/css-color-parser` | 4.2.6 | MIT | No | github.com/csstools/postcss-plugins |
| npm | `@csstools/css-parser-algorithms` | 4.0.2 | MIT | No | github.com/csstools/postcss-plugins |
| npm | `@csstools/css-syntax-patches-for-csstree` | 1.1.15 | MIT-0 | No | github.com/csstools/postcss-plugins |
| npm | `@csstools/css-tokenizer` | 4.0.2 | MIT | No | github.com/csstools/postcss-plugins |
| npm | `@esbuild/win32-x64` | 0.28.2 | MIT | No | github.com/evanw/esbuild |
| npm | `@exodus/bytes` | 1.16.0 | MIT | No | github.com/exodusoss/bytes |
| npm | `@jridgewell/resolve-uri` | 3.1.2 | MIT | No | github.com/jridgewell/resolve-uri |
| npm | `@jridgewell/sourcemap-codec` | 1.6.0 | MIT | No | github.com/jridgewell/sourcemaps |
| npm | `@jridgewell/trace-mapping` | 0.3.31 | MIT | No | github.com/jridgewell/sourcemaps |
| npm | `@oxc-project/types` | 0.156.0 | MIT | No | github.com/oxc-project/oxc |
| npm | `@rolldown/binding-win32-x64-msvc` | 1.2.16 | MIT | No | github.com/rolldown/rolldown |
| npm | `@rolldown/pluginutils` | 1.0.1 | MIT | No | github.com/rolldown/plugins |
| npm | `@tauri-apps/api` | 2.12.4 | Apache-2.0 OR MIT | Yes | github.com/tauri-apps/tauri |
| npm | `@tauri-apps/cli` | 2.12.4 | Apache-2.0 OR MIT | No | github.com/tauri-apps/tauri |
| npm | `@tauri-apps/cli-win32-x64-msvc` | 2.12.4 | Apache-2.0 OR MIT | No | github.com/tauri-apps/tauri |
| npm | `@testing-library/dom` | 10.4.2 | MIT | No | github.com/testing-library/dom-testing-library |
| npm | `@testing-library/react` | 16.3.3 | MIT | No | github.com/testing-library/react-testing-library |
| npm | `@testing-library/user-event` | 14.6.8 | MIT | No | github.com/testing-library/user-event |
| npm | `@types/aria-query` | 5.0.4 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@types/chai` | 5.2.3 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@types/deep-eql` | 4.0.2 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@types/estree` | 1.0.9 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@types/react` | 19.3.0 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@types/react-dom` | 19.3.0 | MIT | No | github.com/definitelytyped/definitelytyped |
| npm | `@typescript/typescript-win32-x64` | 7.0.2 | Apache-2.0 | No | github.com/microsoft/typescript |
| npm | `@vitest/mocker` | 5.0.5 | MIT | No | github.com/vitest-dev/vitest |
| npm | `@vitest/spy` | 5.0.5 | MIT | No | github.com/vitest-dev/vitest |
| npm | `ansi-regex` | 5.0.1 | MIT | No | chalk/ansi-regex |
| npm | `ansi-styles` | 5.2.0 | MIT | No | chalk/ansi-styles |
| npm | `aria-query` | 5.3.0 | Apache-2.0 | No | github.com/a11yance/aria-query |
| npm | `assertion-error` | 2.0.1 | MIT | No | github.com/chaijs/assertion-error |
| npm | `bidi-js` | 1.1.0 | MIT | No | github.com/lojjic/bidi-js |
| npm | `chai` | 6.3.0 | MIT | No | github.com/chaijs/chai |
| npm | `css-tree` | 3.2.1 | MIT | No | csstree/csstree |
| npm | `csstype` | 3.2.3 | MIT | No | github.com/frenic/csstype |
| npm | `data-urls` | 8.0.0 | MIT | No | github.com/jsdom/data-urls |
| npm | `decimal.js` | 10.6.0 | MIT | No | github.com/mikemcl/decimal.js |
| npm | `dequal` | 2.0.3 | MIT | No | lukeed/dequal |
| npm | `detect-libc` | 2.1.2 | Apache-2.0 | No | github.com/lovell/detect-libc |
| npm | `dom-accessibility-api` | 0.5.16 | MIT | No | github.com/eps1lon/dom-accessibility-api |
| npm | `entities` | 8.1.0 | BSD-2-Clause | No | github.com/fb55/entities |
| npm | `es-module-lexer` | 2.3.2 | MIT | No | github.com/guybedford/es-module-lexer |
| npm | `esbuild` | 0.28.2 | MIT | No | github.com/evanw/esbuild |
| npm | `estree-walker` | 3.0.3 | MIT | No | github.com/rich-harris/estree-walker |
| npm | `expect-type` | 1.4.0 | Apache-2.0 | No | github.com/mmkal/expect-type |
| npm | `fdir` | 6.5.0 | MIT | No | github.com/thecodrr/fdir |
| npm | `html-encoding-sniffer` | 7.0.0 | MIT | No | github.com/jsdom/html-encoding-sniffer |
| npm | `is-potential-custom-element-name` | 1.0.1 | MIT | No | github.com/mathiasbynens/is-potential-custom-element-name |
| npm | `js-tokens` | 4.0.0 | MIT | No | lydell/js-tokens |
| npm | `jsdom` | 30.1.3 | MIT | No | github.com/jsdom/jsdom |
| npm | `lightningcss` | 1.33.0 | MPL-2.0 | No | github.com/parcel-bundler/lightningcss |
| npm | `lightningcss-win32-x64-msvc` | 1.33.0 | MPL-2.0 | No | github.com/parcel-bundler/lightningcss |
| npm | `lru-cache` | 11.5.3 | BlueOak-1.0.0 | No | github.com/isaacs/node-lru-cache |
| npm | `lz-string` | 1.5.0 | MIT | No | github.com/pieroxy/lz-string |
| npm | `magic-string` | 1.4.3 | MIT | No | github.com/rich-harris/magic-string |
| npm | `mdn-data` | 2.27.1 | CC0-1.0 | No | github.com/mdn/data |
| npm | `nanoid` | 3.3.20 | MIT | No | ai/nanoid |
| npm | `obug` | 3.0.0 | MIT | No | github.com/sxzz/obug |
| npm | `parse5` | 8.0.1 | MIT | No | github.com/inikulin/parse5 |
| npm | `picocolors` | 1.1.1 | ISC | No | alexeyraspopov/picocolors |
| npm | `picomatch` | 4.0.7 | MIT | No | micromatch/picomatch |
| npm | `postcss` | 8.5.32 | MIT | No | postcss/postcss |
| npm | `pretty-format` | 27.5.1 | MIT | No | github.com/facebook/jest |
| npm | `punycode` | 2.3.1 | MIT | No | github.com/mathiasbynens/punycode.js |
| npm | `react` | 19.3.1 | MIT | Yes | github.com/react/react |
| npm | `react-dom` | 19.3.1 | MIT | Yes | github.com/react/react |
| npm | `react-is` | 17.0.2 | MIT | No | github.com/facebook/react |
| npm | `require-from-string` | 2.0.2 | MIT | No | floatdrop/require-from-string |
| npm | `rolldown` | 1.2.16 | MIT | No | github.com/rolldown/rolldown |
| npm | `saxes` | 6.0.0 | ISC | No | github.com/lddubeau/saxes |
| npm | `scheduler` | 0.28.0 | MIT | Yes | github.com/react/react |
| npm | `source-map-js` | 1.2.2 | BSD-3-Clause | No | 7rulnik/source-map-js |
| npm | `std-env` | 4.3.0 | MIT | No | unjs/std-env |
| npm | `tinybench` | 6.2.2 | MIT | No | tinylibs/tinybench |
| npm | `tinyexec` | 1.3.1 | MIT | No | github.com/tinylibs/tinyexec |
| npm | `tinyglobby` | 0.2.17 | MIT | No | github.com/superchupudev/tinyglobby |
| npm | `tldts` | 7.4.19 | MIT | No | github.com/remusao/tldts |
| npm | `tldts-core` | 7.4.20 | MIT | No | github.com/remusao/tldts |
| npm | `tough-cookie` | 6.0.2 | BSD-3-Clause | No | github.com/salesforce/tough-cookie |
| npm | `tr46` | 7.0.0 | MIT | No | github.com/jsdom/tr46 |
| npm | `typescript` | 7.0.2 | Apache-2.0 | No | github.com/microsoft/typescript |
| npm | `undici` | 8.12.3 | MIT | No | github.com/nodejs/undici |
| npm | `vite` | 8.3.5 | MIT | No | github.com/vitejs/vite |
| npm | `vitest` | 5.0.5 | MIT | No | github.com/vitest-dev/vitest |
| npm | `w3c-xmlserializer` | 6.0.0 | MIT | No | github.com/jsdom/w3c-xmlserializer |
| npm | `webidl-conversions` | 8.0.1 | BSD-2-Clause | No | github.com/jsdom/webidl-conversions |
| npm | `whatwg-mimetype` | 5.0.0 | MIT | No | jsdom/whatwg-mimetype |
| npm | `whatwg-url` | 17.3.0 | MIT | No | github.com/jsdom/whatwg-url |
| npm | `why-is-node-running` | 3.2.1 | MIT | No | github.com/mafintosh/why-is-node-running |
| npm | `xml-name-validator` | 5.0.0 | Apache-2.0 | No | jsdom/xml-name-validator |
| npm | `xmlchars` | 2.2.0 | MIT | No | github.com/lddubeau/xmlchars |
