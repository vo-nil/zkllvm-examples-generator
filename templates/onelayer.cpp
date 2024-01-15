/*  layer {{ layer.layer_number }} of provers */
    value_vector layer{{ layer.layer_number }}( {{ layer.layer_size }} );
    begin = layer{{ layer.prev_layer }}.begin();

{% for leaf in layer.layer_leaves %}
{% if loop.index0 == 0 %} // #pragma zk_multi_prover 0
{% else %} #pragma zk_multi_prover {{ loop.index0 }} {% endif %}
    {
        layer{{layer.layer_number}}[{{loop.index0}}] = evaluate_root(begin +   {{ loop.index0 * per_prover }}, begin +   {{ (loop.index0+1) * per_prover}} );
    }
{% endfor %}
    std::cout << "layer{{layer.layer_number}}:" << std::endl;
    print_array(layer{{layer.layer_number}}.begin(), layer{{layer.layer_number}}.end());
    /*  batch {{layer.layer_number}} of provers end, result in layer{{layer.layer_number}} (len = {{layer.layer_size}}) */
