// use ark_poly::domain::radix2::Radix2EvaluationDomain;
use ark_bn254::Fr;

/// Takes k values, encodes them as 2k values.
pub fn encode(value: &Vec<Fr>) -> Vec<Fr> {
    unimplemented!()
}

/// Takes 2k values, some (but no more than k) of which are unknown/lost, and
/// decodes back the k. This inverses the result of encode:
///
/// ```ignore
/// let vs = vec![1, 2, 3];
/// let code = encode(&vs);
/// assert_eq!(6, code.len());
/// let code = code.map(|x| Some(x)).collect();
/// assert_eq!(vs, decode(&code));
/// ```
///
/// even if some values are lost
///
/// ```ignore
/// let vs = vec![1, 2, 3];
/// let code = encode(&vs);
/// let mut code = code.map(|x| Some(x)).collect();
/// code[1] = None;
/// code[4] = None;
/// code[5] = None;
/// assert_eq!(vs, decode(&code));
/// ```
pub fn decode(code: &Vec<Option<Fr>>) -> Vec<Fr> {
  unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
