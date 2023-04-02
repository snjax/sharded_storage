use std::iter::repeat_with;

// use ark_poly::domain::radix2::Radix2EvaluationDomain;
use ark_ff::fields::Field;
use ark_bn254::Fr;

/// Defines a set of points to evaluate the polynomial in.
pub struct Domain {
    pub g: Fr,
    pub k: usize,
    pub degrees: Vec<Fr>,
}

impl Domain {

    /// If you want to encode a vector of 2^k filed elements, use
    /// `Domain::from_k(k)`.
    pub fn from_k(k: usize) -> Self {
        // This is hardcoded for ark_bn254::Fr
        let n : u64 = 28;
        /// 5 must generate the whole Fr^*
        let g = Fr::from(5).pow(vec![n - k as u64].as_slice());
        Self::new(g, k)
    }

    /// `g` is the generator of the domain group <g>, k defines the number of
    /// elements 2^k to evaluate the polynomial in.
    pub fn new(g: Fr, k: usize) -> Self {
        let mut acc = g.clone();
        let degrees = repeat_with(|| {
            acc *= g;
            acc.clone()
        }).take(1 << (k + 1)).collect();
        Domain { g, k, degrees }
    }

    /// Takes a list of (x, y) points, produces the list of polynomial's
    /// coefficients
    fn interpolate(ps: &Vec<(Fr, Fr)>, xs: &Vec<Fr>) -> Vec<Fr> {
        // computes δ_j(x)
        let delta = |j: usize, x: &Fr| -> Fr {
            let (x_j, _y_j) = ps[j];
            ps.iter().enumerate().map(|(m, (x_m, _y_m))| {
                if m != j {
                    (x - x_m) / (x_j - x_m)
                } else {
                    Fr::ONE
                }
            }).product()
        };
        xs.iter().map(|x| {
            ps.iter().enumerate().map(|(m, (_, y_m))| {
                delta(m, x) * y_m
            }).sum()
        }).collect()
    }

    /// Takes 2^k values, encodes them as 2^(1+k) values.
    ///
    /// The original values can be found inside the codeword on even
    /// positions. The odd positions are filled with the other correlated data
    /// (polynomial evaluations that serve as "checksums"):
    ///
    /// ```
    /// use shamir_ss::Domain;
    /// use ark_ff::fields::Field;
    /// use ark_bn254::Fr;
    /// let d = Domain::from_k(2);
    /// let v : Vec<Fr> = vec![1, 2, 3, 4].iter().map(|&x| Fr::from(x)).collect();
    /// let c = d.encode(v.clone());
    /// assert_eq!(v, vec![c[0], c[2], c[4], c[6]]);
    /// ```
    pub fn encode(&self, value: Vec<Fr>) -> Vec<Fr> {
        let even = (0..).step_by(2).take(1 << self.k);
        let odd  = (1..).step_by(2).take(1 << self.k);

        let known = even.zip(value.iter()).map(|(i, y_i)| {
            (self.degrees[i], y_i.clone())
        }).collect();
        let wanted = odd.map(|i| self.degrees[i]).collect();
        let extra = Self::interpolate(&known, &wanted);
        let mut res = vec![];
        // intersperse even and odd
        for (e, o) in value.into_iter().zip(extra.into_iter()) {
            res.push(e);
            res.push(o);
        }
        res
    }

    /// Takes 2k values, some (but no more than k) of which are unknown/lost, and
    /// decodes back the k. This inverses the result of encode:
    ///
    /// ```
    /// use shamir_ss::Domain;
    /// use ark_ff::fields::Field;
    /// use ark_bn254::Fr;
    /// let d = Domain::from_k(2);
    /// let v : Vec<Fr> = vec![1, 2, 3, 4].iter().map(|&x| Fr::from(x)).collect();
    /// let c = d.encode(v.clone()).into_iter().map(|x| Some(x)).collect();
    /// assert_eq!(Some(v), d.decode(&c));
    /// ```
    ///
    /// even if some values are lost
    ///
    /// ```
    /// use shamir_ss::Domain;
    /// use ark_ff::fields::Field;
    /// use ark_bn254::Fr;
    /// let d = Domain::from_k(2);
    /// let v : Vec<Fr> = vec![1, 2, 3, 4].iter().map(|&x| Fr::from(x)).collect();
    /// let mut c : Vec<Option<Fr>> = d.encode(v.clone()).into_iter().map(|x| Some(x)).collect();
    /// c[0] = None;
    /// c[1] = None;
    /// c[4] = None;
    /// assert_eq!(Some(v), d.decode(&c));
    /// ```
    ///
    /// but if you erase more than half of the points, the decoding will fail:
    ///
    /// ```
    /// use shamir_ss::Domain;
    /// use ark_ff::fields::Field;
    /// use ark_bn254::Fr;
    /// let d = Domain::from_k(2);
    /// let v : Vec<Fr> = vec![1, 2, 3, 4].iter().map(|&x| Fr::from(x)).collect();
    /// let mut c : Vec<Option<Fr>> = d.encode(v.clone()).into_iter().map(|x| Some(x)).collect();
    /// c[0] = None;
    /// c[1] = None;
    /// c[3] = None;
    /// c[4] = None;
    /// assert_eq!(Some(v), d.decode(&c));
    /// ```
    pub fn decode(&self, code: &Vec<Option<Fr>>) -> Option<Vec<Fr>> {
        let known : Vec<_> = self.degrees.iter().zip(code).flat_map(|(x, y)| {
            y.map(|y| (x.clone(), y))
        }).collect();
        let wanted = self.degrees.iter().cloned().step_by(2).take(1 << self.k).collect();
        if known.len() >= (1 << self.k) {
            Some(Self::interpolate(&known, &wanted))
        } else {
            None
        }
    }

}
