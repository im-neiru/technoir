use engine::plugin::package::Packager;

fn main() {
    let packager = Packager::release_mode();

    smol::block_on(packager.pack("./sandbox/wallpaper", None));
}
