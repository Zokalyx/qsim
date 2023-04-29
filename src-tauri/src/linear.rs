use std::ops::Index;
use crate::complex::Complex;

#[derive(Debug)]
pub struct Matrix {
    rows: usize,
    columns: usize,
    entries: Vec<Complex>
}
impl Matrix {
    pub fn new(rows: usize, columns: usize) -> Self {
        Self { rows, columns, entries: vec![Complex::zero(); rows*columns] }
    }

    pub fn get(&self, row: usize, column: usize) -> Option<Complex> {
        self.entries.get((self.columns * row) + column).copied()
    }

    /// Todo: Make iterator instead
    pub fn rows(&self) -> Vec<&[Complex]> {
        let mut rows = vec![];
        for row in 0..self.columns {
            rows.push(&self.entries[row * self.columns..(row + 1) * self.columns])     
        }
        rows
    }

    pub fn set(&mut self, row: usize, column: usize, value: Complex) -> bool {
        let mut entry = self.entries.get_mut((self.columns * row) + column);
        match entry {
            None => false,
            Some(entry) => {
                *entry = value;
                true
            }
        }
    }

    pub fn eigenproblem(&self, iterations: u32) -> Vec<(Complex, Vector)> {
        vec![]
    }
}

#[derive(Debug)]
pub struct Vector(Vec<Complex>);
impl Vector {
    pub fn from(vec: Vec<Complex>) -> Self {
        Self(vec)
    }

    pub fn new(n: usize) -> Self {
        Self(vec![Complex::zero(); n])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> std::slice::Iter<Complex> {
        self.0.iter()
    }

    pub fn inner_product(&self, other: &Vector) -> Result<Complex, String> {
        if self.len() != other.len() {
            return Err("Mismatched dimensions".into());
        }

        Ok(self
            .iter()
            .enumerate()
            .map(|(i, value)| value.conjugate() * other[i])
            .sum::<Complex>())
    }
}
impl Index<usize> for Vector {
    type Output = Complex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}