//! Matrix utility class matching topologicpy Matrix API
//!
//! Provides 4x4 transformation matrices for 3D operations.

use std::f64::consts::PI;

/// A 4x4 matrix stored as a 2D array
pub type Matrix4x4 = [[f64; 4]; 4];

/// Matrix utility class with static methods
pub struct MatrixUtil;

impl MatrixUtil {
    /// Create an identity matrix
    pub fn identity() -> Matrix4x4 {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Create a rotation matrix from Euler angles (in degrees)
    /// Order: X, then Y, then Z
    pub fn by_rotation(rx: f64, ry: f64, rz: f64) -> Matrix4x4 {
        let rx_rad = rx.to_radians();
        let ry_rad = ry.to_radians();
        let rz_rad = rz.to_radians();

        // Rotation around X
        let cx = rx_rad.cos();
        let sx = rx_rad.sin();
        let mx = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cx, -sx, 0.0],
            [0.0, sx, cx, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Rotation around Y
        let cy = ry_rad.cos();
        let sy = ry_rad.sin();
        let my = [
            [cy, 0.0, sy, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sy, 0.0, cy, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Rotation around Z
        let cz = rz_rad.cos();
        let sz = rz_rad.sin();
        let mz = [
            [cz, -sz, 0.0, 0.0],
            [sz, cz, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Combine: Rz * Ry * Rx
        let temp = Self::multiply(mz, my);
        Self::multiply(temp, mx)
    }

    /// Create a rotation matrix around a specific axis (in degrees)
    pub fn by_axis_rotation(axis: [f64; 3], angle: f64) -> Matrix4x4 {
        let angle_rad = angle.to_radians();
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        let t = 1.0 - c;

        // Normalize axis
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if len < 1e-10 {
            return Self::identity();
        }
        let x = axis[0] / len;
        let y = axis[1] / len;
        let z = axis[2] / len;

        [
            [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
            [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
            [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Create a scaling matrix
    pub fn by_scaling(sx: f64, sy: f64, sz: f64) -> Matrix4x4 {
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, sz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Create a uniform scaling matrix
    pub fn by_uniform_scaling(s: f64) -> Matrix4x4 {
        Self::by_scaling(s, s, s)
    }

    /// Create a translation matrix
    pub fn by_translation(tx: f64, ty: f64, tz: f64) -> Matrix4x4 {
        [
            [1.0, 0.0, 0.0, tx],
            [0.0, 1.0, 0.0, ty],
            [0.0, 0.0, 1.0, tz],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// Add two matrices
    pub fn add(a: Matrix4x4, b: Matrix4x4) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = a[i][j] + b[i][j];
            }
        }
        result
    }

    /// Subtract matrix b from matrix a
    pub fn subtract(a: Matrix4x4, b: Matrix4x4) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = a[i][j] - b[i][j];
            }
        }
        result
    }

    /// Multiply two matrices
    pub fn multiply(a: Matrix4x4, b: Matrix4x4) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
    }

    /// Multiply a matrix by a scalar
    pub fn multiply_scalar(m: Matrix4x4, scalar: f64) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = m[i][j] * scalar;
            }
        }
        result
    }

    /// Transpose a matrix
    pub fn transpose(m: Matrix4x4) -> Matrix4x4 {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = m[j][i];
            }
        }
        result
    }

    /// Get the determinant of a 3x3 submatrix (for 4x4 inverse calculation)
    fn determinant_3x3(m: [[f64; 3]; 3]) -> f64 {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Get the determinant of a 4x4 matrix
    pub fn determinant(m: Matrix4x4) -> f64 {
        let mut det = 0.0;
        for j in 0..4 {
            let mut sub = [[0.0; 3]; 3];
            for i in 1..4 {
                let mut col = 0;
                for k in 0..4 {
                    if k != j {
                        sub[i - 1][col] = m[i][k];
                        col += 1;
                    }
                }
            }
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            det += sign * m[0][j] * Self::determinant_3x3(sub);
        }
        det
    }

    /// Compute the inverse of a 4x4 matrix (returns None if singular)
    pub fn inverse(m: Matrix4x4) -> Option<Matrix4x4> {
        let det = Self::determinant(m);
        if det.abs() < 1e-10 {
            return None;
        }

        // Compute adjugate matrix
        let mut adj = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let mut sub = [[0.0; 3]; 3];
                let mut sub_i = 0;
                for ii in 0..4 {
                    if ii == i {
                        continue;
                    }
                    let mut sub_j = 0;
                    for jj in 0..4 {
                        if jj == j {
                            continue;
                        }
                        sub[sub_i][sub_j] = m[ii][jj];
                        sub_j += 1;
                    }
                    sub_i += 1;
                }
                let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                adj[j][i] = sign * Self::determinant_3x3(sub);
            }
        }

        // Divide by determinant
        let inv_det = 1.0 / det;
        Some(Self::multiply_scalar(adj, inv_det))
    }

    /// Transform a point by a matrix
    pub fn transform_point(m: Matrix4x4, point: [f64; 3]) -> [f64; 3] {
        let x = m[0][0] * point[0] + m[0][1] * point[1] + m[0][2] * point[2] + m[0][3];
        let y = m[1][0] * point[0] + m[1][1] * point[1] + m[1][2] * point[2] + m[1][3];
        let z = m[2][0] * point[0] + m[2][1] * point[1] + m[2][2] * point[2] + m[2][3];
        let w = m[3][0] * point[0] + m[3][1] * point[1] + m[3][2] * point[2] + m[3][3];

        if w.abs() > 1e-10 {
            [x / w, y / w, z / w]
        } else {
            [x, y, z]
        }
    }

    /// Transform a direction vector by a matrix (ignores translation)
    pub fn transform_direction(m: Matrix4x4, dir: [f64; 3]) -> [f64; 3] {
        let x = m[0][0] * dir[0] + m[0][1] * dir[1] + m[0][2] * dir[2];
        let y = m[1][0] * dir[0] + m[1][1] * dir[1] + m[1][2] * dir[2];
        let z = m[2][0] * dir[0] + m[2][1] * dir[1] + m[2][2] * dir[2];
        [x, y, z]
    }

    /// Convert matrix to a flat vector (row-major)
    pub fn to_flat(m: Matrix4x4) -> Vec<f64> {
        let mut result = Vec::with_capacity(16);
        for i in 0..4 {
            for j in 0..4 {
                result.push(m[i][j]);
            }
        }
        result
    }

    /// Create matrix from a flat vector (row-major)
    pub fn from_flat(v: &[f64]) -> Option<Matrix4x4> {
        if v.len() != 16 {
            return None;
        }
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = v[i * 4 + j];
            }
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let id = MatrixUtil::identity();
        assert!((id[0][0] - 1.0).abs() < 1e-10);
        assert!((id[1][1] - 1.0).abs() < 1e-10);
        assert!((id[2][2] - 1.0).abs() < 1e-10);
        assert!((id[3][3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_translation() {
        let t = MatrixUtil::by_translation(1.0, 2.0, 3.0);
        let p = MatrixUtil::transform_point(t, [0.0, 0.0, 0.0]);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
        assert!((p[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_scaling() {
        let s = MatrixUtil::by_scaling(2.0, 3.0, 4.0);
        let p = MatrixUtil::transform_point(s, [1.0, 1.0, 1.0]);
        assert!((p[0] - 2.0).abs() < 1e-10);
        assert!((p[1] - 3.0).abs() < 1e-10);
        assert!((p[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_z_90() {
        let r = MatrixUtil::by_rotation(0.0, 0.0, 90.0);
        let p = MatrixUtil::transform_point(r, [1.0, 0.0, 0.0]);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
        assert!(p[2].abs() < 1e-10);
    }

    #[test]
    fn test_transpose() {
        let m = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        let t = MatrixUtil::transpose(m);
        assert!((t[0][1] - 5.0).abs() < 1e-10);
        assert!((t[1][0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_multiply() {
        let id = MatrixUtil::identity();
        let t = MatrixUtil::by_translation(1.0, 2.0, 3.0);
        let result = MatrixUtil::multiply(id, t);
        assert!((result[0][3] - 1.0).abs() < 1e-10);
        assert!((result[1][3] - 2.0).abs() < 1e-10);
        assert!((result[2][3] - 3.0).abs() < 1e-10);
    }
}
