/*
#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/poseidon.hpp>

using namespace nil::crypto3;
using namespace nil::crypto3::algebra::curves;
*/
#include <cstddef>
#include <vector>
#include <iostream>

//using value_type = typename pallas::base_field_type::value_type;
using value_type = size_t;
using value_vector = std::vector<value_type>;
using vv_it = typename value_vector::iterator;

value_type mock_hash(value_type a, value_type b)
{
    return a+b;
}

void print_array(vv_it begin, vv_it end, size_t stride = 1)
{
    std::cout << "stride = " << stride << " [ ";
    for(vv_it i = begin; i != end; i += stride) {
        std::cout << *i << " ";
    }
    std::cout << "]" << std::endl;
}

value_type evaluate_root(vv_it begin, vv_it end)
{
    size_t distance = end-begin;
    size_t stride = 1;

    while (stride < distance) {
        print_array(begin, end, stride);
        for(vv_it i = begin; i != end; i += stride) {
            //*i = hash<hashes::poseidon>(*i, *(i+stride));
            *i = mock_hash(*i, *(i+stride));
        }
        stride *= 2;
    }

    return *begin;
}


/* parameters summary:
 * total number of leaves: 1024
 * per prover: 16
 * total provers: 64
 */

[[circuit]] bool merkle_tree_poseidon (
    value_type expected_root,
    [[private_input]] value_vector layer0)
{
    
    vv_it begin;

    /*  layer 1 of provers */
    value_vector layer1(64);
    begin = layer0.begin();

// #pragma zk_multi_prover 0
    {
        layer1[0] = evaluate_root(begin +   0, begin +   16 );
    }
#pragma zk_multi_prover 1
    {
        layer1[1] = evaluate_root(begin +  16, begin +   32 );
    }
    /* .... */

    std::cout << "layer1:" << std::endl;
    print_array(layer1.begin(), layer1.end());

    /*  batch 1 of provers end, result in layer1 (len = 64) */

    /*  layer 2 of provers */
    value_vector layer2(4);
    begin = layer1.begin();

// #pragma zk_multi_prover 0
    {
        layer2[0] = evaluate_root(begin +   0, begin +   16 );
    }
#pragma zk_multi_prover 1
    {
        layer2[1] = evaluate_root(begin +  16, begin +   32 );
    }
    /* .... */

    /*  batch 2 of provers end, result in layer2 (len = 4) */
    
    std::cout << "layer2:" << std::endl;
    print_array(layer2.begin(), layer2.end());

    /* Last layer can be evaluated with one prover */
    begin = layer2.begin();
    value_type result = evaluate_root(begin, begin + 4);
     
    return (result == expected_root);
}

int main()
{
    size_t n = 1024;
    value_vector x(n);
    fill(x.begin(), x.end(), 1);

    bool result = merkle_tree_poseidon(1024, x);

    std::cout << "result: " << result << std::endl;
    return 0;
}
