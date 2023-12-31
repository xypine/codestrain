use extism::*;

fn main() {
    println!("Loading plugin...");
    let url =
        Wasm::url("https://github.com/extism/plugins/releases/latest/download/count_vowels.wasm");
    let manifest = Manifest::new([url]);
    let mut plugin = Plugin::new(&manifest, [], true).unwrap();
    println!("Plugin loaded!");
    let res = plugin
        .call::<&str, &str>("count_vowels", "Hello, world!")
        .unwrap();
    println!("{}", res);
}
