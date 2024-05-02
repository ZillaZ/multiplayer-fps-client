use std::{
    fs::{read_to_string, write, File},
    io::Write,
};

use raylib::math::Vector3;

pub fn build_models(scene_path: &str, mtl_path: &str) {
    let data = std::fs::read_to_string(scene_path).unwrap();
    let material = std::fs::read_to_string(mtl_path).unwrap();
    let mut object_name = String::new();
    let mut model_file = File::create("static/models/none").unwrap();
    let mut material_file = File::create("static/models/none").unwrap();
    let mut last = String::new();
    let mut new_file = true;
    for line in data.lines() {
        let tokens = line.split_whitespace().collect::<Vec<&str>>();
        match tokens[0] {
            "o" => {
                new_file = create_object(&mut last, &mut model_file, &mut object_name, tokens);
            }
            "usemtl" => {
                if !new_file {
                    continue;
                }
                create_material(
                    &mut model_file,
                    &mut material_file,
                    &material,
                    &mut object_name,
                    tokens,
                )
            }
            _ => {
                if !new_file {
                    continue;
                }
                model_file.write(tokens[0..].join(" ").as_bytes()).unwrap();
                model_file.write("\n".as_bytes()).unwrap();
            }
        }
    }
    println!("{last}");
    let data = std::fs::read_to_string(format!("static/models/{}.obj", &last)).unwrap();
    let mut vertices = get_vertices(&data);
    move_to_origin(&mut vertices);
    update_file(format!("static/models/{}.obj", &last), data, vertices);
    fix_faces(format!("static/models/{}.obj", last));
}

fn create_material(
    model_file: &mut File,
    material_file: &mut File,
    material: &String,
    object_name: &mut String,
    tokens: Vec<&str>,
) {
    model_file.write(tokens.join(" ").as_bytes()).unwrap();
    model_file.write("\n".as_bytes()).unwrap();
    *material_file = File::create(format!("static/models/{}.mtl", object_name)).unwrap();
    let header = "# Blender 4.0.2 MTL File: 'None'\n# www.blender.org\n\n";
    material_file.write(header.as_bytes()).unwrap();
    let begin = material
        .lines()
        .position(|x| x == &format!("newmtl {}", tokens[1]))
        .unwrap();
    let mut end = 0;

    for line in material.lines().collect::<Vec<&str>>()[begin..].iter() {
        if line.len() == 0 {
            end = end + begin + 1;
            break;
        }
        end += 1;
    }
    if end < begin {
        end = end + begin;
    }
    let material_data = &material.lines().collect::<Vec<&str>>()[begin..end];
    let material_data = material_data.join("\n");
    material_file.write(material_data.as_bytes()).unwrap();
}

fn get_vertices(file: &str) -> Vec<Vec<f32>> {
    let mut lines = Vec::<Vec<f32>>::new();
    for line in file.lines() {
        if &line[0..2] != "v " {
            continue;
        }
        let vertices = &line.split_whitespace().collect::<Vec<&str>>()[1..];
        let vertices = vertices
            .into_iter()
            .map(|x| x.parse::<f32>().unwrap())
            .collect::<Vec<f32>>();
        lines.push(vertices);
    }
    lines
}

fn move_to_origin(vertices: &mut Vec<Vec<f32>>) -> Vector3 {
    if vertices.len() < 1 {
        return Vector3::zero();
    }
    let mins = get_max_axis(vertices);
    let (min_x, max_x) = (mins[0].0, mins[0].1);
    let (min_y, max_y) = (mins[1].0, mins[1].1);
    let (min_z, max_z) = (mins[2].0, mins[2].1);

    let diff_x = (max_x + min_x) / 2.0;
    let diff_y = (max_y + min_y) / 2.0;
    let diff_z = (max_z + min_z) / 2.0;
    let obj_position = Vector3::new(diff_x, diff_y, diff_z);
    for vertice in vertices.iter_mut() {
        vertice[0] -= diff_x;
        vertice[1] -= diff_y;
        vertice[2] -= diff_z;
    }
    obj_position
}

fn get_max_axis(vertices: &mut Vec<Vec<f32>>) -> Vec<(f32, f32)> {
    if vertices.len() < 1 {
        return Vec::new();
    }
    let mut max_x = vertices.iter().map(|x| x[0]).collect::<Vec<f32>>();
    max_x.sort_by(|a, b| a.total_cmp(b));
    let mut max_y = vertices.iter().map(|x| x[1]).collect::<Vec<f32>>();
    max_y.sort_by(|a, b| a.total_cmp(b));
    let mut max_z = vertices.iter().map(|x| x[2]).collect::<Vec<f32>>();
    max_z.sort_by(|a, b| a.total_cmp(b));
    vec![
        (max_x[0], max_x.pop().unwrap()),
        (max_y[0], max_y.pop().unwrap()),
        (max_z[0], max_z.pop().unwrap()),
    ]
}

fn create_object(
    last: &mut String,
    model_file: &mut File,
    object_name: &mut String,
    tokens: Vec<&str>,
) -> bool {
    let name = tokens[1].split("-").collect::<Vec<&str>>()[0]
        .trim_matches(|x: char| x.to_string().parse::<i32>().is_ok());
    let new_path = format!("static/models/{}.obj", name);
    let read = std::fs::read_to_string(&new_path);
    if read.is_ok() {
        return false;
    }
    let path = format!("static/models/{}.obj", last);
    if std::fs::read(&path).is_err() {
        File::create(&path).unwrap();
    }
    let read = std::fs::read_to_string(&path).unwrap();
    let mut vertices = get_vertices(&read);
    move_to_origin(&mut vertices);
    update_file(path, read, vertices);
    fix_faces(format!("static/models/{}.obj", last));
    *model_file = File::create(&new_path).unwrap();
    *last = name.into();
    *object_name = name.into();
    let header = format!(
        "# Blender 4.0.2\n# www.blender.org\nmtllib {}.mtl\no {}\n",
        name, name
    );
    model_file.write(header.as_bytes()).unwrap();
    true
}

fn update_file(path: String, data: String, vertices: Vec<Vec<f32>>) {
    let mut new_data = String::new();
    let mut count = 0;
    for line in data.lines() {
        if &line[0..2] != "v " {
            new_data.push_str(line);
        } else {
            new_data.push_str("v ");
            new_data.push_str(
                &vertices[count]
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            );
            count += 1;
        }
        new_data.push_str("\n");
    }
    write(path, new_data).unwrap();
}

fn fix_faces(model_path: String) {
    if let Err(_buf) = read_to_string(model_path.clone()) {
        return;
    }
    let buf = read_to_string(model_path.clone()).unwrap();
    if buf.len() < 1 {
        return;
    }
    let clone = buf.clone();
    let lines = clone.lines().collect::<Vec<&str>>();
    let mut faces = get_faces(lines);
    let fixed = fix_vertices(&mut faces, buf);

    write(model_path, fixed).unwrap();
}

fn get_faces(lines: Vec<&str>) -> Vec<&str> {
    lines
        .into_iter()
        .filter(|x| &x[0..1] == "f")
        .map(|x| &x[1..])
        .collect::<Vec<&str>>()
        .to_vec()
}

fn get_min_vertices(faces: &mut Vec<&str>) -> (u32, u32, u32) {
    let (mut min1, mut min2, mut min3) = (u32::MAX, u32::MAX, u32::MAX);
    for face in faces.iter() {
        let aux = face.split_whitespace().collect::<Vec<&str>>();
        let aux = aux
            .iter()
            .map(|x| {
                x.split("/")
                    .map(|x| x.parse::<u32>().unwrap())
                    .collect::<Vec<u32>>()
            })
            .collect::<Vec<Vec<u32>>>();
        let mut f1 = aux.iter().map(|x| x[0]).collect::<Vec<u32>>();
        let mut f2 = aux.iter().map(|x| x[1]).collect::<Vec<u32>>();
        let mut f3 = aux.iter().map(|x| x[2]).collect::<Vec<u32>>();
        f1.sort();
        f2.sort();
        f3.sort();
        min1 = std::cmp::min(min1, f1[0]);
        min2 = std::cmp::min(min2, f2[0]);
        min3 = std::cmp::min(min3, f3[0]);
    }
    println!("{}|{}|{}", min1, min2, min3);
    (min1, min2, min3)
}

fn fix_vertices(faces: &mut Vec<&str>, buf: String) -> String {
    let mut lines = String::new();
    let (min1, min2, min3) = get_min_vertices(faces);

    for face in faces.iter_mut() {
        let aux = face.split_whitespace().collect::<Vec<&str>>();
        let aux = aux
            .iter()
            .map(|x| {
                x.split("/")
                    .map(|x| x.parse::<u32>().unwrap())
                    .collect::<Vec<u32>>()
            })
            .collect::<Vec<Vec<u32>>>();
        let mut f1 = aux.iter().map(|x| x[0]).collect::<Vec<u32>>();
        let mut f2 = aux.iter().map(|x| x[1]).collect::<Vec<u32>>();
        let mut f3 = aux.iter().map(|x| x[2]).collect::<Vec<u32>>();
        f1 = f1.iter().map(|x| x - min1 + 1).collect();
        f2 = f2.iter().map(|x| x - min2 + 1).collect();
        f3 = f3.iter().map(|x| x - min3 + 1).collect();
        for i in 0..f1.len() {
            if i % 4 == 0 || i == 0 {
                lines.push_str("\nf ");
            }
            lines.push_str(&format!("{}/{}/{} ", f1[i], f2[i], f3[i]));
        }
    }
    let new_data = buf
        .lines()
        .filter(|x| &x[0..1] != "f")
        .collect::<Vec<&str>>();
    let mut new_data = new_data.join("\n");
    new_data.push_str(&lines);
    new_data
}

pub fn delete_models() {
    let files = std::fs::read_dir("static/models").unwrap();
    for entry in files {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if !name.contains("scene") {
            std::fs::remove_file(format!("static/models/{}", name)).unwrap();
        }
    }
}
