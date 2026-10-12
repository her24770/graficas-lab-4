// Framebuffer: buffer de pixeles en memoria, un u32 por pixel (0xRRGGBB).
// Es el formato que espera minifb en Window::update_with_buffer.

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    background_color: u32,
    current_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![0x000000; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
        }
    }

    // Pinta toda la pantalla con el color de fondo.
    pub fn clear(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = self.background_color;
        }
    }

    // Asigna el color de fondo.
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    // Asigna el color activo (con el que pinta point).
    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    // Devuelve el color activo.
    pub fn current_color(&self) -> u32 {
        self.current_color
    }

    // Pinta UN pixel (x, y) con el color activo. Si cae fuera de la pantalla, lo ignora.
    pub fn point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.buffer[y * self.width + x] = self.current_color;
        }
    }

    // Guarda el framebuffer como PNG (sin librerias extra).
    // Escribe un PNG valido con los datos sin comprimir.
    pub fn save_png(&self, path: &str) -> std::io::Result<()> {
        let mut out: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

        // IHDR: ancho, alto, 8 bits por canal, color RGB
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(self.width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.height as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
        write_chunk(&mut out, b"IHDR", &ihdr);

        // Datos crudos: cada fila empieza con un byte de filtro (0) y sigue en RGB
        let mut raw = Vec::with_capacity(self.height * (1 + self.width * 3));
        for y in 0..self.height {
            raw.push(0);
            for x in 0..self.width {
                let p = self.buffer[y * self.width + x];
                raw.push((p >> 16) as u8);
                raw.push((p >> 8) as u8);
                raw.push(p as u8);
            }
        }

        // zlib "sin comprimir": bloques de hasta 65535 bytes
        let mut z: Vec<u8> = vec![0x78, 0x01];
        let mut blocks = raw.chunks(65535).peekable();
        while let Some(block) = blocks.next() {
            let last = blocks.peek().is_none();
            z.push(if last { 1 } else { 0 });
            let len = block.len() as u16;
            z.extend_from_slice(&len.to_le_bytes());
            z.extend_from_slice(&(!len).to_le_bytes());
            z.extend_from_slice(block);
        }
        // suma de verificacion Adler-32
        let (mut a, mut b) = (1u32, 0u32);
        for &byte in &raw {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        write_chunk(&mut out, b"IDAT", &z);

        write_chunk(&mut out, b"IEND", &[]);
        std::fs::write(path, out)
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let mut body = Vec::with_capacity(4 + data.len());
    body.extend_from_slice(kind);
    body.extend_from_slice(data);
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc32(&body).to_be_bytes());
}
