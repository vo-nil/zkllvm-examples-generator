use minijinja::{context, path_loader, Environment, Error, ErrorKind};
use neptune::poseidon::{PoseidonConstants, Poseidon};
use pasta_curves::Fp;
use ff::Field;
use generic_array::typenum::U8;

use serde::{ser::{self, SerializeStruct, SerializeSeq}, Serialize, Serializer};
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

#[derive(clap::ValueEnum, Clone, Serialize, Debug, Default)]
enum HashFunction {
    #[default]
    Poseidon,
    Sha2_256,
    Sha2_512,
}

impl std::fmt::Display for HashFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Serialize, Debug)]
pub struct Config {
    hash_function: String,
    witness_count: usize,
    per_prover: usize,
    prover_count: usize,
    total_layers: usize,
    last_layer_size: usize,
    layers : Vec<LayerConfig>,
}

/// zkLLVM circuit generator for multiple provers testing
#[derive(Parser, Debug, Serialize)]
#[command(author, version, about, long_about = None)]
#[serde(rename_all = "lowercase")]
struct Args {
    /// Circuit basename (circuit will be saved to 'circuit.cpp' and inputs will be
    /// 'circuit_private.inp' and 'circuit_public.inp')
    #[arg(short, long, default_value = "circuit" )]
    circuit: String,

    /// Total number of leaves
    #[arg(short, long, default_value_t = 16)]
    leaves: usize,

    /// Prover capacity - maximum number of leaves per prover
    #[arg(short, long, default_value_t = 4)]
    prover_capacity: usize,

    /// Hash function. 
    #[arg(short='H', long, default_value_t = HashFunction::Poseidon)]
    hash_function: HashFunction,
}


fn build_config(args: &Args) -> Config {

    let mut layers = vec![];

    let mut current_layer_size = args.leaves;
    let mut prev_layer = 0;
    let mut provers = 0;
    let mut prev_layer_size = args.leaves;
    
    while current_layer_size > args.prover_capacity {
        layers.push( LayerConfig {
            prev_layer,
            prev_layer_size,
            layer_number: prev_layer + 1,
            layer_size: current_layer_size / args.prover_capacity,
            prover_base: provers,
            layer_leaves: vec![1; current_layer_size / args.prover_capacity]
        });
        current_layer_size = current_layer_size / args.prover_capacity;
        prev_layer = prev_layer + 1;
        prev_layer_size = current_layer_size;
        provers = provers + current_layer_size;
    }

    let hash_function = match args.hash_function {
        HashFunction::Poseidon => "hashes::poseidon",
        HashFunction::Sha2_256 => "hashes::sha2<256>",
        HashFunction::Sha2_512 => "hashes::sha2<512>",
    };

    Config {
        hash_function: hash_function.to_string(),
        witness_count: args.leaves,
        per_prover: args.prover_capacity,
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
/*
fn serialize_fp<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serializer
*/

#[derive(Debug)]
struct MyFp(Fp);

#[derive(Debug, Serialize)]
enum ValueType {
    #[serde(rename = "vector")]
    Vector(Vec<MyFp>),
    #[serde(rename = "field", untagged)]
    Field(MyFp),
}

impl Serialize for MyFp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
            let mut state = serializer.serialize_struct("field", 1)?;
            let value = format!("{:?}", self.0);
            state.serialize_field("field", &value)?;
            state.end()
    }
}

#[derive(Serialize, Debug)]
enum OneInput {
    #[serde(rename = "array")]
    Vector(Vec<ValueType>),
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

    let config = build_config(&args);
    debug!("Circuit config: {:?}", config);

    /* generate circuit */
    let circuit = tmpl.render(&config).unwrap();
    let circuit_filename = format!("{0}.cpp", args.circuit);
    fs::write(&circuit_filename, circuit).unwrap();
    info!("Circuit saved to {}", &circuit_filename);


    let private_input : Vec<OneInput> = match args.hash_function {
        HashFunction::Poseidon => {
            vec![
                OneInput::Vector((1..args.leaves as u64+1)
                    .map(|x| ValueType::Field(MyFp(x.into()))).collect())]
        },
        _ => {
            vec![
                OneInput::Vector((1..args.leaves as u64+1)
                    .map(|x| ValueType::Vector(vec![MyFp(0.into()), MyFp(x.into())])).collect())]
        }
    };

    let private_input_str = serde_json::to_string_pretty(&private_input).unwrap();
    let private_input_filename = format!("{}_private.inp", args.circuit);
    fs::write(&private_input_filename, &private_input_str).unwrap();
    info!("Private input saved to {}", &private_input_filename);

    let public_input : Vec<OneInput> = vec![];

    let public_input_str = serde_json::to_string_pretty(&public_input).unwrap();
    let public_input_filename = format!("{}_public.inp", args.circuit);
    fs::write(&public_input_filename, &public_input_str).unwrap();
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
