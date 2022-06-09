fn main() {
    println!(r"cargo:rustc-link-search=/opt/homebrew/Cellar/sdl2/2.0.22/lib");
    println!(r"cargo:rustc-link-search=/opt/homebrew/Cellar/sdl2_image/2.0.5/lib");
    println!(r"cargo:rustc-link-search=/opt/homebrew/Cellar/sdl2_ttf/2.0.18_1/lib");
}