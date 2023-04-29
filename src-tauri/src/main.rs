#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
mod formula;
mod linear;

#[derive(Deserialize, Serialize)]
struct Datapoint {
  x: f32,
  y: f32,
}
impl Datapoint {
  fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }
}

#[derive(Deserialize, Serialize)]
struct Datapoints {
  values: Vec<Datapoint>
}
impl Datapoints {
  fn new(values: Vec<Datapoint>) -> Self {
    Self { values }
  }
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet, formula_error, compute_formula])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}

#[tauri::command]
fn formula_error(formula: &str) -> String {
  match formula::Formula::new(formula) {
    Err(error) => error,
    Ok(_) => "".into(),
  }
}

#[tauri::command]
fn compute_formula(formula: &str, start: f32, end: f32, resolution: u32) -> Datapoints {
  let formula = formula::Formula::new(formula)
    .unwrap_or(formula::Formula::new("0").unwrap());

  let step = (end - start) / (resolution as f32);
  let mut values = vec![];
  for i in 0..resolution {
    let x = start + (i as f32) * step;
    values.push(Datapoint::new(x, formula.evaluate(x).unwrap_or(0.0)))
  }

  dbg!(&formula);
  Datapoints::new(values)
}
