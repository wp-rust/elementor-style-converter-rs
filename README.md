# elementor-style-converter-rs

Rust port of [Elementor's PHP `css-converter` module](https://github.com/elementor/elementor) from the atomic widgets architecture.

Converts Elementor atomic widget CSS strings into typed `PropValue` trees. Part of the [`elementor-to-blocks`](https://github.com/bhubbard/elementor-to-blocks) WordPress content migration toolchain.

> **Not affiliated with or endorsed by Elementor Ltd.**

## Installation

```toml
[dependencies]
elementor-style-converter-rs = "0.1"
```

## Usage

```rust
use elementor_style_converter_rs::{CssConverter, ConverterRegistryFactory};

let registry = ConverterRegistryFactory::create_registry();
let expanders = ConverterRegistryFactory::create_expanders();
let converter = CssConverter::new(registry, expanders);

let result = converter.convert("color: red; padding: 10px 20px; font-size: 16px");
// result.props     — typed PropValues for each recognized declaration
// result.custom_css — unconverted declarations passed through as-is
// result.rejected  — incompatible declarations (e.g. animation)
```

## Related Crates

| Crate | Purpose |
|---|---|
| [`elementor-style-renderer-rs`](https://crates.io/crates/elementor-style-renderer-rs) | Renders PropValue trees back to CSS (the inverse) |
| [`wp-style-engine-rs`](https://crates.io/crates/wp-style-engine-rs) | Compiles Gutenberg block style objects to CSS |

## License

MIT — see [LICENSE](./LICENSE).
