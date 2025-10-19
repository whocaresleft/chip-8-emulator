fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();

        #[cfg(feature = "debug-ver")]
        res.set_icon(".\\icons\\logo-debug.ico");

        #[cfg(feature = "release-ver")]
        res.set_icon(".\\icons\\logo-release.ico");

        res.compile().unwrap();
    }
}