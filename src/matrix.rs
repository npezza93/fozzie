use std::ops::{Index, IndexMut};

type Idx = (usize, usize);

pub struct Matrix {
    width: usize,
    contents: Vec<f32>,
}

impl Matrix {
    pub fn new(width: usize, height: usize) -> Self {
        Matrix {
            width,
            contents: vec![0 as f32; width * height],
        }
    }
}

impl Index<Idx> for Matrix {
    type Output = f32;

    fn index(&self, (width, height): Idx) -> &Self::Output {
        &self.contents[height * self.width + width]
    }
}

impl IndexMut<Idx> for Matrix {
    fn index_mut(&mut self, (width, height): Idx) -> &mut Self::Output {
        &mut self.contents[height * self.width + width]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matrix() {
        let mut matrix = Matrix::new(1024, 768);

        assert_eq!(matrix[(1023, 767)], 0.0);

        matrix[(12, 24)] = 123.456;
        assert_eq!(matrix[(12, 24)], 123.456);
        assert_eq!(matrix[(24, 12)], 0.0);
    }
}
