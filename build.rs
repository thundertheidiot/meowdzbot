use std::env;

use std::path::Path;

use copy_dir::copy_dir;

fn main() {
    println!("cargo:rerun-if-changed=static");

    let repo = env::var("CARGO_MANIFEST_DIR").unwrap();
    let repo = Path::new(&repo);

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let assets = repo.join("static");
    let res = copy_dir(assets, out_dir.join("static"));
    println!("cargo:warning=Static status: {:?}", res);
}
