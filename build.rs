fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("parvan.ico");
        res.set("ProductName", "Parvan Programming Language");
        res.set("FileDescription", "ParvanLang Compiler & Runtime");
        res.set("LegalCopyright", "Copyright (C) 2026 Parvan Team");
        if let Err(e) = res.compile() {
            eprintln!("Warning: could not compile windows icon: {}", e);
        }
    }
}
