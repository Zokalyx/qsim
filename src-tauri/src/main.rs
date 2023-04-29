#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};

mod formula;
mod linear;
mod complex;
use complex::Complex;
use formula::Formula;
use linear::{Matrix, Vector};

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

struct Experiment {
  potential: Formula
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet, formula_error, compute_formula, simulate])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}

#[tauri::command]
fn formula_error(formula: &str) -> String {
  match Formula::new(formula) {
    Err(error) => error,
    Ok(_) => "".into(),
  }
}

#[tauri::command]
fn compute_formula(formula: &str, start: f32, end: f32, resolution: u32) -> Datapoints {
  let formula = Formula::new(formula)
    .unwrap_or(Formula::new("0").unwrap());

  let step = (end - start) / (resolution as f32);
  let values = formula
    .get_vector(start, end, resolution)
    .into_iter()
    .enumerate()
    .map(|(i, y)| Datapoint::new(start + (i as f32) * step, y.modulus()))
    .collect();

  Datapoints::new(values)
}

#[tauri::command]
fn simulate(potential: &str, wavefunction: &str, start: f32, end: f32, resolution: u32) -> bool {
  let potential = Formula::new(potential);
  let potential = if let Ok(formula) = potential {
    Vector::from(formula.get_vector(start, end, resolution))
  } else {
    return false
  };

  let wavefunction = Formula::new(wavefunction);
  dbg!(&wavefunction);
  let wavefunction = if let Ok(formula) = wavefunction {
    Vector::from(formula.get_vector(start, end, resolution))
  } else {
    return false
  };

  let hamiltonian = hamiltonian(&potential);

  true
}

fn hamiltonian(potential: &Vector) -> Matrix {
  // http://facweb1.redlands.edu/fac/eric_hill/Phys341/Computation/Comp%201%20Setting%20up%20the%20discrete%20Schr%C3%B6dinger%20equation.pdf
  let n = potential.len();
  let mut h = Matrix::new(n, n);

  for i in 0..n {
    if i > 0 {
      h.set(i, i-1, (-1.0).into());
    }
    h.set(i, i, Complex::from(2.0) + potential[i]);
    if i < n-1 {
      h.set(i, i+1, (-1.0).into());
    }
  }

  h
}
