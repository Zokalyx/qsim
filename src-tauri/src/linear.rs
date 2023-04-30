use std::ops::{Index, Mul};
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

    pub fn get_row(&self, row: usize) -> &[Complex] {
        &self.entries[row * self.columns..(row + 1) * self.columns]
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
impl Mul<Vector> for Matrix {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        if self.columns != rhs.len() {
            panic!("Incompatible sizes");
        }

        let mut product = Vector::new(rhs.len());
        for (row, entry) in product.iter_mut().enumerate() {
            *entry = (0..self.columns)
                .map(|column| self.get(row, column).unwrap() * rhs[column])
                .sum()
        }

        product
    }
}

#[derive(Debug, Clone)]
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

    pub fn iter(&self) -> std::slice::Iter<Complex> {
        self.0.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Complex> {
        self.0.iter_mut()
    }

    pub fn to_vec(self) -> Vec<Complex> {
        self.0
    }

    fn scale(&mut self, scalar: Complex) {
        self.0 = self.0.iter().map(|value| *value * scalar).collect()
    }

    pub fn scaled_by(&self, scalar: Complex) -> Vector {
        let mut new = vec![];
        for i in 0..self.len() {
            new.push(self[i] * scalar);
        }
        Vector(new)
    }

    pub fn normalize(&mut self) {
        let length = self.inner_product(&self).unwrap().sqrt();
        if length.is_zero() {
            return
        }
        self.scale(length.inverse())
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

    pub fn add(&mut self, other: &Vector) {
        self.0 = self.0.iter().enumerate().map(|(i, value)| *value + other[i]).collect()
    }
}
impl Index<usize> for Vector {
    type Output = Complex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}