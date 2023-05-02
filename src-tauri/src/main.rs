#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use eigenvalues::{self, utils::sort_eigenpairs};
use nalgebra::{
    allocator::Allocator,
    base::{DMatrix, Matrix},
    DefaultAllocator,
};
use serde::{Deserialize, Serialize};
use tauri::State;

mod complex;
mod formula;
mod linear;
use complex::Complex;
use formula::Formula;
use linear::{Matrix as m, Vector};
use std::sync::Mutex;

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
    values: Vec<Datapoint>,
}
impl Datapoints {
    fn new(values: Vec<Datapoint>) -> Self {
        Self { values }
    }
}

struct ExperimentState {
    state: Mutex<Option<Experiment>>,
}
#[derive(Debug)]
struct Experiment {
    resolution: u32,
    potential: Vector,
    wavefunction: Vector,
    eigenvalues: Vector,
    eigenvectors: Vec<Vector>,
    coefficients: Vector,
}

fn main() {
    tauri::Builder::default()
        .manage(ExperimentState {
            state: Default::default(),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            formula_error,
            compute_formula,
            simulate,
            get_eigenvector,
            evolve,
            restart
        ])
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
fn compute_formula(
    formula: &str,
    start: f32,
    end: f32,
    resolution: u32,
    normalize: bool,
) -> Datapoints {
    let formula = Formula::new(formula).unwrap_or(Formula::new("0").unwrap());

    let step = (end - start) / (resolution as f32);
    let values = formula.get_vector(start, end, resolution);
    let mut values = Vector::from(values);

    if normalize {
        values.normalize();
    }

    let values = values
        .to_vec()
        .into_iter()
        .enumerate()
        .map(|(i, y)| Datapoint::new(start + (i as f32) * step, y.real))
        .collect();

    Datapoints::new(values)
}

#[tauri::command]
fn get_eigenvector(
    n: usize,
    state: State<ExperimentState>,
    start: f32,
    end: f32,
    resolution: u32,
) -> Datapoints {
    let data = state.state.lock().unwrap();

    // vvv huh? vvv
    match &*data {
        None => Datapoints { values: vec![] },
        Some(experiment) => {
            let step = (end - start) / resolution as f32;
            let values = &experiment.eigenvectors[n];
            let values = values
                .iter()
                .enumerate()
                .map(|(i, y)| Datapoint::new(start + (i as f32) * step, y.real))
                .collect();

            Datapoints { values }
        }
    }
}

#[tauri::command]
fn simulate(
    potentialFormula: &str,
    potentialDatapoints: Datapoints,
    usePotentialFormula: bool,
    wavefunctionFormula: &str,
    wavefunctionDatapoints: Datapoints,
    useWavefunctionFormula: bool,
    start: f32,
    end: f32,
    resolution: u32,
    state: State<ExperimentState>,
    momentum: f32,
) -> bool {
    let potential = if usePotentialFormula {
        let potential = Formula::new(potentialFormula);
        if let Ok(formula) = potential {
            Vector::from(formula.get_vector(start, end, resolution))
        } else {
            return false;
        }
    } else {
        Vector::from(potentialDatapoints.values.iter().map(|d| Complex::from(d.y)).collect::<Vec<Complex>>())
    };

    
    let mean_momentum = Formula::complex_phase(momentum);
    let mut wavefunction = if useWavefunctionFormula {
        if let Ok(formula) = Formula::new(wavefunctionFormula) {
            let formula = formula.adjoin(mean_momentum, formula::Operator::Multiplication);
            let mut wavefunction = Vector::from(formula.get_vector(start, end, resolution));
            wavefunction
        } else {
            return false;
        }
    } else {
        let mut mean_momentum_evaluated = vec![Complex::zero(); wavefunctionDatapoints.values.len()];
        for i in 0..wavefunctionDatapoints.values.len() {
            if let Ok(value) = mean_momentum.evaluate_complex(Complex::from(wavefunctionDatapoints.values[i].x)) {
                mean_momentum_evaluated[i] = value;
            } else {
                return false
            }
        }

        Vector::from(wavefunctionDatapoints.values
            .iter()
            .enumerate()
            .map(|(i, d)| Complex::from(d.y) * mean_momentum_evaluated[i])
            .collect::<Vec<Complex>>())
    };
    wavefunction.normalize();

    let hamiltonian = hamiltonian3(&potential);
    let eigensolutions = hamiltonian.symmetric_eigen();
    let eigensolutions = sort_eigenpairs(eigensolutions, true);

    let eigenvalues = eigensolutions
        .eigenvalues
        .iter()
        .map(|value| Complex::from(*value as f32))
        .collect::<Vec<Complex>>();
    let eigenvalues = Vector::from(eigenvalues);

    let eigenvectors = eigensolutions.eigenvectors;
    let eigenvectors = eigenvectors
        .column_iter()
        .map(|row| {
            let row = row
                .iter()
                .map(|value| Complex::from(*value as f32))
                .collect::<Vec<Complex>>();
            let mut vector = Vector::from(row);
            vector.normalize();
            vector
        })
        .collect::<Vec<Vector>>();

    let coefficients = Vector::from(
        eigenvectors
            .iter()
            .map(|eigenvector| eigenvector.inner_product(&wavefunction).unwrap())
            .collect::<Vec<Complex>>(),
    );

    *state.state.lock().unwrap() = Some(Experiment {
        potential,
        wavefunction,
        eigenvalues,
        eigenvectors,
        coefficients,
        resolution,
    });

    true
}

#[tauri::command]
fn restart(state: State<ExperimentState>) {
    let mut data = state.state.lock().unwrap();
    *data = None;
}

#[tauri::command]
fn evolve(time: f32, state: State<ExperimentState>, start: f32, end: f32) -> Datapoints {
    let data = state.state.lock().unwrap();

    match &*data {
        None => Datapoints { values: vec![] },
        Some(experiment) => {
            let mut result = Vector::new(experiment.resolution as usize);
            for i in 0..experiment.resolution {
                let mut c = experiment.coefficients[i as usize];
                if i > experiment.resolution / 2 {
                    c = Complex::zero();
                }
                let eivec = &experiment.eigenvectors[i as usize];
                let eival = experiment.eigenvalues[i as usize];
                result.add(
                    &eivec.scaled_by(
                        c * Complex::from(std::f32::consts::E)
                            .powf(&(-eival.times_i() * Complex::from(time))),
                    ),
                );
            }
            let step = (end - start) / experiment.resolution as f32;
            let values = result
                .to_vec()
                .iter()
                .enumerate()
                .map(|(i, y)| Datapoint::new(start + (i as f32) * step, y.modulus_squared()))
                .collect();
            Datapoints { values }
        }
    }
}

/*
#[tauri::command]
fn simulate2(
    potential_formula: &str,
    potential_datapoints: Datapoints,
    use_potential_formula: bool,
    wavefunction_formula: &str,
    wavefunction_datapoints: Datapoints,
    use_wavefunction_formula: bool,
    start: f32,
    end: f32,
    resolution: u32,
    state: State<ExperimentState>,
    mean_momentum: f32,
) -> bool {
    let potential = Formula::new(potential_formula);
    let potential = if let Ok(formula) = potential {
        Vector::from(formula.get_vector(start, end, resolution))
    } else {
        return false;
    };

    let mean_momentum = Formula::complex_phase(mean_momentum);
    let wavefunction = Formula::new(wavefunction_formula);
    if let Ok(formula) = wavefunction {
        let formula = formula.adjoin(mean_momentum, formula::Operator::Multiplication);
        let mut wavefunction = Vector::from(formula.get_vector(start, end, resolution));
        wavefunction.normalize();
        wavefunction
    } else {
        return false;
    };

    let hamiltonian = hamiltonian3(&potential);
    let eigensolutions = eigenvalues::lanczos::HermitianLanczos::new::<DMatrix<f64>>(
        hamiltonian,
        resolution as usize,
        eigenvalues::SpectrumTarget::Lowest,
    )
    .unwrap();

    let mut eigenvalues = eigensolutions
        .eigenvalues
        .iter()
        .map(|value| Complex::from(*value as f32))
        .collect::<Vec<Complex>>();
    // eigenvalues.sort_by(|a, b| a.modulus().partial_cmp(&b.modulus()).unwrap());
    let eigenvalues = Vector::from(eigenvalues);

    let mut eigenvectors = eigensolutions.eigenvectors;
    let eigenvectors = eigenvectors
        .row_iter()
        .map(|row| {
            let row = row
                .iter()
                .map(|value| Complex::from(*value as f32))
                .collect::<Vec<Complex>>();
            let mut vector = Vector::from(row);
            vector.normalize();
            vector
        })
        .collect::<Vec<Vector>>();

    let coefficients = Vector::from(
        eigenvectors
            .iter()
            .map(|eigenvector| {
                eigenvector
                  .inner_product(&wavefunction)
                  .unwrap()
            })
            .collect::<Vec<Complex>>(),
    );

    *state.state.lock().unwrap() = Some(Experiment {
        potential,
        wavefunction,
        eigenvalues,
        eigenvectors,
        coefficients,
        resolution,
    });

    true
}
*/

fn hamiltonian(potential: &Vector) -> m {
    // http://facweb1.redlands.edu/fac/eric_hill/Phys341/Computation/Comp%201%20Setting%20up%20the%20discrete%20Schr%C3%B6dinger%20equation.pdf
    let n = potential.len();
    let mut h = m::new(n, n);

    for i in 0..n {
        if i > 0 {
            h.set(i, i - 1, (-1.0).into());
        }
        h.set(i, i, Complex::from(2.0) + potential[i]);
        if i < n - 1 {
            h.set(i, i + 1, (-1.0).into());
        }
    }

    h
}

fn hamiltonian2(potential: &Vector) -> DMatrix<f32> {
    let n = potential.len();
    let mut h = DMatrix::from_element(n, n, 0.0);

    for i in 0..n {
        if i > 0 {
            h[i * (n + 1) - 1] = -1.0
        }
        h[i * (n + 1)] = potential[i].real + 2.0;
        if i < n - 1 {
            h[i * (n + 1) + 1] = -1.0
        }
    }

    h[0] = 1.0;

    h
}

fn hamiltonian3(potential: &Vector) -> DMatrix<f64> {
    let n = potential.len();
    DMatrix::from_fn(n, n, |i, j| {
        if i > 0 && j == i - 1 {
            -1.0f64
        } else if i == j {
            2.0f64 + (potential[i].real) as f64
        } else if j == i + 1 && i < n - 1 {
            -1.0f64
        } else {
            0.0f64
        }
    })
}
