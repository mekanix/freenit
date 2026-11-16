fn main() {
    // Download the Iconify sets used by `iconify_icon!` and stage them for compilation.
    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("lucide")
        .stage()
        .unwrap();
}
