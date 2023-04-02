use ark_bn254::Bn254;
use ark_ec::{msm::VariableBaseMSM, PairingEngine, ProjectiveCurve};
use ark_ff::PrimeField;
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::kzg10::KZG10;
use ark_serialize::CanonicalSerialize;
use rand::rngs::OsRng;

pub const EVALUATION_DOMAIN_SIZE: usize = 1 << 10;
pub const NUM_OF_SHARDS: usize = 1 << 4;

// slice of points should be corresponding to values of polynomial we are going to commit
pub fn simple_commit<E: PairingEngine>(poly: &[E::Fr], points: &[E::G1Affine]) -> E::G1Affine {
    let scalars = poly.iter().map(|c| c.into_repr()).collect::<Vec<_>>();
    VariableBaseMSM::multi_scalar_mul(&points, &scalars).into_affine()
}

fn main() {
    let crs =
        KZG10::<Bn254, DensePolynomial<_>>::setup(EVALUATION_DOMAIN_SIZE * 2, true, &mut OsRng)
            .unwrap();

    // open ../res/crs.bin for writing
    let mut f = std::fs::File::create("../res/crs.bin").unwrap();
    // serialize crs
    crs.serialize(&mut f).unwrap();
}
