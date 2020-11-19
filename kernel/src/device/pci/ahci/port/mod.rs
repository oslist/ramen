// SPDX-License-Identifier: GPL-3.0-or-later

mod command_list;
mod fis;
mod received_fis;

use {
    super::registers::{port, Registers},
    crate::{
        mem::allocator::page_box::PageBox,
        multitask::task::{self, Task},
    },
    alloc::rc::Rc,
    command_list::CommandList,
    core::{cell::RefCell, convert::TryInto, mem},
    fis::RegFixH2D,
    received_fis::ReceivedFis,
};

const MAX_PORTS: usize = 32;

pub fn spawn_tasks(
    registers: &Rc<RefCell<Registers>>,
    task_collection: &Rc<RefCell<task::Collection>>,
) {
    (0..MAX_PORTS)
        .filter_map(|i| Port::new(registers.clone(), i))
        .for_each(|port| {
            task_collection
                .borrow_mut()
                .add_task_as_woken(Task::new(task(port)))
        })
}

async fn task(mut port: Port) {
    info!("This is a task of port {}", port.index);
    port.init();
    port.start();

    let buf = port.read();
    info!("Buf: {:?}", buf);
}

pub struct Port {
    registers: Rc<RefCell<Registers>>,
    command_list: CommandList,
    received_fis: ReceivedFis,
    index: usize,
}
impl Port {
    fn idle(&mut self) {
        self.edit_port_rg(|rg| {
            rg.cmd.update(|cmd| {
                cmd.set_start_bit(false);
                cmd.set_fis_receive_enable(false)
            })
        });

        while {
            self.parse_port_rg(|reg| {
                let cmd = reg.cmd.read();
                cmd.command_list_running() || cmd.fis_receive_running()
            })
        } {}
    }

    fn new(registers: Rc<RefCell<Registers>>, index: usize) -> Option<Self> {
        if Self::exists(&registers, index) {
            Some(Self::generate(registers, index))
        } else {
            None
        }
    }

    fn exists(registers: &Rc<RefCell<Registers>>, index: usize) -> bool {
        let registers = &registers.borrow();
        let pi: usize = registers.generic.pi.read().0.try_into().unwrap();
        pi & (1 << index) != 0
    }

    fn generate(registers: Rc<RefCell<Registers>>, index: usize) -> Self {
        let command_list = CommandList::new(&*registers.borrow());
        let received_fis = ReceivedFis::new();
        Self {
            registers,
            received_fis,
            command_list,
            index,
        }
    }

    fn init(&mut self) {
        self.idle();
        self.register_command_list_and_received_fis();
        self.clear_error_bits();
    }

    fn register_command_list_and_received_fis(&mut self) {
        self.assert_64bit_accessing_is_supported();
        self.register_command_list();
        self.register_received_fis();
    }

    fn assert_64bit_accessing_is_supported(&self) {
        let registers = &self.registers.borrow();
        assert!(registers.generic.cap.read().supports_64bit_addressing());
    }

    fn register_command_list(&mut self) {
        let addr = self.command_list.phys_addr_to_headers();
        self.edit_port_rg(|rg| rg.clb.update(|b| b.set(addr)));
    }

    fn register_received_fis(&mut self) {
        self.register_fis_addr();
        self.enable_receiving_fis();
    }

    fn register_fis_addr(&mut self) {
        let addr = self.received_fis.phys_addr();
        self.edit_port_rg(|rg| rg.fb.update(|b| b.set(addr)));
    }

    fn enable_receiving_fis(&mut self) {
        self.edit_port_rg(|r| r.cmd.update(|r| r.set_fis_receive_enable(true)));
    }

    fn clear_error_bits(&mut self) {
        // Refer to P.31 and P.104 of Serial ATA AHCI 1.3.1 Specification
        const BIT_MASK: u32 = 0x07ff_0f03;
        self.edit_port_rg(|rg| rg.serr.update(|serr| serr.0 = BIT_MASK));
    }

    fn start(&mut self) {
        if self.ready_to_start() {
            self.start_processing();
        }
    }

    fn ready_to_start(&self) -> bool {
        !self.command_list_is_running() && self.fis_receive_enabled() && self.device_is_present()
    }

    fn command_list_is_running(&self) -> bool {
        self.parse_port_rg(|r| r.cmd.read().command_list_running())
    }

    fn fis_receive_enabled(&self) -> bool {
        self.parse_port_rg(|r| r.cmd.read().fis_receive_enable())
    }

    fn device_is_present(&self) -> bool {
        self.parse_port_rg(|r| {
            r.ssts.read().device_detection() == 3
                || [2, 6, 8].contains(&r.ssts.read().interface_power_management())
        })
    }

    fn start_processing(&mut self) {
        self.edit_port_rg(|r| r.cmd.update(|r| r.set_start_bit(true)))
    }

    fn read(&mut self) -> PageBox<[u8]> {
        let mut buf = PageBox::new_slice(0, 512);
        self.init_command_list_for_reading_sectors(1, &mut buf);
        buf
    }

    fn init_command_list_for_reading_sectors(&mut self, sector_cnt: u16, buf: &mut PageBox<[u8]>) {
        self.init_command_header_and_table(sector_cnt, buf).unwrap();
    }

    fn init_command_header_and_table(
        &mut self,
        sector_cnt: u16,
        buf: &mut PageBox<[u8]>,
    ) -> Result<(), Error> {
        let slot_index = self
            .get_available_command_slot()
            .ok_or(Error::SlotIsNotAvailable)?;
        self.init_command_list_header(slot_index, sector_cnt);
        self.init_command_table(slot_index, sector_cnt, buf);
        self.init_fis(slot_index);
        self.edit_port_rg(|r| r.ci.update(|ci| ci.issue(slot_index)));

        while self.parse_port_rg(|r| r.ci.read().get() & (1 << slot_index) != 0) {}

        Ok(())
    }

    fn init_command_list_header(&mut self, slot_index: usize, sector_cnt: u16) {
        let header = &mut self.command_list.headers[slot_index];
        header.set_cfl((mem::size_of::<RegFixH2D>() / mem::size_of::<u32>()) as u8);
        header.set_prdtl(((sector_cnt - 1) >> 4) + 1);
    }

    fn init_command_table(&mut self, slot_index: usize, sector_cnt: u16, buf: &mut PageBox<[u8]>) {
        let prdtl: usize = self.command_list.headers[slot_index].prdtl().into();
        let table = &mut self.command_list.tables[slot_index];
        for i in 0..prdtl {
            table.prdt[i].set_dba(buf.phys_addr() + 4 * 1024 * i);
            table.prdt[i].set_dbc(8 * 1024 - 1);
        }
        table.prdt[prdtl - 1].set_dbc((u32::from(sector_cnt) << 9) - 1);
    }

    fn init_fis(&mut self, slot_index: usize) {
        let fis = &mut self.command_list.tables[slot_index].fis;
        fis.set_lba(0);
        fis.set_c(true);
        fis.set_command(0x25);
        fis.set_device(1 << 6);
    }

    fn get_available_command_slot(&self) -> Option<usize> {
        for i in 0..self.num_of_command_slots() {
            if self.slot_available(i) {
                return Some(i);
            }
        }

        None
    }

    fn num_of_command_slots(&self) -> usize {
        let cap = &self.registers.borrow().generic.cap.read();
        cap.num_of_command_slots().try_into().unwrap()
    }

    fn slot_available(&self, index: usize) -> bool {
        let bitflag_used_slots = self.parse_port_rg(|r| r.sact.read().get() | r.ci.read().get());
        bitflag_used_slots & (1 << index) == 0
    }

    fn parse_port_rg<T, U>(&self, f: T) -> U
    where
        T: Fn(&port::Registers) -> U,
    {
        let registers = &self.registers.borrow_mut();
        let port_rg = registers.port_regs[self.index].as_ref().unwrap();
        f(port_rg)
    }

    fn edit_port_rg<T>(&mut self, f: T)
    where
        T: Fn(&mut port::Registers),
    {
        let registers = &mut self.registers.borrow_mut();
        let port_rg = registers.port_regs[self.index].as_mut().unwrap();
        f(port_rg);
    }
}

#[derive(Debug)]
enum Error {
    SlotIsNotAvailable,
}
