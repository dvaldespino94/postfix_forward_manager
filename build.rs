fn main() {
    if build_target::target_os().unwrap() == build_target::Os::Windows {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("app.ico")
            .set("InternalName", "PostfixForwardScript.exe")
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        res.compile().unwrap();
    }
}
