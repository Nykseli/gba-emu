#[derive(Debug, Default)]
#[repr(C)]
pub struct GBAHeader {
    /// u32 rom_entry_point @ 0x00
    ///
    /// Space for a single 32bit ARM opcode that redirects to the actual startaddress
    /// of the cartridge, this should be usually a "B <start>" instruction.
    ///
    /// Note: This entry is ignored by Multiboot slave GBAs
    pub rom_entry_point: u32,

    /// u8 nintendo_logo[0x9C] @ 0x04;
    ///
    /// Huffman compressed nintendo logo that's displayed during boot
    /// 156 bytes
    nintendo_logo: Vec<u8>,

    /// char game_title[0x0C] @ 0xA0
    ///
    /// Game title (max 12 characters)
    pub game_title: String,

    /// char game_code[0x04] @ 0xAC
    ///
    /// Game code (4 characters)
    pub game_code: String,

    /// char maker_code[0x2] @ 0xB0
    ///
    /// Maker code (2 characters)
    pub maker_code: String,

    /// u8 fixed_value @ 0xB2
    ///
    /// Fixed value, must be 0x96
    fixed_value: u8,

    /// u8 main_unit_code @ 0xB3
    ///
    /// Main unit code 0x0 for current GBA models
    main_unit_code: u8,

    /// u8 device_type @ 0xB4;
    ///
    /// Device type, usually 0, may contain debugging related things
    device_type: u8,

    /// u8 reserved_area[0x07] @ 0xB5;
    ///
    /// reserved area, should be zero filled
    reserved_area: [u8; 7],

    /// u8 software_version @ 0xBC;
    ///
    /// Software version, usually 0x0
    software_version: u8,

    /// u8 complement_check @ 0xBD;
    ///
    /// complement check TODO:
    complement_check: u8,

    /// u8 reserved_area2[0x02] @ 0xBE
    ///
    /// reserved area, should be zero filled
    reserved_area2: [u8; 2],

    /// u32 ram_entry_point @ 0xC0
    ///
    /// Normal/Multiplay mode Entry Point
    /// This entry is used only if the GBA has been booted by using Normal or Multiplay
    /// transfer mode (but not by Joybus mode).
    /// Typically deposit a ARM-32bit "B <start>" branch opcode at this location,
    /// which is pointing to your actual initialization procedure.
    pub ram_entry_point: u32,

    /// u8 boot_mode @ 0xC4
    ///
    /// The slave GBA download procedure overwrites this byte by a value which is
    /// indicating the used multiboot transfer mode.
    ///
    /// Value  Expl.
    /// 01h    Joybus mode
    /// 02h    Normal mode
    /// 03h    Multiplay mode
    ///
    /// Typically set this byte to zero by inserting DCB 00h in your source.
    /// Be sure that your uploaded program does not contain important program code
    /// or data at this location, or at the ID-byte location below.
    pub boot_mode: u8,

    /// u8 slave_id @ 0xC5
    ///
    /// If the GBA has been booted in Normal or Multiplay mode, this byte becomes
    /// overwritten by the slave ID number of the local GBA (that'd be always 01h for normal mode).
    ///
    /// Value  Expl.
    /// 01h    Slave #1
    /// 02h    Slave #2
    /// 03h    Slave #3
    /// Typically set this byte to zero by inserting DCB 00h in your source.
    /// When booted in Joybus mode, the value is NOT changed and remains the same as uploaded from the master GBA.
    pub slave_id: u8,

    /// u8 reserved_area3[26] @ 0xBE
    ///
    /// reserved area, should be zero filled
    reserved_area3: [u8; 2],

    /// u32 joy_entry_point @ 0xE0
    ///
    /// If the GBA has been booted by using Joybus transfer mode, then the entry
    /// point is located at this address rather than at 20000C0h.
    /// Either put your initialization procedure directly at this address, or redirect
    /// to the actual boot procedure by depositing a "B <start>" opcode here
    /// (either one using 32bit ARM code). Or, if you are not intending to support
    /// joybus mode (which is probably rarely used), ignore this entry.
    pub joy_entry_point: u32,
}

impl GBAHeader {
    fn add_rom_entry_point(&mut self, data: &[u8]) {
        self.rom_entry_point = u32::from_le_bytes(data[0..4].try_into().unwrap());
    }

    fn add_logo(&mut self, data: &[u8]) {
        self.nintendo_logo = data[0x04..0x04 + 0x9C].into();
    }

    fn add_game_title(&mut self, data: &[u8]) {
        self.game_title =
            String::from_utf8(data[0xA0..0xA0 + 12].into()).expect("Game title utf8 string");
    }

    fn add_game_code(&mut self, data: &[u8]) {
        self.game_code =
            String::from_utf8(data[0xAC..0xAC + 4].into()).expect("Game code utf8 string");
    }

    fn add_maker_code(&mut self, data: &[u8]) {
        self.maker_code =
            String::from_utf8(data[0xB0..0xB0 + 2].into()).expect("Game maker utf8 string");
    }

    fn add_fixed_value(&mut self, data: &[u8]) {
        self.fixed_value = data[0xB2];
        assert_eq!(self.fixed_value, 0x96);
    }

    fn add_main_unit_code(&mut self, data: &[u8]) {
        self.main_unit_code = data[0xB3];
    }

    fn add_device_type(&mut self, data: &[u8]) {
        self.device_type = data[0xB4];
    }

    fn add_software_version(&mut self, data: &[u8]) {
        self.software_version = data[0xBC];
    }

    fn add_complement_check(&mut self, data: &[u8]) {
        // TODO: do the check
        self.complement_check = data[0xBD];
    }

    fn add_ram_entry_point(&mut self, data: &[u8]) {
        self.ram_entry_point = u32::from_le_bytes(data[0xC0..0xC0 + 4].try_into().unwrap());
    }

    fn add_boot_mode(&mut self, data: &[u8]) {
        self.boot_mode = data[0xC4];
    }

    fn add_slave_id(&mut self, data: &[u8]) {
        self.slave_id = data[0xC5];
    }

    fn add_joy_entry_point(&mut self, data: &[u8]) {
        self.joy_entry_point = u32::from_le_bytes(data[0xE0..0xE0 + 4].try_into().unwrap());
    }

    pub fn from_file(bytes: &[u8]) -> Self {
        let mut header: Self = Default::default();

        header.add_rom_entry_point(bytes);
        header.add_logo(bytes);
        header.add_game_title(bytes);
        header.add_game_code(bytes);
        header.add_maker_code(bytes);
        header.add_fixed_value(bytes);
        header.add_main_unit_code(bytes);
        header.add_device_type(bytes);
        header.add_software_version(bytes);
        header.add_complement_check(bytes);
        header.add_ram_entry_point(bytes);
        header.add_boot_mode(bytes);
        header.add_slave_id(bytes);
        header.add_joy_entry_point(bytes);

        header
    }
}
