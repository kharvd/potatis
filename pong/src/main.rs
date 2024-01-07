use std::cell::RefCell;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use mos6502::cpu::Cpu;
use mos6502::memory::Bus;
use mos6502::mos6502::Mos6502;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use structopt::StructOpt;

enum MemoryMap {
  Ram,
  Display,
  Rom,
}

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

struct DisplayBuffer {
  // each pixel is 1 byte: 0x1 = white, 0x0 = black
  pub buffer: [u8; WIDTH * HEIGHT],
}

impl DisplayBuffer {
  fn new() -> Self {
    let buffer = [0; WIDTH * HEIGHT];
    Self { buffer }
  }

  fn read8(&self, address: u16) -> u8 {
    // address points at a block of 8 pixels
    // each pixel is 1 byte: 0xff = white, 0x00 = black
    // LSB is the leftmost pixel
    let block = address as usize * 8;
    let mut val = 0;
    for i in 0..8 {
      val |= (self.buffer[block + i] & 0x1) << i;
    }
    val
  }

  fn write8(&mut self, val: u8, address: u16) {
    let block = address as usize * 8;
    for i in 0..8 {
      let pixel_value = (val >> i) & 0x1;
      let rgb332 = if pixel_value == 0 { 0x00 } else { 0xff };

      self.buffer[block + i] = rgb332;
    }
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
  }

  // for y in 0..HEIGHT {
  //   display_buffer
  //     .borrow_mut()
  //     .write8(0x01, ((WIDTH / 8) * y) as u16);
  //   display_buffer
  //     .borrow_mut()
  //     .write8(0x80, ((WIDTH / 8) * y + ((WIDTH - 1) / 8)) as u16);
  // }

  // for x in (0..WIDTH).step_by(8) {
  //   display_buffer.borrow_mut().write8(0xff, (x / 8) as u16);
  //   let y = HEIGHT - 1;
  //   display_buffer
  //     .borrow_mut()
  //     .write8(0xff, (x / 8 + (WIDTH / 8) * y) as u16);
  // }

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let scale = 2;

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

  texture
    .update(None, &display_buffer.borrow().buffer, WIDTH)
    .unwrap();

  canvas.copy(&texture, None, None).unwrap();
  // canvas.present();

  // canvas.set_draw_color(Color::RGB(0, 255, 255));
  // canvas.clear();
  // canvas.present();
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

    // if i > 5 {
    //   break;
    // }

    machine.tick();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
  }
}
