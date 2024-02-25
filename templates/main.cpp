#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/poseidon.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include <nil/crypto3/algebra/curves/pallas.hpp>

using namespace nil::crypto3;
using namespace nil::crypto3::algebra::curves;

using value_type = typename {{hash_function}}::block_type;

{% for ev in evaluate_roots %}
{% include 'evaluate_root.cpp' %}
{% endfor %}

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
        result = evaluate_root_{{last_layer_size}}_0_{{last_layer_size}}_{{last_layer_size}}(layer{{total_layers}});
    }

    return result;
}

