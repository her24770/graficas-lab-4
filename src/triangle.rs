use crate::framebuffer::Framebuffer;
use nalgebra_glm::Vec2;

// Funcion auxiliar: de que lado de la linea A->B queda el punto P.
// Es positiva de un lado, negativa del otro y cero justo sobre la linea.
fn edge_function(a: &Vec2, b: &Vec2, p: &Vec2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}

// Pinta un triangulo relleno con el color activo del framebuffer.
// Recibe los 3 vertices ya en coordenadas de pantalla (en pixeles).
pub fn triangle(fb: &mut Framebuffer, a: Vec2, b: Vec2, c: Vec2) {
    // 1. Rectangulo (bounding box) que contiene al triangulo,
    //    recortado para no salirse de la pantalla
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0);
    let max_x = a.x.max(b.x).max(c.x).ceil().min(fb.width as f32 - 1.0);
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0);
    let max_y = a.y.max(b.y).max(c.y).ceil().min(fb.height as f32 - 1.0);
    if min_x > max_x || min_y > max_y {
        return; // el triangulo esta completamente fuera de la pantalla
    }

    // 2. Area del triangulo (con signo); si es 0 no hay nada que pintar
    let area = edge_function(&a, &b, &c);
    if area.abs() < 1e-6 {
        return;
    }

    // 3. Revisar cada pixel del rectangulo: se pinta si queda dentro del triangulo
    for y in (min_y as usize)..=(max_y as usize) {
        for x in (min_x as usize)..=(max_x as usize) {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5); // centro del pixel

            // coordenadas baricentricas: si las tres son >= 0, P esta dentro
            let w0 = edge_function(&b, &c, &p) / area;
            let w1 = edge_function(&c, &a, &p) / area;
            let w2 = edge_function(&a, &b, &p) / area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                fb.point(x, y);
            }
        }
    }
}
