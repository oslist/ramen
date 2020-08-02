use core::ptr;
use uefi::table::boot;
use uefi::table::boot::MemoryType;

struct PageMapInfo {
    virt: usize,
    phys: usize,
    bytes: usize,
}

impl PageMapInfo {
    fn new(virt: usize, phys: usize, bytes: usize) -> Self {
        Self { virt, phys, bytes }
    }

    fn map(&self, mem_map: &mut [boot::MemoryDescriptor]) -> () {
        map_virt_to_phys(self.virt, self.phys, self.bytes, mem_map);
    }
}

pub fn init(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    remove_table_protection();

    let map_info = [
        PageMapInfo::new(0xffff_ffff_8000_0000, 0x0020_0000, (512 + 4 + 128) * 1024),
        PageMapInfo::new(
            0xffff_ffff_8020_0000,
            get_vram_ptr(),
            calculate_vram_bytes(),
        ),
    ];

    for info in &map_info {
        info.map(mem_map);
    }

    update_vram_ptr();
}

fn remove_table_protection() -> () {
    unsafe {
        asm!(
            "mov rax, cr0
        and eax, 0xfffeffff
        mov cr0, rax"
        )
    }
}

fn update_vram_ptr() -> () {
    unsafe {
        ptr::write(0x0ff8 as *mut u64, 0xffff_ffff_8020_0000u64);
    }
}

fn get_vram_ptr() -> usize {
    unsafe { ptr::read(0x0ff8 as *const u64) as usize }
}

fn calculate_vram_bytes() -> usize {
    unsafe {
        ptr::read(0x0ff2 as *const u8) as usize
            * ptr::read(0x0ff4 as *const u16) as usize
            * ptr::read(0x0ff6 as *const u16) as usize
            / 8
    }
}

fn map_virt_to_phys(
    virt: usize,
    phys: usize,
    bytes: usize,
    mem_map: &mut [boot::MemoryDescriptor],
) -> () {
    let num_of_pages = bytes_to_pages(bytes);

    for i in 0..num_of_pages {
        virt_points_phys(virt + BYTES_OF_PAGE * i, phys + BYTES_OF_PAGE * i, mem_map);
    }
}

fn bytes_to_pages(bytes: usize) -> usize {
    (bytes + BYTES_OF_PAGE - 1) / BYTES_OF_PAGE
}

fn virt_points_phys(virt: usize, phys: usize, mem_map: &mut [boot::MemoryDescriptor]) -> () {
    virt_points_phys_recur(virt, phys, get_pml4_addr(), mem_map, TableType::Pml4);
}

fn get_pml4_addr() -> usize {
    let addr;
    unsafe {
        asm!("mov rax, cr3",out("rax") addr,options(nomem, preserves_flags, nostack));
    }

    addr
}

fn virt_points_phys_recur(
    virt: usize,
    phys: usize,
    table_addr: usize,
    mem_map: &mut [boot::MemoryDescriptor],
    table: TableType,
) -> () {
    let ptr_to_entry = ptr_to_entry(virt, table_addr, table);

    if let TableType::Pt = table {
        return unsafe { ptr::write(ptr_to_entry, phys | PAGE_EXISTS) };
    }

    let mut entry = unsafe { ptr::read(ptr_to_entry) };

    if !entry_exists(entry) {
        entry = create_table(mem_map) | PAGE_EXISTS;
        unsafe { ptr::write(ptr_to_entry, entry) }
    }

    virt_points_phys_recur(
        virt,
        phys,
        get_addr_from_table_entry(entry),
        mem_map,
        table.next_table().unwrap(), // `table` can't be `Pt`. This `unwrap` always succeeds.
    )
}

fn get_offset_of_entry(virt_addr: usize, table: TableType) -> usize {
    (virt_addr
        >> match table {
            TableType::Pml4 => 39,
            TableType::Pdpt => 30,
            TableType::Pd => 21,
            TableType::Pt => 12,
        }
        & 0x1ff)
        * TABLE_ENTRY_SIZE
}

fn ptr_to_entry(virt: usize, table_addr: usize, table: TableType) -> *mut usize {
    (table_addr + get_offset_of_entry(virt, table)) as *mut _
}

fn entry_exists(entry: usize) -> bool {
    entry & PAGE_EXISTS == 1
}

fn create_table(mem_map: &mut [boot::MemoryDescriptor]) -> usize {
    let addr = allocate_page_for_page_table(mem_map);
    unsafe { initialize_page_table(addr) }

    addr
}

fn get_addr_from_table_entry(entry: usize) -> usize {
    entry & 0xffff_ffff_ffff_f000
}

fn allocate_page_for_page_table(mem_map: &mut [boot::MemoryDescriptor]) -> usize {
    for descriptor in mem_map.iter_mut() {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            let addr = descriptor.phys_start;
            descriptor.phys_start += BYTES_OF_PAGE as u64;
            descriptor.page_count -= 1;

            return addr as usize;
        }
    }

    // Shouldn't reach here.
    panic!("Failed to allocate memory for a page table.");
}

unsafe fn initialize_page_table(table_addr: usize) -> () {
    ptr::write_bytes(table_addr as *mut u8, 0, BYTES_OF_PAGE)
}

#[derive(Copy, Clone)]
enum TableType {
    Pml4,
    Pdpt,
    Pd,
    Pt,
}

impl TableType {
    fn next_table(&self) -> Option<TableType> {
        match self {
            TableType::Pt => None,
            TableType::Pd => Some(TableType::Pt),
            TableType::Pdpt => Some(TableType::Pd),
            TableType::Pml4 => Some(TableType::Pdpt),
        }
    }
}

const TABLE_ENTRY_SIZE: usize = 8;

const PAGE_EXISTS: usize = 1;
const BYTES_OF_PAGE: usize = 0x1000;
