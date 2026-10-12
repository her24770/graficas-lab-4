use nalgebra_glm::Vec3;

// El modelo cargado desde un archivo OBJ.
pub struct Model {
    pub vertices: Vec<Vec3>,   // todos los puntos (x, y, z)
    pub faces: Vec<[usize; 3]>, // cada cara = 3 indices a `vertices` (desde 0)
}

// Lee un archivo .obj. Devuelve un error si no pudo abrirlo.
pub fn load_obj(path: &str) -> std::io::Result<Model> {
    let text = std::fs::read_to_string(path)?;

    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for line in text.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            // "v x y z": un vertice
            Some("v") => {
                let c: Vec<f32> = parts.filter_map(|s| s.parse().ok()).collect();
                if c.len() >= 3 {
                    vertices.push(Vec3::new(c[0], c[1], c[2]));
                }
            }
            // "f 1/1/1 2/2/2 3/3/3": una cara. De cada grupo solo importa el
            // primer numero (el vertice). El OBJ cuenta desde 1, por eso el -1.
            Some("f") => {
                let idx: Vec<usize> = parts
                    .filter_map(|token| token.split('/').next()?.parse::<usize>().ok())
                    .filter(|&i| i >= 1)
                    .map(|i| i - 1)
                    .collect();
                // (si alguna cara trae mas de 3 vertices, se parte en triangulos)
                for i in 1..idx.len().saturating_sub(1) {
                    faces.push([idx[0], idx[i], idx[i + 1]]);
                }
            }
            _ => {}
        }
    }

    Ok(Model { vertices, faces })
}
