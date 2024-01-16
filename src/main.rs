use minijinja::{context, path_loader, Environment, Error, ErrorKind};
use neptune::poseidon::{PoseidonConstants, Poseidon};
use pasta_curves::Fp;
use ff::Field;
use generic_array::typenum::U8;

use serde::{ser::{self, SerializeStruct}, Serialize, Serializer};
use clap::Parser;

use log::{info, debug};
use std::fs;

#[derive(Serialize, Debug)]
pub struct LayerConfig {
    prev_layer: usize,
    prev_layer_size: usize,
    layer_number: usize,
    layer_size: usize,
    prover_base: usize,
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
    let mut provers = 0;
    let mut prev_layer_size = witness_count;
    
    while current_layer_size > per_prover {
        layers.push( LayerConfig {
            prev_layer,
            prev_layer_size,
            layer_number: prev_layer + 1,
            layer_size: current_layer_size / per_prover,
            prover_base: provers,
            layer_leaves: vec![1; current_layer_size / per_prover]
        });
        current_layer_size = current_layer_size / per_prover;
        prev_layer = prev_layer + 1;
        prev_layer_size = current_layer_size;
        provers = provers + current_layer_size;
    }

    Config {
        witness_count,
        per_prover,
        prover_count: provers,
        layers,
        total_layers: prev_layer,
        last_layer_size: current_layer_size
    }
}

/*
fn ser_vector_fp<S>(v: &Vec<Fp>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    use serde::ser::{SerializeSeq, SerializeStruct};
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for x in v {
        let mut state = seq.serialize_struct("field", 1)?;
        let value = format!("{:?}", x);
        state.serialize_field("field", &value)?;
        state.end()?;
    }
    seq.end()
}
*/

#[derive(Debug)]
struct MyFp(Fp);

impl Serialize for MyFp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {

            let mut state = serializer.serialize_struct("field", 1)?;
            let value = format!("{:?}", self.0);
            state.serialize_field("field", &value)?;
            state.end()
    }
}

#[derive(Serialize, Debug)]
enum OneInput {
    #[serde(rename = "field")]
    Field(MyFp),
    #[serde(rename = "array")]
    Vector(Vec<MyFp>),
}

#[derive(Serialize, Debug)]
struct Input {
    inputs: Vec<OneInput>,
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
    fs::write(&circuit_filename, circuit).unwrap();
    info!("Circuit saved to {}", &circuit_filename);


    let private_input : Vec<OneInput> = vec![
        OneInput::Vector(vec![1;witness_count].into_iter()
        .map(|x| MyFp(x.into())).collect())];

    let pi_str = serde_json::to_string_pretty(&private_input).unwrap();
    let private_input_filename = format!("{}_private.inp", args.circuit);
    fs::write(&private_input_filename, pi_str).unwrap();
    info!("Private input saved to {}", &private_input_filename);

    let public_input_filename = format!("{}_public.inp", args.circuit);
    fs::write(&public_input_filename, "[]").unwrap();
    info!("Public input saved to {}", &public_input_filename);


    /* generate inputs 
    let witness = vec![1; witness_count];
    let consts : PoseidonConstants::<Fp, U8> = PoseidonConstants::new();
    let mut poseidon = Poseidon::<Fp, U8>::new(&consts);

    poseidon.input(0.into());
    poseidon.input(1.into());
    poseidon.input(2.into());
    let res = poseidon.hash();

    info!("hash: {:?}", res);
    */



}
