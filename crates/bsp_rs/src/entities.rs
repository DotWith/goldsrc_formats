use std::collections::HashMap;

use valve_fmt_shared::Vec3;

pub type Entity = HashMap<String, EntityValue>;

#[derive(Debug, PartialEq)]
pub enum EntityValue {
    String(String),
    Number(f32),
    Vec3(Vec3),
}

pub fn parse_entities(data: &str) -> Vec<Entity> {
    let mut map = Vec::new();

    for line in data.lines() {
        match line {
            "{" => map.push(Entity::default()),
            "}" => {}
            line => process_line(line, &mut map),
        }
    }

    map
}

fn process_line(line: &str, map: &mut Vec<Entity>) {
    let mut kv = line.split('"');
    if let Some(key) = kv.nth(1) {
        if let Some(value) = kv.nth(1) {
            if let Some(entity) = map.last_mut() {
                if let Ok(number) = value.parse::<f32>() {
                    entity.insert(key.to_string(), EntityValue::Number(number));
                    return;
                } else if let Some(vec3) = parse_vec3(value) {
                    entity.insert(key.to_string(), EntityValue::Vec3(vec3));
                    return;
                }
                entity.insert(key.to_string(), EntityValue::String(value.to_string()));
            }
        }
    }
}

fn parse_vec3(value: &str) -> Option<Vec3> {
    let components: Vec<&str> = value.split_whitespace().collect();

    if components.len() == 3 {
        if let (Ok(x), Ok(y), Ok(z)) = (
            components[0].parse(),
            components[1].parse(),
            components[2].parse(),
        ) {
            return Some(Vec3::new(x, y, z));
        }
    }

    None
}
