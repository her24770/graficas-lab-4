mod framebuffer;
mod obj;
mod triangle;

use framebuffer::Framebuffer;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm as glm;
use obj::{Model, load_obj};
use triangle::triangle;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const BACKGROUND: u32 = 0x120B1A; // color de fondo
const SHIP_COLOR: u32 = 0xBA68C8; // color de la nave

// Como se ve la nave (en grados). Cambialos para verla desde otro lado.
// VIEW_PITCH: inclinacion de la camara (90 = vista desde arriba).
// VIEW_YAW: giro alrededor de la nave.
const VIEW_PITCH: f32 = 50.0;
const VIEW_YAW: f32 = -20.0;

// Oscurece un color 0xRRGGBB multiplicandolo por k (0 a 1).
fn shade(color: u32, k: f32) -> u32 {
    let channel = |shift: u32| (((color >> shift) & 0xFF) as f32 * k).clamp(0.0, 255.0) as u32;
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn render(fb: &mut Framebuffer, model: &Model) {
    let base = fb.current_color(); // el color de la nave (se oscurece segun la luz)
    let (width, height) = (fb.width as f32, fb.height as f32);

    // 1. Girar todos los vertices para mirar la nave desde el angulo elegido
    let identity = glm::Mat4::identity();
    let rotation = glm::rotate(&identity, VIEW_PITCH.to_radians(), &glm::vec3(1.0, 0.0, 0.0))
        * glm::rotate(&identity, VIEW_YAW.to_radians(), &glm::vec3(0.0, 1.0, 0.0));

    let rotated: Vec<glm::Vec3> = model
        .vertices
        .iter()
        .map(|v| (rotation * glm::vec4(v.x, v.y, v.z, 1.0)).xyz())
        .collect();

    // 2. Calcular el centro y el tamano de la nave en pantalla, para
    //    dejarla en el centro y que ocupe ~80% de la ventana
    let mut min_p = glm::vec2(f32::MAX, f32::MAX);
    let mut max_p = glm::vec2(f32::MIN, f32::MIN);
    for v in &rotated {
        min_p = glm::vec2(min_p.x.min(v.x), min_p.y.min(v.y));
        max_p = glm::vec2(max_p.x.max(v.x), max_p.y.max(v.y));
    }
    let center = (min_p + max_p) / 2.0;
    let size = max_p - min_p;
    let scale = 0.8 * (width / size.x).min(height / size.y);

    // convierte un vertice 3D en un punto de la pantalla
    let to_screen = |v: &glm::Vec3| {
        glm::vec2(
            width / 2.0 + (v.x - center.x) * scale,
            height / 2.0 - (v.y - center.y) * scale,
        )
    };

    // 3. Recorrer TODAS las caras: tomar sus 3 indices, buscar los 3 vertices
    //    y guardarlas para dibujarlas
    struct Item {
        depth: f32,  // que tan lejos esta (para dibujar primero lo lejano)
        face: usize, // numero de cara
        light: f32,  // cuanta luz recibe (0 a 1)
        front: bool, // true si la cara mira hacia la camara
    }
    let light_dir = glm::normalize(&glm::vec3(-0.5, 0.6, 0.8));
    let mut items: Vec<Item> = Vec::new();

    for (i, f) in model.faces.iter().enumerate() {
        let (a, b, c) = (rotated[f[0]], rotated[f[1]], rotated[f[2]]);

        let normal = glm::normalize(&glm::cross(&(b - a), &(c - a)));
        let light = 0.25 + 0.75 * glm::dot(&normal, &light_dir).max(0.0);

        items.push(Item {
            depth: (a.z + b.z + c.z) / 3.0,
            face: i,
            light,
            front: normal.z > 0.0,
        });
    }

    // 4. Dibujar TODAS las caras: primero las que miran hacia atras (quedan
    //    tapadas), despues las de frente; dentro de cada grupo, de lo mas
    //    lejano a lo mas cercano
    items.sort_by(|p, q| {
        p.front
            .cmp(&q.front)
            .then(p.depth.partial_cmp(&q.depth).unwrap())
    });

    for item in &items {
        let f = model.faces[item.face];

        let a = to_screen(&rotated[f[0]]);
        let b = to_screen(&rotated[f[1]]);
        let c = to_screen(&rotated[f[2]]);

        fb.set_current_color(shade(base, item.light));
        triangle(fb, a, b, c);
    }

    fb.set_current_color(base);
}

fn main() {
    // Uso: cargo run --release -- [archivo.obj] [--save imagen.png]
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut model_path = String::from("models/nave.obj");
    let mut save_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--save" && i + 1 < args.len() {
            save_path = Some(args[i + 1].clone());
            i += 2;
        } else {
            model_path = args[i].clone();
            i += 1;
        }
    }

    let model = match load_obj(&model_path) {
        Ok(model) => model,
        Err(e) => {
            eprintln!(
                "No pude abrir {model_path}: {e} (ejecuta el programa desde la carpeta del proyecto)"
            );
            return;
        }
    };
    println!(
        "Modelo cargado: {} vertices, {} triangulos",
        model.vertices.len(),
        model.faces.len()
    );

    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    fb.set_background_color(BACKGROUND);

    // Con --save: dibuja una vez, guarda la imagen y sale (sin abrir ventana)
    if let Some(path) = &save_path {
        fb.clear();
        fb.set_current_color(SHIP_COLOR);
        render(&mut fb, &model);
        match fb.save_png(path) {
            Ok(()) => println!("Guardado {path}"),
            Err(e) => eprintln!("No pude guardar {path}: {e}"),
        }
        return;
    }

    let mut window = Window::new(
        "Software Renderer",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        fb.clear();

        fb.set_current_color(SHIP_COLOR); // pueden cambiar los colores
        render(&mut fb, &model);

        // tecla S: guardar una captura
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            match fb.save_png("render.png") {
                Ok(()) => println!("Guardado render.png"),
                Err(e) => eprintln!("No pude guardar render.png: {e}"),
            }
        }

        window
            .update_with_buffer(&fb.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
