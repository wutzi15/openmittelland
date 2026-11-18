// The Windows exe carries the icon (icon/icon.ico, drawn from icon/icon.svg).
fn main() {
    println!("cargo:rerun-if-changed=icon/icon.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new().set_icon("icon/icon.ico").compile().expect("icon resource");
    }
}
