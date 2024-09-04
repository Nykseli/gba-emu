mod gba_file;

fn main() {
    let header = gba_file::GBAHeader::from_file("demos.gba");
    println!("{:08x}", header.rom_entry_point);
    println!("title: {}", header.game_title);
    println!("game code: {}", header.game_code);
    println!("maker code: {}", header.maker_code);
    println!("{:08x}", header.ram_entry_point);
    println!("{:08x}", header.joy_entry_point);
}
