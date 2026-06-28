# api::presets_store

Renders the "store" page that lists installable preset services inside the
WebUI. `mod.rs` defines the `PresetMeta` and `PresetInfo` deserialisation
types and the `PresetCategory` enum; `category_names.rs` maps categories
to human-readable labels; `category_styling/` produces the inline CSS
class names per category; `renderer.rs` walks the on-disk preset JSON
files (optionally filtered by detected GPUs) and emits the full HTML
table.
