use minijinja::{context, path_loader, Environment, Error, ErrorKind};
use neptune::poseidon::{PoseidonConstants, Poseidon};
use pasta_curves::Fp;
use ff::Field;
use generic_array::typenum::U8;

use serde::Serialize;
use clap::Parser;

use log::{info, debug};
use std::fs;

#[derive(Serialize, Debug)]
pub struct LayerConfig {
    prev_layer: usize,
    layer_number: usize,
    layer_size: usize,
    layer_leaves: Vec<usize>,
}

#[derive(Serialize, Debug)]
pub struct Config {
    witness_count: usize,
    per_prover: usize,
    prover_count: usize,
    total_layers: usize,
    last_layer_size: usize,
    layers : Vec<LayerConfig>,
}

/// zkLLVM circuit generator for multiple provers testing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Circuit basename (circuit will be saved to 'circuit.cpp' and inputs will be
    /// 'circuit_private.inp' and 'circuit_public.inp')
    #[arg(short, long, default_value = "circuit" )]
    circuit: String,

    /// Total number of leaves
    #[arg(short, long, default_value_t = 1024)]
    leaves: usize,

    /// Prover capacity - maximum number of leaves per prover
    #[arg(short, long, default_value_t = 16)]
    prover_capacity: usize
}


fn build_config(witness_count: usize, per_prover: usize) -> Config {

    let mut layers = vec![];

    let mut current_layer_size = witness_count;
    let mut prev_layer = 0;
    
    while current_layer_size >= per_prover {
        layers.push( LayerConfig {
            prev_layer,
            layer_number: prev_layer + 1,
            layer_size: current_layer_size / per_prover,
            layer_leaves: vec![1; current_layer_size / per_prover]
        });
        current_layer_size = current_layer_size / per_prover;
        prev_layer = prev_layer + 1;
    }

    Config {
        witness_count,
        per_prover,
        prover_count: witness_count / per_prover,
        layers,
        total_layers: prev_layer,
        last_layer_size: current_layer_size
    }
}

fn main() {
    env_logger::init();
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    let tmpl = env.get_template("main.cpp").unwrap();
    
    let args = Args::parse();

    let witness_count = args.leaves;
    let per_prover = args.prover_capacity;

    let config = build_config( witness_count, per_prover);
    debug!("Circuit config: {:?}", config);

    /* generate circuit */
    let circuit = tmpl.render(&config).unwrap();
    let circuit_filename = format!("{0}.cpp", args.circuit);
    fs::write(circuit_filename, circuit).unwrap();
    info!("Circuit saved to {0}.cpp", args.circuit);

    /* generate inputs */
    let witness = vec![1; witness_count];
    let consts : PoseidonConstants::<Fp, U8> = PoseidonConstants::new();
    let mut poseidon = Poseidon::<Fp, U8>::new(&consts);

    poseidon.input(0.into());
    poseidon.input(1.into());
    poseidon.input(2.into());
    let res = poseidon.hash();

    info!("hash: {:?}", res);



}
