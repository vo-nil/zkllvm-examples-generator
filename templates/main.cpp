#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/poseidon.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include <nil/crypto3/algebra/curves/pallas.hpp>

using namespace nil::crypto3;
using namespace nil::crypto3::algebra::curves;

using value_type = typename {{hash_function}}::block_type;

template<std::size_t size>
value_type evaluate_root(
        typename std::array<value_type, size>::iterator begin,
        typename std::array<value_type, size>::iterator end,
        std::size_t distance)
{
    std::size_t stride = 1;

    while (stride < distance) {
        for(auto i = begin; i != end; i += 2*stride) {
            *i = hash<{{hash_function}}>(*i, *(i+stride));
        }
        stride *= 2;
    }
    return *begin;
}


/* parameters summary:
 * total number of leaves: {{ witness_count }}
 * per prover: {{ per_prover }}
 * total provers: {{ prover_count }}
 */

[[circuit]] value_type merkle_tree (
    [[private_input]] std::array<value_type, {{witness_count}}> layer0)
{
    
    {% for layer in layers %}
    {% include 'onelayer.cpp' %}
    {% endfor %}

    /* Last layer can be evaluated with one prover */
    value_type result;
#pragma zk_multi_prover {{ prover_count }} 
    {
        auto begin = layer{{total_layers}}.begin();
        result = evaluate_root<{{last_layer_size}}>(begin, begin + {{last_layer_size}}, {{last_layer_size}});
    }
     
    return result;
}

