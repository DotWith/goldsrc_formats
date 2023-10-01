use bsp_rs::read_bsp;

fn main() {
    let bytes = include_bytes!("catacombs_01.bsp");
    let bsp = read_bsp(bytes).unwrap();
    println!("{:#?}", bsp.textures);
}
