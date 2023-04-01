use ark_poly_commit::{kzg10::{KZG10}};
use ark_poly::{univariate::DensePolynomial};

use ark_bn254::Bn254;
use ark_serialize::CanonicalSerialize;
use rand::{rngs::OsRng};

const EVALUATION_DOMAIN_SIZE: usize = 1<<10;


fn main() {
    let crs = 
    KZG10::<Bn254, DensePolynomial<_>>::setup(
        EVALUATION_DOMAIN_SIZE*2,
        false,
        &mut OsRng
    ).unwrap();

    // open ../res/crs.bin for writing
    let mut f = std::fs::File::create("../res/crs.bin").unwrap();
    // serialize crs
    crs.serialize(&mut f).unwrap();
}
