const PAGE_SIZE_4K: u64 = 4096;
const PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
const PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct PageDirectory {
    data: [u64; 512],
}

impl PageDirectory {
    const fn new() -> Self {
        Self { data: [0; 512] }
    }
    #[allow(non_snake_case)]
    fn set_2M_page_entry(&mut self, idx: usize, addr: u64) {
        self.data[idx] = addr | 0x83;
    }
}

static mut PAGE_DIRECTORIES: [PageDirectory; 64] = [PageDirectory::new(); 64];

#[repr(align(4096))]
struct PageDirectoryPointerTable {
    data: [u64; 512],
}

impl PageDirectoryPointerTable {
    const fn new() -> Self {
        Self { data: [0; 512] }
    }
    fn set_entry(&mut self, idx: usize, pdp: u64) {
        self.data[idx] = pdp | 0x03;
    }
}

static mut PDP_TABLE: PageDirectoryPointerTable = PageDirectoryPointerTable::new();

#[repr(align(4096))]
struct PML4 {
    data: [u64; 512],
}

impl PML4 {
    const fn new() -> Self {
        Self { data: [0; 512] }
    }
    fn set_entry(&mut self, idx: usize, pdptp: u64) {
        self.data[idx] = pdptp | 0x03;
    }
}

static mut PML4_: PML4 = PML4::new();

#[allow(static_mut_refs)]
pub fn setup_identity_page_table() {
    // Setup identity page table for 64 GB address space.
    unsafe {
        PML4_.set_entry(0, &PDP_TABLE as *const PageDirectoryPointerTable as u64);
        for i in 0..64 {
            PDP_TABLE.set_entry(i, &PAGE_DIRECTORIES[i] as *const PageDirectory as u64);
            for j in 0..512 {
                PAGE_DIRECTORIES[i]
                    .set_2M_page_entry(j, (i as u64) * PAGE_SIZE_1G + (j as u64) * PAGE_SIZE_2M);
            }
        }
        let pml4_addr = &PML4_ as *const PML4 as u64;
        core::arch::asm!(
            "mov cr3, {}", in(reg) pml4_addr
        );
    }
}
