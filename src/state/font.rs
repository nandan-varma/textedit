pub fn load_system_font() -> anyhow::Result<Vec<u8>> {
    #[cfg(target_os = "macos")]
    {
        std::fs::read("/System/Library/Fonts/SFNSMono.ttf")
            .or_else(|_| std::fs::read("/System/Library/Fonts/Monaco.dfont"))
            .or_else(|_| std::fs::read("/System/Library/Fonts/Supplemental/Andale Mono.ttf"))
            .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf")
            .or_else(|_| {
                std::fs::read("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf")
            })
            .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
    }
    #[cfg(target_os = "windows")]
    {
        std::fs::read("C:\\Windows\\Fonts\\consola.ttf")
            .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\cour.ttf"))
            .map_err(|e| anyhow::anyhow!("Failed to load system font: {}", e))
    }
}
