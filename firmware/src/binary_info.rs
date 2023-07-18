use rp_binary_info::{
    build_date, custom_string,
    entry::{Addr, IdAndString},
    program_name, version, Header, MappingTableEntry, ID_RP_PICO_BOARD,
    ID_RP_PROGRAM_BUILD_ATTRIBUTE, ID_RP_PROGRAM_DESCRIPTION, ID_RP_PROGRAM_FEATURE,
    ID_RP_PROGRAM_URL, TAG_RASPBERRY_PI,
};

extern "C" {
    static __bi_entries_start: Addr;
    static __bi_entries_end: Addr;
    static __sdata: u32;
    static __edata: u32;
    static __sidata: u32;
}

#[link_section = ".bi_header"]
#[used]
pub static PICOTOOL_META: Header =
    unsafe { Header::new(&__bi_entries_start, &__bi_entries_end, &MAPPING_TABLE) };

// This tells picotool how to convert RAM addresses back into Flash addresses
static MAPPING_TABLE: [MappingTableEntry; 2] = [
    // This is the entry for .data
    MappingTableEntry {
        source_addr_start: unsafe { &__sidata },
        dest_addr_start: unsafe { &__sdata },
        dest_addr_end: unsafe { &__edata },
    },
    // This is the terminating marker
    MappingTableEntry {
        source_addr_start: core::ptr::null(),
        dest_addr_start: core::ptr::null(),
        dest_addr_end: core::ptr::null(),
    },
];

// This is a list of references to our table entries
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [Addr; 8] = [
    PROGRAM_NAME.addr(),
    PROGRAM_VERSION.addr(),
    PROGRAM_BUILD_DATE.addr(),
    PROGRAM_URL.addr(),
    PROGRAM_DESCRIPTION.addr(),
    PROGRAM_FEATURE.addr(),
    PROGRAM_BUILD_ATTRIBUTE.addr(),
    PICO_BOARD.addr(),
];

static PROGRAM_NAME: IdAndString = program_name(concat!(env!("CARGO_PKG_NAME"), "\0"));
static PROGRAM_VERSION: IdAndString = version(concat!("v", env!("CARGO_PKG_VERSION"), "\0"));
static PROGRAM_BUILD_DATE: IdAndString = build_date(concat!(
    include_str!(concat!(env!("OUT_DIR"), "/build_date.txt")),
    "\0"
));
static PROGRAM_URL: IdAndString = custom_string(
    TAG_RASPBERRY_PI,
    ID_RP_PROGRAM_URL,
    concat!(env!("CARGO_PKG_REPOSITORY"), "\0"),
);
static PROGRAM_DESCRIPTION: IdAndString = custom_string(
    TAG_RASPBERRY_PI,
    ID_RP_PROGRAM_DESCRIPTION,
    concat!(env!("CARGO_PKG_DESCRIPTION"), "\0"),
);
static PROGRAM_FEATURE: IdAndString = custom_string(
    TAG_RASPBERRY_PI,
    ID_RP_PROGRAM_FEATURE,
    concat!(
        "mod taps, layers, chords, mouse keys, cirque trackpad, neopixel animations, single firmware binary",
        "\0"
    ),
);
static PROGRAM_BUILD_ATTRIBUTE: IdAndString = custom_string(
    TAG_RASPBERRY_PI,
    ID_RP_PROGRAM_BUILD_ATTRIBUTE,
    concat!(
        include_str!(concat!(env!("OUT_DIR"), "/build_attribute.txt")),
        "\0"
    ),
);
static PICO_BOARD: IdAndString = custom_string(
    TAG_RASPBERRY_PI,
    ID_RP_PICO_BOARD,
    concat!("dilemma-v2", "\0"),
);
