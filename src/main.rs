use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, DisableLineWrap, EnableLineWrap},
};
use rand::Rng;
use std::{
    env,
    io::{stdout, Write},
    time::Duration,
};

const CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%^&*()_+-=[]{}|;:,.<>?アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン";

#[derive(Clone, Copy)]
enum ColorScheme {
    Green,
    Blue,
    Red,
    Purple,
    Cyan,
    Rainbow,
}

impl ColorScheme {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "green" => Some(Self::Green),
            "blue" => Some(Self::Blue),
            "red" => Some(Self::Red),
            "purple" => Some(Self::Purple),
            "cyan" => Some(Self::Cyan),
            "rainbow" => Some(Self::Rainbow),
            _ => None,
        }
    }

    fn get_colors(&self, i: usize, length: usize, x: u16) -> Color {
        // Calculate fade factor (0.0 at head, 1.0 at tail)
        let fade = i as f32 / length as f32;

        match self {
            Self::Green => {
                if i == 0 {
                    Color::Rgb { r: 200, g: 255, b: 200 } // Bright white-green head
                } else if i == 1 {
                    Color::Rgb { r: 100, g: 255, b: 100 } // Near-head glow
                } else {
                    // Smooth fade from bright green to dark green
                    let intensity = (1.0 - fade * 0.85).max(0.15);
                    let g = (255.0 * intensity) as u8;
                    let r = (30.0 * (1.0 - fade)) as u8;
                    Color::Rgb { r, g, b: 0 }
                }
            }
            Self::Blue => {
                if i == 0 {
                    Color::Rgb { r: 200, g: 220, b: 255 }
                } else if i == 1 {
                    Color::Rgb { r: 100, g: 150, b: 255 }
                } else {
                    let intensity = (1.0 - fade * 0.85).max(0.15);
                    let b = (255.0 * intensity) as u8;
                    let g = (100.0 * intensity) as u8;
                    Color::Rgb { r: 0, g, b }
                }
            }
            Self::Red => {
                if i == 0 {
                    Color::Rgb { r: 255, g: 220, b: 200 }
                } else if i == 1 {
                    Color::Rgb { r: 255, g: 100, b: 100 }
                } else {
                    let intensity = (1.0 - fade * 0.85).max(0.15);
                    let r = (255.0 * intensity) as u8;
                    let g = (30.0 * (1.0 - fade)) as u8;
                    Color::Rgb { r, g, b: 0 }
                }
            }
            Self::Purple => {
                if i == 0 {
                    Color::Rgb { r: 240, g: 200, b: 255 }
                } else if i == 1 {
                    Color::Rgb { r: 200, g: 100, b: 255 }
                } else {
                    let intensity = (1.0 - fade * 0.85).max(0.15);
                    let r = (180.0 * intensity) as u8;
                    let b = (255.0 * intensity) as u8;
                    Color::Rgb { r, g: 0, b }
                }
            }
            Self::Cyan => {
                if i == 0 {
                    Color::Rgb { r: 200, g: 255, b: 255 }
                } else if i == 1 {
                    Color::Rgb { r: 100, g: 255, b: 255 }
                } else {
                    let intensity = (1.0 - fade * 0.85).max(0.15);
                    let g = (255.0 * intensity) as u8;
                    let b = (255.0 * intensity) as u8;
                    Color::Rgb { r: 0, g, b }
                }
            }
            Self::Rainbow => {
                if i == 0 {
                    Color::White
                } else {
                    let hue = ((x as f32 * 10.0 + i as f32 * 15.0) % 360.0) / 360.0;
                    let intensity = (1.0 - fade * 0.8).max(0.2);
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, intensity);
                    Color::Rgb { r, g, b }
                }
            }
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

struct Settings {
    frame_delay_ms: u64,   // Lower = faster (default 30)
    density: f64,          // Spawn probability 0.0-1.0 (default 0.15)
    spawns_per_frame: u32, // Max spawns per frame (default 3)
    min_length: usize,     // Min drop length (default 5)
    max_length: usize,     // Max drop length (default 25)
    min_speed: u8,         // Min drop speed (default 1)
    max_speed: u8,         // Max drop speed, lower = faster (default 3)
    color_scheme: ColorScheme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            frame_delay_ms: 50,    // Slower, more relaxed
            density: 0.4,          // Moderate density
            spawns_per_frame: 4,   // Moderate coverage
            min_length: 10,        // Longer trails for fade effect
            max_length: 30,        // Long trails
            min_speed: 2,          // Slower drops
            max_speed: 4,          // Even slower variation
            color_scheme: ColorScheme::Green,
        }
    }
}

struct Drop {
    x: u16,
    y: i32,
    speed: u8,
    length: usize,
    chars: Vec<char>,
    tick: u8,
}

impl Drop {
    fn new(x: u16, settings: &Settings) -> Self {
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(settings.min_length..=settings.max_length);
        let chars_vec: Vec<char> = CHARS.chars().collect();

        Drop {
            x,
            y: rng.gen_range(-30..0),
            speed: rng.gen_range(settings.min_speed..=settings.max_speed),
            length,
            chars: (0..length)
                .map(|_| chars_vec[rng.gen_range(0..chars_vec.len())])
                .collect(),
            tick: 0,
        }
    }

    fn update(&mut self, height: u16, color_scheme: ColorScheme) -> Vec<(u16, u16, char, Color)> {
        self.tick += 1;
        if self.tick % self.speed != 0 {
            return vec![];
        }

        self.y += 1;

        // Shimmer effect - multiple characters can change per frame
        let mut rng = rand::thread_rng();
        let shimmer_count = rng.gen_range(0..=2);
        let chars_vec: Vec<char> = CHARS.chars().collect();
        for _ in 0..shimmer_count {
            if rng.gen_bool(0.5) {
                let idx = rng.gen_range(0..self.length);
                self.chars[idx] = chars_vec[rng.gen_range(0..chars_vec.len())];
            }
        }

        let mut draws = vec![];

        for (i, &ch) in self.chars.iter().enumerate() {
            let char_y = self.y - i as i32;
            if char_y >= 0 && char_y < height as i32 {
                let color = color_scheme.get_colors(i, self.length, self.x);
                draws.push((self.x, char_y as u16, ch, color));
            }
        }

        // Clear tail
        let tail_y = self.y - self.length as i32;
        if tail_y >= 0 && tail_y < height as i32 {
            draws.push((self.x, tail_y as u16, ' ', Color::Black));
        }

        draws
    }

    fn is_done(&self, height: u16) -> bool {
        self.y - self.length as i32 > height as i32
    }
}

struct Matrix {
    drops: Vec<Drop>,
    width: u16,
    height: u16,
    settings: Settings,
}

impl Matrix {
    fn new(settings: Settings) -> Self {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        Matrix {
            drops: vec![],
            width,
            height,
            settings,
        }
    }

    fn spawn_drops(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..rng.gen_range(1..=self.settings.spawns_per_frame) {
            if rng.gen_bool(self.settings.density) {
                let x = rng.gen_range(0..self.width);
                self.drops.push(Drop::new(x, &self.settings));
            }
        }
    }

    fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        execute!(stdout, Hide, DisableLineWrap, Clear(ClearType::All))?;

        loop {
            // Check for key press (non-blocking)
            if poll(Duration::from_millis(1))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        // Exit keys
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => break,
                        KeyCode::Char('c') | KeyCode::Char('d') | KeyCode::Char('z')
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            break
                        }
                        // Speed controls
                        KeyCode::Up => {
                            self.settings.frame_delay_ms =
                                self.settings.frame_delay_ms.saturating_sub(5).max(5);
                        }
                        KeyCode::Down => {
                            self.settings.frame_delay_ms =
                                (self.settings.frame_delay_ms + 5).min(100);
                        }
                        // Density controls
                        KeyCode::Right => {
                            self.settings.density = (self.settings.density + 0.05).min(1.0);
                            self.settings.spawns_per_frame =
                                (self.settings.spawns_per_frame + 1).min(20);
                        }
                        KeyCode::Left => {
                            self.settings.density = (self.settings.density - 0.05).max(0.05);
                            self.settings.spawns_per_frame =
                                self.settings.spawns_per_frame.saturating_sub(1).max(1);
                        }
                        // Length controls
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            self.settings.max_length = (self.settings.max_length + 5).min(50);
                        }
                        KeyCode::Char('-') => {
                            self.settings.max_length =
                                self.settings.max_length.saturating_sub(5).max(5);
                        }
                        // Color schemes
                        KeyCode::Char('1') => {
                            self.settings.color_scheme = ColorScheme::Green;
                        }
                        KeyCode::Char('2') => {
                            self.settings.color_scheme = ColorScheme::Blue;
                        }
                        KeyCode::Char('3') => {
                            self.settings.color_scheme = ColorScheme::Red;
                        }
                        KeyCode::Char('4') => {
                            self.settings.color_scheme = ColorScheme::Purple;
                        }
                        KeyCode::Char('5') => {
                            self.settings.color_scheme = ColorScheme::Cyan;
                        }
                        KeyCode::Char('6') => {
                            self.settings.color_scheme = ColorScheme::Rainbow;
                        }
                        _ => {}
                    }
                }
            }

            // Update terminal size
            if let Ok((w, h)) = terminal::size() {
                self.width = w;
                self.height = h;
            }

            self.spawn_drops();

            let mut active_drops = vec![];

            for mut drop in self.drops.drain(..) {
                let draws = drop.update(self.height, self.settings.color_scheme);

                for (x, y, ch, color) in draws {
                    execute!(
                        stdout,
                        MoveTo(x, y),
                        SetForegroundColor(color),
                        Print(ch)
                    )?;
                }

                if !drop.is_done(self.height) {
                    active_drops.push(drop);
                }
            }

            self.drops = active_drops;
            stdout.flush()?;

            std::thread::sleep(Duration::from_millis(self.settings.frame_delay_ms));
        }

        // Cleanup
        execute!(
            stdout,
            Show,
            EnableLineWrap,
            SetForegroundColor(Color::Reset),
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        terminal::disable_raw_mode()?;

        Ok(())
    }
}

fn print_help() {
    println!("Matrix Rain Terminal Screensaver");
    println!();
    println!("USAGE: matrix [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -s, --speed <MS>       Frame delay in ms (default: 50, lower = faster)");
    println!("  -d, --density <0-100>  Spawn density percentage (default: 40)");
    println!("  -n, --spawns <N>       Max spawns per frame (default: 4)");
    println!("  -l, --length <N>       Max drop length (default: 30)");
    println!("  -c, --color <SCHEME>   Color: green, blue, red, purple, cyan, rainbow");
    println!("  -h, --help             Show this help");
    println!();
    println!("RUNTIME CONTROLS:");
    println!("  ↑/↓         Adjust speed (faster/slower)");
    println!("  ←/→         Adjust density (less/more drops)");
    println!("  +/-         Adjust drop length");
    println!("  1-6         Color schemes (green/blue/red/purple/cyan/rainbow)");
    println!("  q/Esc/Enter/Space/Ctrl+C  Quit");
    println!();
    println!("PRESETS:");
    println!("  Gentle:   matrix -s 40 -d 20 -n 3 -l 20");
    println!("  Sparse:   matrix -s 50 -d 10 -n 2 -l 15");
    println!("  Chaos:    matrix -s 5 -d 90 -n 15 -l 45 -c rainbow");
}

fn parse_args() -> Settings {
    let args: Vec<String> = env::args().collect();
    let mut settings = Settings::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-s" | "--speed" => {
                if let Some(val) = args.get(i + 1) {
                    settings.frame_delay_ms = val.parse().unwrap_or(30);
                    i += 1;
                }
            }
            "-d" | "--density" => {
                if let Some(val) = args.get(i + 1) {
                    let pct: f64 = val.parse().unwrap_or(15.0);
                    settings.density = (pct / 100.0).clamp(0.01, 1.0);
                    i += 1;
                }
            }
            "-n" | "--spawns" => {
                if let Some(val) = args.get(i + 1) {
                    settings.spawns_per_frame = val.parse().unwrap_or(3);
                    i += 1;
                }
            }
            "-l" | "--length" => {
                if let Some(val) = args.get(i + 1) {
                    settings.max_length = val.parse().unwrap_or(25);
                    i += 1;
                }
            }
            "-c" | "--color" => {
                if let Some(val) = args.get(i + 1) {
                    if let Some(scheme) = ColorScheme::from_str(val) {
                        settings.color_scheme = scheme;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    settings
}

fn main() -> std::io::Result<()> {
    let settings = parse_args();

    println!("Matrix Rain - Press any exit key (q/Esc/Enter/Space/Ctrl+C)");
    println!("Controls: ↑↓ speed | ←→ density | +/- length | 1-6 colors");
    std::thread::sleep(Duration::from_millis(1500));

    let mut matrix = Matrix::new(settings);
    matrix.run()
}
