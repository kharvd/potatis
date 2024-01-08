use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use mos6502::cpu::Cpu;
use mos6502::memory::Bus;
use mos6502::mos6502::Mos6502;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use structopt::StructOpt;

enum MemoryMap {
  Ram,
  Display,
  Rom,
}

const WIDTH: usize = 240;
const HEIGHT: usize = 192;

enum DisplayPort {
  PortX = 0,
  PortY = 1,
  PortColor = 2,
  PortCommand = 3,
}

impl DisplayPort {
  fn from_u16(val: u16) -> Self {
    match val {
      0 => DisplayPort::PortX,
      1 => DisplayPort::PortY,
      2 => DisplayPort::PortColor,
      3 => DisplayPort::PortCommand,
      _ => panic!("invalid display port"),
    }
  }
}

enum DisplayCommand {
  Nop = 0,
  Draw = 1,
  Clear = 2,
  Flush = 3,
}

impl DisplayCommand {
  fn from_u8(val: u8) -> Self {
    match val {
      0 => DisplayCommand::Nop,
      1 => DisplayCommand::Draw,
      2 => DisplayCommand::Clear,
      3 => DisplayCommand::Flush,
      _ => panic!("invalid display command"),
    }
  }
}

struct DisplayBuffer {
  pub buffer: [u8; WIDTH * HEIGHT],
  port_x: u8,
  port_y: u8,
  port_color: u8,
  port_command: u8,
  was_updated: bool,
}

impl DisplayBuffer {
  fn new() -> Self {
    let buffer = [0; WIDTH * HEIGHT];
    Self {
      buffer,
      port_x: 0,
      port_y: 0,
      port_color: 0,
      port_command: 0,
      was_updated: false,
    }
  }

  fn read8(&self, address: u16) -> u8 {
    panic!("cannot read from display buffer")
  }

  fn write8(&mut self, val: u8, address: u16) {
    match DisplayPort::from_u16(address) {
      DisplayPort::PortX => self.port_x = val,
      DisplayPort::PortY => self.port_y = val,
      DisplayPort::PortColor => self.port_color = val,
      DisplayPort::PortCommand => self.port_command = val,
    }

    match DisplayCommand::from_u8(self.port_command) {
      DisplayCommand::Draw => self.draw(),
      DisplayCommand::Flush => self.was_updated = true,
      DisplayCommand::Clear => self.clear(),
      _ => {}
    }

    self.port_command = 0;
  }

  fn draw(&mut self) {
    let x = self.port_x as usize;
    let y = self.port_y as usize;
    let color = self.port_color;
    self.buffer[y * WIDTH + x] = color;
  }

  fn clear(&mut self) {
    self.buffer = [0; WIDTH * HEIGHT];
  }

  fn flush(&mut self) {
    self.was_updated = true;
  }

  fn was_updated(&mut self) -> bool {
    let result = self.was_updated;
    self.was_updated = false;
    result
  }
}

struct Rom {
  rom: [u8; 0x5a80],
}

impl Rom {
  fn new(buffer: &[u8]) -> Self {
    Self {
      rom: buffer.try_into().unwrap(),
    }
  }

  fn load_file(file_name: &str) -> Self {
    let data = std::fs::read(file_name).unwrap();
    Self::new(&data)
  }

  fn read8(&self, address: u16) -> u8 {
    self.rom[address as usize]
  }
}

struct PongBus {
  ram: [u8; 0x8000],                   // [0x0000; 0x8000)
  display: Rc<RefCell<DisplayBuffer>>, // [0x8000; 0xA580)
  rom: Rc<RefCell<Rom>>,               // [0xA580; 0xFFFF)
}

impl PongBus {
  fn new(display_buffer: Rc<RefCell<DisplayBuffer>>, rom: Rc<RefCell<Rom>>) -> Self {
    Self {
      ram: [0; 0x8000],
      display: display_buffer,
      rom,
    }
  }
}

impl PongBus {
  fn map_address(&self, address: u16) -> (MemoryMap, u16) {
    match address {
      0x0000..=0x7FFF => (MemoryMap::Ram, address),
      0x8000..=0xA57F => (MemoryMap::Display, address - 0x8000),
      0xA580..=0xFFFF => (MemoryMap::Rom, address - 0xA580),
    }
  }
}

impl Bus for PongBus {
  fn read8(&self, address: u16) -> u8 {
    // println!("read8: {:#06x}", address);
    let (memory_map, mapped_address) = self.map_address(address);
    match memory_map {
      MemoryMap::Ram => self.ram[mapped_address as usize],
      MemoryMap::Display => self.display.borrow().read8(mapped_address),
      MemoryMap::Rom => self.rom.borrow().read8(mapped_address),
    }
  }

  fn write8(&mut self, val: u8, address: u16) {
    // println!("write8: {:#06x} = {:#04x}", address, val);
    let (memory_map, mapped_address) = self.map_address(address);
    match memory_map {
      MemoryMap::Ram => self.ram[mapped_address as usize] = val,
      MemoryMap::Display => self.display.borrow_mut().write8(val, mapped_address),
      MemoryMap::Rom => panic!("cannot write to ROM"),
    }
  }
}

#[derive(StructOpt, Debug)]
struct Cli {
  rom: PathBuf,
  #[structopt(short, long)]
  verbose: bool,
  #[structopt(short, long)]
  debug: bool,
}

fn main() {
  let args = Cli::from_args();
  let display_buffer = Rc::new(RefCell::new(DisplayBuffer::new()));
  let rom = Rc::new(RefCell::new(Rom::load_file(args.rom.to_str().unwrap())));
  let bus = PongBus::new(display_buffer.clone(), rom.clone());
  let mut cpu = Cpu::new(bus);
  cpu.reset();
  let mut machine = Mos6502::new(cpu);
  machine.debugger().verbose(args.verbose);
  if args.debug {
    machine.debugger().enable();
    machine.debugger().watch_memory_range(0..=5, |mem| {
      println!("watched memory range: {:?}", mem);
    });
  }

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let scale = 4;

  let window = video_subsystem
    .window("pong", WIDTH as u32 * scale, HEIGHT as u32 * scale)
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().build().unwrap();
  let creator = canvas.texture_creator();
  let mut texture = creator
    .create_texture_target(PixelFormatEnum::RGB332, WIDTH as u32, HEIGHT as u32)
    .unwrap();
  canvas.copy(&texture, None, None).unwrap();

  let mut event_pump = sdl_context.event_pump().unwrap();
  let mut i = 0;
  'running: loop {
    i += 1;
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => break 'running,
        _ => {}
      }
    }

    machine.tick();

    if display_buffer.borrow_mut().was_updated() {
      texture
        .update(None, &display_buffer.borrow().buffer, WIDTH)
        .unwrap();
      canvas.copy(&texture, None, None).unwrap();
      canvas.present();

      ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120)); // 120 fps
    }
  }
}
