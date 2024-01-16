/*  layer {{ layer.layer_number }} of provers */
    std::array<value_type, {{ layer.layer_size }}> layer{{ layer.layer_number }};
    {
        auto begin = layer{{ layer.prev_layer }}.begin();

{% for leaf in layer.layer_leaves %}
{% if loop.index0 + layer.prover_base == 0 %} // #pragma zk_multi_prover 0
{% else %} #pragma zk_multi_prover {{ loop.index0 + layer.prover_base }} {% endif %}
        {
            layer{{layer.layer_number}}[{{loop.index0}}] = 
                evaluate_root<{{layer.prev_layer_size}}>(
                        begin +   {{ loop.index0 * per_prover }}, begin +   {{ (loop.index0+1) * per_prover}}, {{per_prover}} );
        }
{% endfor %}
    }
    /*  batch {{layer.layer_number}} of provers end, result in layer{{layer.layer_number}} (len = {{layer.layer_size}}) */
