use ark_bn254::Bn254;
use ark_ec::VariableBaseMSM;
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use once_cell::sync::OnceCell;

// FIXME: It's probably better to include the file itself in the binary since it's small.
const CR_PATH: &str = "../res/crs.bin";

pub fn commit<E: PairingEngine>(poly: &[E::Fr], points: &[E::G1Affine]) -> E::G1Affine {
    let scalars = poly.iter().map(|c| c.into_repr()).collect::<Vec<_>>();
    VariableBaseMSM::multi_scalar_mul(&points, &scalars).into_affine()
}
