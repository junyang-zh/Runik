//! mod aspace: an implementation of address spaces

use super::frame::{ frame_alloc, FrameTracker };
use super::page_table::{ PTEFlags, PageTable, PageTableEntry };
use super::addr::{ PhysPageNum, VirtAddr, VirtPageNum, StepByOne, VPNRange };
use crate::plat::qemu::{ MMIO, MEMORY_END };
use crate::arch::paging::PAGE_SIZE;
use crate::sync::UPSafeCell;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::*;
use riscv::register::satp;
use bitflags::bitflags;
use xmas_elf::ElfFile;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[derive(Debug)]
pub struct AddrSpace {
    page_table: PageTable,
    segments: Vec<Segment>,
}

impl AddrSpace {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            segments: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .segments
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.segments.remove(idx);
        }
    }
    /// Add a new Segment into this AddrSpace.
    /// Assuming that there are no conflicts in the virtual address
    /// space.
    pub fn push(&mut self, mut map_area: Segment, data: Option<(VirtAddr, &[u8])>) {
        map_area.map(&mut self.page_table);
        if let Some((va, data)) = data {
            map_area.copy_data(&mut self.page_table, va, data);
        }
        self.segments.push(map_area);
    }
    /// Without kernel stacks.
    pub fn new_with_kernel() -> Self  {
        let mut addr_space = Self::new_bare();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        // println!("mapping .text section");
        addr_space.push(
            Segment::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        // println!("mapping .rodata section");
        addr_space.push(
            Segment::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        // println!("mapping .data section");
        addr_space.push(
            Segment::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        // println!("mapping .bss section");
        addr_space.push(
            Segment::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        // println!("mapping physical memory");
        addr_space.push(
            Segment::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        //println!("mapping memory-mapped registers");
        for pair in MMIO {
            addr_space.push(
                Segment::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        addr_space
    }
    /// also returns user_sp_base (brk).
    pub fn load_elf(&mut self, elf: &ElfFile) -> usize {
        // map program headers of elf, with U flag
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "Invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                // let mut map_perm = MapPermission::empty();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                println!("[kernel] mapping app section [{:#x} {:#x}) -> [{:?} {:?}), permission: {:?}",
                    ph.offset(), ph.offset() + ph.file_size(), start_va, end_va, map_perm);
                let map_area = Segment::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                self.push(
                    map_area,
                    Some((start_va, (&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]))),
                );
                // We should zero out the .bss section if there is any
                if ph.file_size() < ph.mem_size() {
                    let bss_start_va = start_va + (ph.file_size() as isize);
                    let bss_size = (ph.mem_size() - ph.file_size()) as usize;
                    self.segments.last_mut().expect("Impossible").zero_out(
                        &mut self.page_table, bss_start_va, bss_size);
                }
            }
        }
        // Construct a RW user stack based on max_end_vpn
        let max_end_va: VirtAddr = max_end_vpn.into();
        let user_stack_base_va: VirtAddr = (usize::from(max_end_va) + PAGE_SIZE - 0x10).into();
        println!("[kernel] mapping app stack {:?} {:?}", max_end_va, user_stack_base_va);
        self.push(
            Segment::new(
                max_end_va,
                user_stack_base_va,
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        user_stack_base_va.into()
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.segments.clear();
    }
}

#[derive(Debug)]
pub struct Segment {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl Segment {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn from_another(another: &Segment) -> Self {
        Self {
            vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
            data_frames: BTreeMap::new(),
            map_type: another.map_type,
            map_perm: another.map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
            MapType::Linear(pn_offset) => {
                // check for sv39
                assert!(vpn.0 < (1usize << 27));
                ppn = PhysPageNum((vpn.0 as isize + pn_offset) as usize);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    /// data: start-aligned but maybe with shorter length
    pub fn copy_data(&mut self, page_table: &mut PageTable, start: VirtAddr, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut i = 0; // data index
        let size = data.len();
        let end: VirtAddr = start + (size as isize);
        let mut cur = VirtAddr::from(start.floor());
        let mut cur_vpn = start.floor();
        assert!(self.vpn_range.get_start() <= cur_vpn, "Addr not in range in zero_out(), underflow!");
        assert!(cur_vpn < self.vpn_range.get_end(), "Addr not in range in zero_out(), overflow!");
        loop {
            let slice_start = usize::from(start.max(cur)) - usize::from(cur);
            let slice_end = usize::from(end.min(cur + (PAGE_SIZE as isize))) - usize::from(cur);
            let src = &data[i..i + slice_end - slice_start];
            let dst = &mut page_table
                .translate(cur_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[slice_start..slice_end];
            dst.copy_from_slice(src);
            cur += PAGE_SIZE as isize;
            i += slice_end - slice_start;
            if i >= size || cur >= end || cur_vpn >= self.vpn_range.get_end() {
                break;
            }
            cur_vpn.step();
        }
    }
    /// like memset [start, start + size) with zero, helpful when clearing .bss
    pub fn zero_out(&mut self, page_table: &mut PageTable, start: VirtAddr, size: usize) {
        assert_eq!(self.map_type, MapType::Framed);
        let end: VirtAddr = start + (size as isize);
        let mut cur = VirtAddr::from(start.floor());
        let mut cur_vpn = start.floor();
        assert!(self.vpn_range.get_start() <= cur_vpn, "Addr not in range in zero_out(), underflow!");
        assert!(cur_vpn < self.vpn_range.get_end(), "Addr not in range in zero_out(), overflow!");
        loop {
            let slice_start = usize::from(start.max(cur)) - usize::from(cur);
            let slice_end = usize::from(end.min(cur + (PAGE_SIZE as isize))) - usize::from(cur);
            let dst = &mut page_table
                .translate(cur_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[slice_start..slice_end];
            dst.fill(0);
            cur += PAGE_SIZE as isize;
            if cur >= end || cur_vpn >= self.vpn_range.get_end() {
                break;
            }
            cur_vpn.step();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
    /// offset of page num
    Linear(isize),
}

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<AddrSpace>> =
        Arc::new(unsafe { UPSafeCell::new(AddrSpace::new_with_kernel()) });
}

/// Load the kernel space with elf file
pub fn kspace_load_elf(elf: &ElfFile) -> usize {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    kernel_space.load_elf(elf)
}

/// Activate sv39
pub fn kspace_activate() {
    let kernel_space = KERNEL_SPACE.exclusive_access();
    kernel_space.activate();
}

pub fn kspace_from_user_buffer(ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let page_table = &kernel_space.page_table;
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}
